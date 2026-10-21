//! Sparse, owner-ordered ordinary model particle simulation before propagation.
use super::current_affine::{CurrentAffines, CurrentTransformSource, NodePoseSource};
use super::evaluation::{EvaluationFrame, ModelEvaluationWorld};
use super::model_step_order::{ModelStepOrder, ModelStepParent};
use super::model_tree::{actor_frame, prepare_tree, publish_tree};
use super::mounts::MountedModel;
use super::{ModelRig, Playback, PoseOverrides};
use crate::child_coordinates::ChildCoordinates;
use crate::effects::model_budget::{ModelActorLease, ModelActorTree};
use crate::effects::model_children::{
    current_hierarchy_visible, observe_model_start, synchronize_child, ModelParticleChild,
    ModelParticleStart, ParticleModelSubtree,
};
use crate::effects::model_ownership::ModelParticleBirths;
use crate::effects::EffectChange;
use crate::effects::{EffectFrame, EffectWorld, ModelOutputs, RuntimeParticles};
use crate::features::{ModelAttachment, OwnedAttachmentClocks};
use crate::instances::{ModelInstance, OwnedModelFactory, PreparedModelRequest};
use crate::physics::PhysicsSimulation;
use crate::pose::PoseContext;
use bevy::ecs::change_detection::Tick;
use bevy::ecs::system::SystemState;
use bevy::math::Affine3A;
use bevy::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use wc3::model::{AnimationSample, Time};

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
    birth_roots: HashMap<Entity, BirthActor>,
    birth_affines: CurrentAffines,
    pending: VecDeque<Entity>,
    visited: HashSet<Entity>,
    fresh: HashSet<Entity>,
    fresh_order: Vec<Entity>,
    birth_scratch: RefCell<Vec<usize>>,
    attachment_starts: HashSet<Entity>,
    entities: Vec<Entity>,
    actors: QueryState<Entity, ModelActors>,
    trees: QueryState<Entity, With<ModelActorTree>>,
    tree_roots: Vec<Entity>,
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
            birth_roots: HashMap::new(),
            birth_affines: CurrentAffines::default(),
            pending: VecDeque::new(),
            visited: HashSet::new(),
            fresh: HashSet::new(),
            fresh_order: Vec::new(),
            birth_scratch: RefCell::new(Vec::new()),
            attachment_starts: HashSet::new(),
            entities: Vec::new(),
            actors: world.query_filtered(),
            trees: world.query_filtered(),
            tree_roots: Vec::new(),
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
#[derive(Clone, Copy)]
struct BirthActor {
    root: Mat4,
    basis: Mat4,
    camera: Option<Mat4>,
    elapsed: f64,
}
struct BirthFrames<'a> {
    world: &'a World,
    roots: &'a HashMap<Entity, BirthActor>,
    scratch: &'a RefCell<Vec<usize>>,
}
impl CurrentTransformSource for BirthFrames<'_> {
    fn edge(&self, entity: Entity) -> Option<(Option<Entity>, Mat4)> {
        if let Some(start) = self.roots.get(&entity) {
            return Some((None, start.root));
        }
        if let Some(source) = self.world.get::<NodePoseSource>(entity) {
            if let Some(start) = self.roots.get(&source.rig) {
                let rig = self.world.get::<ModelRig>(source.rig)?;
                if rig.nodes.get(source.index) != Some(&entity) {
                    return None;
                }
                let playback = self.world.get::<Playback>(source.rig)?;
                let overrides = self.world.get::<PoseOverrides>(source.rig)?;
                let mut selection = playback.selection().ok()?;
                selection.clip_time = Time::seconds(0.).ok()?;
                selection.shared_time = selection.clip_time;
                let sample = rig.model.animation.sample(selection).ok()?;
                let pose = PoseContext {
                    nodes: &rig.model.nodes,
                    transition: rig.transition.view_at(playback.transition_clock),
                    overrides: &overrides.0,
                    camera: start
                        .camera
                        .filter(|_| start.basis.determinant().abs() > 1e-12)
                        .map(|camera| Affine3A::from_mat4(start.basis.inverse() * camera)),
                }
                .at_offset(-playback.shared_time);
                let (matrix, _) = pose.sample_node(
                    source.index,
                    &sample.at_times(selection.clip_time, selection.shared_time),
                    &mut self.scratch.borrow_mut(),
                );
                return Some((None, start.basis * matrix));
            }
        }
        WorldFrames(self.world).edge(entity)
    }
}
pub(super) fn actor_parent(world: &World, entity: Entity) -> Option<Entity> {
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
    pipeline.tree_roots.clear();
    pipeline.tree_roots.extend(pipeline.trees.iter(world));
    for &root in &pipeline.tree_roots {
        prepare_tree(world, root);
    }
    pipeline.entities.clear();
    pipeline.entities.extend(pipeline.actors.iter(world));
    if pipeline.entities.is_empty() {
        for &root in &pipeline.tree_roots {
            publish_tree(world, root);
        }
        return;
    }
    let evaluation_tick = pipeline.evaluation.meta().get_last_run();
    let simulation_tick = pipeline.simulation.meta().get_last_run();
    let factory_tick = pipeline.factory.meta().get_last_run();
    let attachments_tick = pipeline.attachment_clocks.meta().get_last_run();
    pipeline.affines.begin_frame();
    pipeline.birth_roots.clear();
    pipeline.birth_affines.begin_frame();
    pipeline.visited.clear();
    pipeline.fresh.clear();
    pipeline.fresh_order.clear();
    pipeline.attachment_starts.clear();
    if let Some(mut births) = world.get_resource_mut::<ModelParticleBirths>() {
        for entity in births.0.drain(..) {
            if pipeline.fresh.insert(entity) {
                pipeline.fresh_order.push(entity);
            }
        }
    }
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
    pipeline
        .pending
        .extend(pipeline.fresh_order.iter().copied());
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
        let new_actor = pipeline.fresh.contains(&entity) || world.get::<ModelRig>(entity).is_none();
        if world.get::<ModelInstance>(entity).is_some()
            || world.get::<PreparedModelRequest>(entity).is_some()
        {
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
        if world.get::<ModelRig>(entity).is_some()
            && world.get::<ModelInstance>(entity).is_none()
            && world.get::<PreparedModelRequest>(entity).is_none()
        {
            if let Some(mut lease) = world.get_mut::<ModelActorLease>(entity) {
                lease.ready();
            }
        }
        if (new_actor || world.get::<ModelParticleStart>(entity).is_some())
            && initialize_attachment_start(world, pipeline, entity)
        {
            pipeline.attachment_starts.insert(entity);
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
        process_actor(
            world,
            pipeline,
            entity,
            evaluation_tick,
            simulation_tick,
            new_actor,
        );
        if let Some(mut births) = world.get_resource_mut::<ModelParticleBirths>() {
            for entity in births.0.drain(..) {
                pipeline.fresh.insert(entity);
                pipeline.pending.push_back(entity);
            }
        }
    }
    pipeline.tree_roots.clear();
    pipeline.tree_roots.extend(pipeline.trees.iter(world));
    for &root in &pipeline.tree_roots {
        publish_tree(world, root);
    }
    let tick = world.read_change_tick();
    pipeline.evaluation.meta_mut().set_last_run(tick);
    pipeline.simulation.meta_mut().set_last_run(tick);
    pipeline.factory.meta_mut().set_last_run(tick);
    pipeline.attachment_clocks.meta_mut().set_last_run(tick);
}
/// Independent attachments advance their own clock once over the accepted
/// particle-age interval. A mapped mount does not scale that interval again.
fn initialize_attachment_start(
    world: &mut World,
    pipeline: &mut ModelPipeline,
    entity: Entity,
) -> bool {
    let Some(attachment) = world.get::<ModelAttachment>(entity) else {
        return false;
    };
    let Some(parent) = actor_parent(world, entity) else {
        return false;
    };
    let Some(clock) = world.get::<Playback>(parent) else {
        return false;
    };
    let source_time = clock.shared_time;
    if world.get::<ModelParticleStart>(entity).is_none() {
        let Some(start) = pipeline.birth_roots.get(&parent) else {
            return false;
        };
        let Some(rig) = world.get::<ModelRig>(parent) else {
            return false;
        };
        let Ok(mut selection) = clock.selection() else {
            return false;
        };
        if let Some(socket) = attachment.definition.socket() {
            let Ok(current) = rig.model.animation.sample(selection) else {
                return false;
            };
            if current.track(&socket.visibility).is_some()
                || attachment.definition.visibility(&current) <= 0.
            {
                return false;
            }
            selection.clip_time = Time::seconds(0.).unwrap();
            selection.shared_time = selection.clip_time;
            let Ok(initial) = rig.model.animation.sample(selection) else {
                return false;
            };
            if attachment.definition.visibility(&initial) <= 0. {
                return false;
            }
        }
        let interval = start.elapsed;
        let Some(basis) = pipeline.birth_affines.resolve(
            entity,
            &BirthFrames {
                world,
                roots: &pipeline.birth_roots,
                scratch: &pipeline.birth_scratch,
            },
        ) else {
            return false;
        };
        world.entity_mut(entity).insert(ModelParticleStart {
            basis,
            source: parent,
            observed_time: Some(source_time),
            interval,
        });
    }
    let Some(start) = observe_model_start(world, entity, parent, source_time, 0.) else {
        return false;
    };
    let Some(mut clock) = world.get_mut::<Playback>(entity) else {
        return false;
    };
    if seed_independent_clock(&mut clock, start.interval) {
        true
    } else {
        world.entity_mut(entity).remove::<ModelParticleStart>();
        false
    }
}
fn seed_independent_clock(clock: &mut Playback, elapsed: f64) -> bool {
    if clock.clip_time != 0. || clock.shared_time != 0. || !clock.speed.is_finite() {
        return false;
    }
    let delta = if clock.paused {
        0.
    } else {
        elapsed * clock.speed
    };
    if !delta.is_finite() || delta < 0. {
        return false;
    }
    clock.clip_time = delta;
    clock.shared_time = delta;
    clock.transition_clock += delta;
    true
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
    new_actor: bool,
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
    let actor_frame = actor_frame(world, entity, owned);
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
    let driver = world.get::<ModelParticleDriver>(entity).is_some();
    let initial_root = world
        .get::<ModelParticleStart>(entity)
        .map(|start| start.basis)
        .or_else(|| {
            (new_actor
                && (world.get::<ModelAttachment>(entity).is_none()
                    || pipeline.attachment_starts.contains(&entity))
                && actor_parent(world, entity)
                    .is_some_and(|parent| pipeline.birth_roots.contains_key(&parent)))
            .then(|| {
                pipeline.birth_affines.resolve(
                    entity,
                    &BirthFrames {
                        world,
                        roots: &pipeline.birth_roots,
                        scratch: &pipeline.birth_scratch,
                    },
                )
            })
            .flatten()
        });
    let initial_basis = initial_root.and_then(|root| {
        world
            .get::<Transform>(basis_entity)
            .map(|basis| root * basis.to_matrix())
    });
    let Ok(selection) = playback.selection() else {
        return;
    };
    let root_visible = current_hierarchy_visible(world, entity);
    pipeline.simulation.meta_mut().set_last_run(simulation_tick);
    let mut prepared = None;
    let mut seeded = false;
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
                RuntimeParticles::set_actor_frame(particles, actor_frame.clone());
                if let Some(birth_basis) = initial_basis {
                    let mut initial_selection = selection;
                    initial_selection.clip_time = Time::seconds(0.).unwrap();
                    initial_selection.shared_time = initial_selection.clip_time;
                    if let Ok(initial_sample) = animation.sample(initial_selection) {
                        model.particles.seed_owned(
                            particles,
                            EffectFrame {
                                root_visible,
                                traversal: Default::default(),
                                change: Default::default(),
                                time: 0.,
                                selection: initial_selection,
                                sample: &initial_sample,
                                pose: Some(
                                    PoseContext {
                                        nodes: &model.nodes,
                                        transition: transition.view_at(playback.transition_clock),
                                        overrides: &overrides.0,
                                        camera: camera
                                            .filter(|_| birth_basis.determinant().abs() > 1e-12)
                                            .map(|camera| {
                                                Affine3A::from_mat4(birth_basis.inverse() * camera)
                                            }),
                                    }
                                    .at_offset(-playback.shared_time),
                                ),
                                node_poses: poses,
                                node_visible: visible,
                                basis_world: birth_basis,
                            },
                            &mut effects,
                        );
                        seeded = true;
                    }
                }
                if driver {
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
                        actor_frame.is_some(),
                        actor_frame.as_ref(),
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
    }
    pipeline.simulation.apply(world);
    if seeded {
        let captured_interval = world
            .get::<ModelParticleStart>(entity)
            .map(|start| start.interval);
        world.entity_mut(entity).remove::<ModelParticleStart>();
        let elapsed = captured_interval.unwrap_or_else(|| {
            actor_parent(world, entity)
                .and_then(|parent| pipeline.birth_roots.get(&parent))
                .map_or(playback.shared_time, |parent| parent.elapsed)
        });
        pipeline.birth_roots.insert(
            entity,
            BirthActor {
                root: initial_root.unwrap(),
                basis: initial_basis.unwrap(),
                camera,
                elapsed,
            },
        );
        pipeline.birth_affines.begin_frame();
    }
    if let Some(mut driver) = world.get_mut::<ModelParticleDriver>(entity) {
        driver.prepared = prepared;
    }
    // Model projections changed detached roots without ordinary ChildOf edges.
    pipeline.affines.begin_frame();
}

#[cfg(test)]
#[path = "model_driver_tests.rs"]
mod tests;
