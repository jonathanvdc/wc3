//! Sparse, owner-ordered ordinary model particle simulation before propagation.
use super::current_affine::{CurrentAffines, CurrentTransformSource, NodePoseSource};
use super::evaluation::{EvaluationFrame, ModelEvaluationWorld};
use super::model_step_order::{ModelStepOrder, ModelStepParent};
use super::mounts::MountedModel;
use super::{ModelRig, Playback, PoseOverrides};
use crate::child_coordinates::ChildCoordinates;
use crate::effects::model_children::{
    current_hierarchy_visible, synchronize_child, ModelParticleChild, ParticleModelSubtree,
};
use crate::effects::model_ownership::ModelParticleBirths;
use crate::effects::EffectChange;
use crate::effects::{EffectFrame, EffectWorld, ModelOutputs};
use crate::features::OwnedAttachmentClocks;
use crate::instances::{ModelInstance, OwnedModelFactory};
use crate::physics::PhysicsSimulation;
use crate::pose::PoseContext;
use bevy::ecs::change_detection::Tick;
use bevy::ecs::system::SystemState;
use bevy::math::Affine3A;
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};
use wc3::model::AnimationSample;

#[derive(Clone, Copy)]
pub(super) struct PreparedEffectStep {
    pub selection: AnimationSample,
    pub time: f64,
    pub change: EffectChange,
    pub basis: Mat4,
    pub camera: Option<Mat4>,
    pub visible: bool,
}
/// Only roots which actually own model-particle emitters pay for this state.
#[derive(Component, Default)]
pub(crate) struct ModelParticleDriver {
    pub(super) prepared: Option<PreparedEffectStep>,
}

type StepParameters = (
    EffectWorld<'static, 'static>,
    Query<'static, 'static, (&'static mut ModelRig, &'static PoseOverrides)>,
    ModelOutputs<'static, 'static>,
);
type ModelActors = Or<(With<ModelParticleDriver>, With<ParticleModelSubtree>)>;

pub(crate) struct ModelPipeline {
    evaluation: SystemState<ModelEvaluationWorld<'static, 'static, With<ParticleModelSubtree>>>,
    simulation: SystemState<StepParameters>,
    attachment_clocks: SystemState<OwnedAttachmentClocks<'static, 'static>>,
    factory: SystemState<OwnedModelFactory<'static, 'static>>,
    order: ModelStepOrder,
    affines: CurrentAffines,
    pending: VecDeque<Entity>,
    visited: HashSet<Entity>,
    entities: Vec<Entity>,
    actors: QueryState<Entity, ModelActors>,
}
impl ModelPipeline {
    fn new(world: &mut World) -> Self {
        Self {
            evaluation: SystemState::new(world),
            simulation: SystemState::new(world),
            factory: SystemState::new(world),
            attachment_clocks: SystemState::new(world),
            order: ModelStepOrder::default(),
            affines: CurrentAffines::default(),
            pending: VecDeque::new(),
            visited: HashSet::new(),
            entities: Vec::new(),
            actors: world.query_filtered(),
        }
    }
}
struct WorldFrames<'a>(&'a World);
impl CurrentTransformSource for WorldFrames<'_> {
    fn edge(&self, entity: Entity) -> Option<(Option<Entity>, Mat4)> {
        let world = self.0;
        let transform = world.get::<Transform>(entity)?;
        if let Some(source) = world.get::<NodePoseSource>(entity) {
            let rig = world.get::<ModelRig>(source.rig)?;
            if rig.nodes.get(source.index) != Some(&entity) {
                return None;
            }
            Some((Some(rig.basis_entity), *rig.poses.get(source.index)?))
        } else {
            Some((
                world.get::<ChildOf>(entity).map(ChildOf::parent),
                transform.to_matrix(),
            ))
        }
    }
}
fn actor_parent(world: &World, entity: Entity) -> Option<Entity> {
    if let Some(child) = world.get::<ModelParticleChild>(entity) {
        return Some(child.root);
    }
    if let Some(mount) = world.get::<MountedModel>(entity) {
        return Some(mount.parent);
    }
    world.get::<ChildCoordinates>(entity)?;
    let mut cursor = world.get::<ChildOf>(entity)?.parent();
    for _ in 0..4096 {
        if let Some(source) = world.get::<NodePoseSource>(cursor) {
            let rig = world.get::<ModelRig>(source.rig)?;
            return (rig.nodes.get(source.index) == Some(&cursor)).then_some(source.rig);
        }
        if world.get::<ModelRig>(cursor).is_some() {
            return Some(cursor);
        }
        cursor = world.get::<ChildOf>(cursor)?.parent();
    }
    None
}
/// Run after ordinary model poses and before Bevy's single propagation pass.
/// Application Transform edits ordered before this stage affect owned actors now.
pub(crate) fn simulate_model_particles(world: &mut World, mut state: Local<Option<ModelPipeline>>) {
    let pipeline = state.get_or_insert_with(|| ModelPipeline::new(world));
    pipeline.entities.clear();
    pipeline.entities.extend(pipeline.actors.iter(world));
    if pipeline.entities.is_empty() {
        return;
    }
    let evaluation_tick = pipeline.evaluation.meta().get_last_run();
    let simulation_tick = pipeline.simulation.meta().get_last_run();
    let factory_tick = pipeline.factory.meta().get_last_run();
    let attachments_tick = pipeline.attachment_clocks.meta().get_last_run();
    pipeline.affines.begin_frame();
    pipeline.visited.clear();
    pipeline.pending.clear();
    for &entity in &pipeline.entities {
        if let Some(mut driver) = world.get_mut::<ModelParticleDriver>(entity) {
            driver.prepared = None;
        }
    }
    let entries = pipeline.entities.iter().copied().map(|entity| {
        let owned = world.get::<ParticleModelSubtree>(entity).is_some();
        let parent = actor_parent(world, entity);
        let dependency = match parent {
            Some(parent)
                if world.get::<ModelParticleDriver>(parent).is_some()
                    || world.get::<ParticleModelSubtree>(parent).is_some() =>
            {
                ModelStepParent::Pending(parent)
            }
            Some(parent) if world.get_entity(parent).is_err() => ModelStepParent::Missing,
            None if owned => ModelStepParent::Missing,
            _ => ModelStepParent::Evaluated,
        };
        (entity, dependency)
    });
    pipeline
        .pending
        .extend(pipeline.order.prepare(entries).iter().copied());
    // Invalid ownership never reuses a previous transform/pose as a new source.
    for &entity in pipeline.order.invalid() {
        if let Some(mut visibility) = world.get_mut::<Visibility>(entity) {
            *visibility = Visibility::Hidden;
        }
    }
    while let Some(entity) = pipeline.pending.pop_front() {
        if !pipeline.visited.insert(entity) || world.get_entity(entity).is_err() {
            continue;
        }
        if !synchronize_owned_actor(world, entity) {
            continue;
        }
        if world.get::<ModelInstance>(entity).is_some() {
            pipeline.factory.meta_mut().set_last_run(factory_tick);
            if let Ok(mut factory) = pipeline.factory.get_mut(world) {
                factory.populate(entity);
            }
            pipeline.factory.apply(world);
            // An automatic attachment gains Playback only when its ready asset
            // is populated; inherit the current camera before its first pose.
            if !synchronize_owned_actor(world, entity) {
                continue;
            }
        }
        pipeline
            .attachment_clocks
            .meta_mut()
            .set_last_run(attachments_tick);
        if let Ok(mut clocks) = pipeline.attachment_clocks.get_mut(world) {
            clocks.restart(entity);
        }
        pipeline.attachment_clocks.apply(world);
        pipeline.affines.begin_frame();
        process_actor(world, pipeline, entity, evaluation_tick, simulation_tick);
        if let Some(mut births) = world.get_resource_mut::<ModelParticleBirths>() {
            pipeline.pending.extend(births.0.drain(..));
        }
    }
    let tick = world.read_change_tick();
    pipeline.evaluation.meta_mut().set_last_run(tick);
    pipeline.simulation.meta_mut().set_last_run(tick);
    pipeline.factory.meta_mut().set_last_run(tick);
    pipeline.attachment_clocks.meta_mut().set_last_run(tick);
}
fn synchronize_owned_actor(world: &mut World, entity: Entity) -> bool {
    let Some(parent) = actor_parent(world, entity) else {
        return world.get::<ParticleModelSubtree>(entity).is_none();
    };
    let Some(parent_playback) = world.get::<Playback>(parent).cloned() else {
        return false;
    };
    if world.get::<ModelParticleChild>(entity).is_some() {
        return synchronize_child(world, entity);
    } else if let Some(mount) = world.get::<MountedModel>(entity) {
        let clip = parent_playback
            .clip
            .and_then(|clip| mount.clips.get(&clip).copied());
        let inherited = world.get::<PhysicsSimulation>(entity) == mount.inherited_physics.as_ref();
        let source = mount.inherited_from.unwrap_or(parent);
        let physics = inherited
            .then(|| world.get::<PhysicsSimulation>(source).cloned())
            .flatten();
        if let Some(mut playback) = world.get_mut::<Playback>(entity) {
            if let Some(clip) = clip {
                playback.clip = Some(clip);
                playback.clip_time = parent_playback.clip_time;
                playback.shared_time = parent_playback.shared_time;
                playback.event_traversal = parent_playback.event_traversal;
                playback.transition_request = parent_playback.transition_request;
                playback.transition_clock = parent_playback.transition_clock;
            }
            playback.driving_camera = parent_playback.driving_camera;
            playback.speed = parent_playback.speed;
            playback.paused = parent_playback.paused;
        }
        if inherited {
            match physics.clone() {
                Some(settings) => {
                    world.entity_mut(entity).insert(settings);
                }
                None => {
                    world.entity_mut(entity).remove::<PhysicsSimulation>();
                }
            }
            if let Some(mut mount) = world.get_mut::<MountedModel>(entity) {
                mount.inherited_physics = physics;
                mount.inherited_from = Some(source);
            }
        }
    } else if let Some(mut playback) = world.get_mut::<Playback>(entity) {
        if playback.driving_camera != parent_playback.driving_camera {
            playback.driving_camera = parent_playback.driving_camera;
        }
    }
    true
}
fn process_actor(
    world: &mut World,
    pipeline: &mut ModelPipeline,
    entity: Entity,
    evaluation_tick: Tick,
    simulation_tick: Tick,
) {
    let Some(rig) = world.get::<ModelRig>(entity) else {
        return;
    };
    let basis_entity = rig.basis_entity;
    let Some(playback) = world.get::<Playback>(entity).cloned() else {
        return;
    };
    let Some(basis) = pipeline.affines.resolve(basis_entity, &WorldFrames(world)) else {
        return;
    };
    let camera = playback
        .driving_camera
        .and_then(|camera| pipeline.affines.resolve(camera, &WorldFrames(world)));
    let camera_local = match (playback.driving_camera, camera) {
        (None, _) => Ok(None),
        (Some(_), Some(camera)) if basis.determinant().abs() > 1e-12 => {
            Ok(Some(Affine3A::from_mat4(basis.inverse() * camera)))
        }
        _ => Err("invalid current model frame or driving camera"),
    };
    let physics_source = world
        .get::<MountedModel>(entity)
        .and_then(|mount| mount.inherited_from)
        .and_then(|source| world.get::<ModelRig>(source).map(|rig| rig.basis_entity))
        .and_then(|source| pipeline.affines.resolve(source, &WorldFrames(world)))
        .map(Affine3A::from_mat4);
    let owned = world.get::<ParticleModelSubtree>(entity).is_some();
    if owned {
        pipeline.evaluation.meta_mut().set_last_run(evaluation_tick);
        if let Ok(mut evaluation) = pipeline.evaluation.get_mut(world) {
            evaluation.evaluate_entity(
                entity,
                EvaluationFrame {
                    basis: Affine3A::from_mat4(basis),
                    camera: camera_local,
                    physics_source,
                },
            );
        }
        pipeline.evaluation.apply(world);
        pipeline.affines.begin_frame();
    }
    if world.get::<ModelParticleDriver>(entity).is_none() {
        return;
    }
    let Ok(selection) = playback.selection() else {
        return;
    };
    let root_visible = current_hierarchy_visible(world, entity);
    pipeline.simulation.meta_mut().set_last_run(simulation_tick);
    let mut prepared = None;
    if let Ok((mut effects, mut instances, mut outputs)) = pipeline.simulation.get_mut(world) {
        if let Ok((mut instance, overrides)) = instances.get_mut(entity) {
            let animation = instance.model.animation.clone();
            if let Ok(sample) = animation.sample(selection) {
                let ModelRig {
                    model,
                    poses,
                    visible,
                    particles,
                    transition,
                    ..
                } = &mut *instance;
                let (time, change) = model.particles.update_model_actors(
                    particles,
                    entity,
                    EffectFrame {
                        root_visible,
                        traversal: playback.event_traversal,
                        change: Default::default(),
                        time: playback.shared_time,
                        selection,
                        sample: &sample,
                        pose: Some(PoseContext {
                            nodes: &model.nodes,
                            transition: transition.view_at(playback.transition_clock),
                            overrides: &overrides.0,
                            camera: camera_local.ok().flatten(),
                        }),
                        node_poses: poses,
                        node_visible: visible,
                        basis_world: basis,
                    },
                    &mut effects,
                    &mut outputs,
                    playback.driving_camera,
                    !owned,
                );
                prepared = Some(PreparedEffectStep {
                    selection,
                    time,
                    change,
                    basis,
                    camera,
                    visible: root_visible,
                });
            }
        }
    }
    pipeline.simulation.apply(world);
    if let Some(mut driver) = world.get_mut::<ModelParticleDriver>(entity) {
        driver.prepared = prepared;
    }
    // Model projections changed detached roots without ordinary ChildOf edges.
    pipeline.affines.begin_frame();
}

#[cfg(test)]
#[path = "model_driver_tests.rs"]
mod tests;
