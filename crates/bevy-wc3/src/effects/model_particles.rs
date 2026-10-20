//! External-model particles with frozen world-space births and independent clocks.
use super::super::child_coordinates::ChildCoordinates;
use super::super::coordinates::basis;
use super::super::{ModelAsset, ModelInstance, ModelRig, Playback};
use super::atlas_sprites::retimed_selection;
use super::emission::{EffectChange, EmissionClock};
use super::EffectFrame;
use crate::child_models::{ChildModelBinding, ChildModelKey};
use crate::diagnostics::{RenderDiagnosticKind, RenderIssue};
use crate::effects::simulation::{EmissionPhase, EmissionRounding, EmitterRng};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::TAU;
#[cfg(test)]
use wc3::model::ModelMetadata;
use wc3::model::{
    ClipId, FeatureId, MdlxParticle1Resource, Model, NodeFeature, NodeId, ParticleEmitter, Property,
};

const CAPACITY: usize = 1024;

/// Source identity and bounded population of an external-model emitter.
#[derive(Component, Clone, Debug)]
pub struct ModelEmitter {
    /// Semantic feature identity.
    pub feature: FeatureId,
    /// Hierarchy anchor.
    pub node: NodeId,
    /// Living independently animated child models.
    pub alive: usize,
    /// Births omitted by the explicit per-update or population budget.
    pub omitted: u64,
    /// Emission time omitted by the bounded fixed clock.
    pub skipped_seconds: f64,
}
#[derive(Clone)]
struct Emitter {
    feature: FeatureId,
    node: NodeId,
    anchor: usize,
    model: Handle<ModelAsset>,
    placement: ChildCoordinates,
    rate: Property<f32>,
    activation: Property<f32>,
    speed: Property<f32>,
    gravity: Property<f32>,
    lifetime: Property<f32>,
    latitude: Property<f32>,
}
#[derive(Clone, Default)]
pub(crate) struct RuntimeModelParticles {
    emitters: Vec<Emitter>,
    clips: HashMap<ClipId, (f64, bool)>,
    gravity_axis: Vec3,
    diagnostics: Vec<(FeatureId, RenderIssue)>,
}
struct State {
    entity: Entity,
    children: Vec<(Entity, f64, f64)>,
    phase: EmissionPhase,
    rng: EmitterRng,
    scratch: Vec<usize>,
    previous_node: Mat4,
    omitted: u64,
    skipped_seconds: f64,
}
#[derive(Default)]
pub(crate) struct ModelParticleInstances {
    states: Vec<State>,
    owner: Option<Entity>,
    previous_time: Option<f64>,
    previous_basis: Mat4,
    previous_clip: Option<ClipId>,
}

/// Independently clocked child whose world-space trajectory belongs to an emitter.
#[derive(Component)]
pub(crate) struct ModelParticleChild {
    root: Entity,
    owner: Entity,
    birth: f64,
    lifetime: f64,
    origin: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    rotation: Quat,
    scale: Vec3,
}
impl ModelParticleChild {
    pub(crate) fn parent_model(&self) -> Entity {
        self.root
    }
}

impl RuntimeModelParticles {
    pub(super) fn append_remapped(
        &mut self,
        source: &Self,
        indices: &[usize],
        nodes: &HashMap<NodeId, NodeId>,
        clips: &HashMap<ClipId, ClipId>,
    ) {
        self.emitters
            .extend(source.emitters.iter().cloned().map(|mut value| {
                value.node = nodes[&value.node];
                value.anchor = indices[value.anchor];

                value
            }));
        self.diagnostics.extend(source.diagnostics.iter().cloned());
        for (&host, &part) in clips {
            if let Some(&policy) = source.clips.get(&part) {
                self.clips.entry(host).or_insert(policy);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.emitters.is_empty()
    }
    pub fn diagnostics(&self) -> &[(FeatureId, RenderIssue)] {
        &self.diagnostics
    }
    pub fn compile(
        model: &Model<'_>,
        nodes: &HashMap<NodeId, usize>,
        children: &HashMap<ChildModelKey, ChildModelBinding>,
    ) -> Self {
        let mut result = Self {
            clips: model
                .clips()
                .iter()
                .map(|(id, c)| (id, (c.duration().as_seconds(), c.is_looping())))
                .collect(),
            gravity_axis: basis(model).map_or(Vec3::Z, |basis| {
                basis
                    .to_matrix()
                    .transform_vector3(Vec3::Z)
                    .normalize_or_zero()
            }),
            ..Default::default()
        };
        for (feature, attached) in model.features() {
            let NodeFeature::Particle(ParticleEmitter::MdlxEmitter1(definition)) =
                &attached.payload
            else {
                continue;
            };
            let details = &definition.details;
            let mut diagnostic = |kind, message: &str| {
                result
                    .diagnostics
                    .push((feature, RenderIssue::new(kind, message)))
            };
            if !matches!(details.resource, MdlxParticle1Resource::Model(_)) || details.text_resource
            {
                diagnostic(RenderDiagnosticKind::Unsupported, "Model-particle image/text resources require a separate sprite interpretation and are not instantiated");
                continue;
            }
            let Some(child) = children.get(&ChildModelKey::particle_geometry(feature)) else {
                diagnostic(
                    RenderDiagnosticKind::Resource,
                    "Model-particle child resource is unresolved",
                );
                continue;
            };
            let (rate, speed, lifetime) = (
                definition.emission.rate,
                definition.motion.speed,
                definition.lifetime.duration,
            );
            diagnostic(RenderDiagnosticKind::Unsupported, "Model-particle longitude is retained but unused; latitude uses the Warcraft III radian convention");
            diagnostic(RenderDiagnosticKind::Budget, "External-model particles retain at most 1024 children and 1024 births per update; rates use the shared bounded 60 Hz emission clock");
            result.emitters.push(Emitter {
                feature,
                node: attached.node,
                anchor: nodes[&attached.node],
                model: child.model.clone(),
                placement: ChildCoordinates::from_model(model, child.coordinates),
                rate,
                activation: definition.emission.activation,
                speed,
                gravity: definition.motion.gravity,
                lifetime,
                latitude: details.latitude,
            });
        }
        result
    }
    pub fn spawn(&self, commands: &mut Commands, owner: Entity) -> ModelParticleInstances {
        ModelParticleInstances {
            owner: Some(owner),
            states: self
                .emitters
                .iter()
                .map(|emitter| State {
                    entity: commands
                        .spawn((
                            ModelEmitter {
                                feature: emitter.feature,
                                node: emitter.node,
                                alive: 0,
                                omitted: 0,
                                skipped_seconds: 0.,
                            },
                            Transform::default(),
                            Visibility::default(),
                            ChildOf(owner),
                        ))
                        .id(),
                    children: Vec::new(),
                    phase: Default::default(),
                    rng: EmitterRng::new(emitter.anchor as u32),
                    scratch: Vec::new(),
                    previous_node: Mat4::IDENTITY,
                    omitted: 0,
                    skipped_seconds: 0.,
                })
                .collect(),
            ..Default::default()
        }
    }
    pub fn update(
        &self,
        instances: &mut ModelParticleInstances,
        commands: &mut Commands,
        root: Entity,
        frame: &EffectFrame<'_, '_, '_>,
        assets: &Assets<ModelAsset>,
        statistics: &mut Query<&mut ModelEmitter>,
    ) {
        let previous = instances.previous_time.unwrap_or(frame.time);
        let reset = frame.time < previous
            || instances.previous_clip != frame.selection.clip
            || frame.change == EffectChange::Reset;
        for (emitter, state) in self.emitters.iter().zip(&mut instances.states) {
            let current_node = frame.basis_world * frame.feature_matrix(emitter.anchor);
            if reset {
                for (entity, _, _) in state.children.drain(..) {
                    commands.entity(entity).try_despawn();
                }
                state.phase.reset();
            }
            state.children.retain(|(entity, birth, lifetime)| {
                if frame.time - birth >= *lifetime {
                    commands.entity(*entity).try_despawn();
                    false
                } else {
                    true
                }
            });
            let Some(asset) = assets.get(&emitter.model) else {
                state.previous_node = current_node;
                continue;
            };
            let start = if reset { frame.time } else { previous };
            let mut clock = EmissionClock::new(start, frame.time);
            let mut budget = CAPACITY;
            for step in clock.by_ref() {
                let Some((clip, shared)) = retimed_selection(frame, step.sample_time) else {
                    continue;
                };
                let sample = frame.sample.at_times(clip, shared);
                let ended = frame
                    .selection
                    .clip
                    .and_then(|id| self.clips.get(&id))
                    .is_some_and(|(duration, looping)| !*looping && clip.as_seconds() >= *duration);
                let active = sample.animated(&emitter.activation) > 0.1;
                let rate = if active
                    && !ended
                    && frame.root_visible
                    && frame
                        .node_visible
                        .get(emitter.anchor)
                        .copied()
                        .unwrap_or(true)
                {
                    f64::from(sample.animated(&emitter.rate).max(0.))
                } else {
                    0.
                };
                let schedule = state.phase.continuous(
                    rate,
                    step.end - step.start,
                    budget,
                    EmissionRounding::Model,
                );
                budget = budget.saturating_sub(schedule.count as usize);
                for index in 0..schedule.count as usize {
                    if state.children.len() >= CAPACITY {
                        state.omitted = state.omitted.saturating_add(1);
                        continue;
                    }
                    let time = step.start + schedule.offset(index);
                    let Some((clip, shared)) = retimed_selection(frame, time) else {
                        continue;
                    };
                    let sample = frame.sample.at_times(clip, shared);
                    let lifetime = f64::from(sample.animated(&emitter.lifetime));
                    let speed = sample.animated(&emitter.speed);
                    let latitude = sample.animated(&emitter.latitude);
                    if !lifetime.is_finite()
                        || lifetime <= frame.time - time
                        || !speed.is_finite()
                        || !latitude.is_finite()
                    {
                        continue;
                    }
                    let alpha = ((time - previous) / (frame.time - previous)).clamp(0., 1.) as f32;
                    let node = if let Some(pose) = frame.pose {
                        let (authored, visible) = pose
                            .at_offset(time - frame.time)
                            .sample_feature_node(emitter.anchor, &sample, &mut state.scratch);
                        if !visible {
                            continue;
                        }
                        interpolated(instances.previous_basis, frame.basis_world, alpha) * authored
                    } else {
                        interpolated(state.previous_node, current_node, alpha)
                    };
                    let (scale, rotation, origin) = node.to_scale_rotation_translation();
                    let yaw = state.rng.random() * TAU;
                    let pitch = state.rng.signed() * latitude;
                    let direction =
                        Quat::from_rotation_z(yaw) * Quat::from_rotation_y(pitch) * Vec3::Z;
                    let velocity = (rotation * direction) * speed * scale;
                    let acceleration =
                        -self.gravity_axis * sample.animated(&emitter.gravity) * scale.z;
                    let age = frame.time - time;
                    let (_, source_rotation, _) = frame.basis_world.to_scale_rotation_translation();
                    let spin = source_rotation * Quat::from_rotation_z(state.rng.random() * TAU);
                    let entity = commands
                        .spawn((
                            ModelInstance(emitter.model.clone()),
                            emitter.placement,
                            ModelParticleChild {
                                root,
                                owner: instances.owner.unwrap(),
                                birth: time,
                                lifetime,
                                origin,
                                velocity,
                                acceleration,
                                rotation: spin,
                                scale,
                            },
                            Playback {
                                clip: asset.compiled.clips.first().map(|(id, _)| *id),
                                clip_time: age,
                                shared_time: age,
                                paused: true,
                                ..Default::default()
                            },
                            Transform {
                                translation: origin
                                    + velocity * age as f32
                                    + acceleration * (0.5 * (age * age) as f32),
                                rotation: spin,
                                scale,
                            },
                            Visibility::default(),
                        ))
                        .id();
                    state.children.push((entity, time, lifetime));
                }
            }
            state.skipped_seconds += clock.omitted_seconds();
            state.previous_node = current_node;
            if let Ok(mut stats) = statistics.get_mut(state.entity) {
                if stats.alive != state.children.len() {
                    stats.alive = state.children.len();
                }
                if stats.omitted != state.omitted {
                    stats.omitted = state.omitted;
                }
                if stats.skipped_seconds != state.skipped_seconds {
                    stats.skipped_seconds = state.skipped_seconds;
                }
            }
        }
        instances.previous_time = Some(frame.time);
        instances.previous_clip = frame.selection.clip;
        instances.previous_basis = frame.basis_world;
    }
}
fn interpolated(a: Mat4, b: Mat4, t: f32) -> Mat4 {
    let (sa, ra, ta) = a.to_scale_rotation_translation();
    let (sb, rb, tb) = b.to_scale_rotation_translation();
    Mat4::from_scale_rotation_translation(sa.lerp(sb, t), ra.slerp(rb, t), ta.lerp(tb, t))
}

pub(crate) fn hierarchy_visible(
    mut entity: Entity,
    hierarchy: &Query<(&Visibility, Option<&ChildOf>)>,
) -> bool {
    loop {
        let Ok((visibility, parent)) = hierarchy.get(entity) else {
            return true;
        };
        match visibility {
            Visibility::Visible => return true,
            Visibility::Hidden => return false,
            Visibility::Inherited => {}
        }
        let Some(parent) = parent else {
            return true;
        };
        entity = parent.parent();
    }
}

type ParentPlaybackData = (&'static Playback, Option<&'static ModelRig>);
type ChildPlaybackData = (&'static mut Transform, &'static mut Playback);
type PlaybackQueries = (
    Query<'static, 'static, ParentPlaybackData>,
    Query<'static, 'static, ChildPlaybackData, With<ModelParticleChild>>,
);

/// Keep delayed child loads on particle age, and clean up detached owned children.
pub(crate) fn synchronize_model_particles(
    mut commands: Commands,
    children: Query<(Entity, &ModelParticleChild, &Visibility)>,
    hierarchy: Query<(&Visibility, Option<&ChildOf>)>,
    entities: Query<()>,
    mut playback: ParamSet<PlaybackQueries>,
) {
    for (entity, particle, visibility) in &children {
        let parent_time = playback
            .p0()
            .get(particle.root)
            .map(|(playback, instance)| {
                instance.map_or(playback.shared_time, |instance| {
                    instance.effect_time(playback)
                })
            })
            .ok();
        let Some(parent_time) = parent_time.filter(|_| entities.contains(particle.owner)) else {
            commands.entity(entity).try_despawn();
            continue;
        };
        let visible = hierarchy_visible(particle.root, &hierarchy);
        let desired = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != desired {
            commands.entity(entity).insert(desired);
        }
        let age = parent_time - particle.birth;
        if age < 0. || age >= particle.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }
        if let Ok((mut transform, mut clock)) = playback.p1().get_mut(entity) {
            let age32 = age as f32;
            let translation = particle.origin
                + particle.velocity * age32
                + particle.acceleration * (0.5 * age32 * age32);
            if transform.translation != translation {
                transform.translation = translation;
            }
            if transform.rotation != particle.rotation {
                transform.rotation = particle.rotation;
            }
            if transform.scale != particle.scale {
                transform.scale = particle.scale;
            }
            if clock.clip_time != age {
                clock.clip_time = age;
                clock.shared_time = age;
            }
            if !clock.paused {
                clock.paused = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime::synchronize_model_cameras;
    use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

    use super::*;
    fn child(app: &mut App, root: Entity, owner: Entity) -> Entity {
        app.world_mut()
            .spawn((
                ModelParticleChild {
                    root,
                    owner,
                    birth: 1.,
                    lifetime: 10.,
                    origin: Vec3::new(3., 4., 5.),
                    velocity: Vec3::X * 2.,
                    acceleration: Vec3::NEG_Z * 4.,
                    rotation: Quat::from_rotation_y(0.7),
                    scale: Vec3::splat(0.5),
                },
                Transform::default(),
                Visibility::default(),
                Playback::default(),
            ))
            .id()
    }
    #[test]
    fn delayed_children_follow_particle_age_and_frozen_world_trajectory() {
        let mut slot_values = SlotValues::default();
        let mut app = App::new();
        app.add_systems(
            Update,
            (synchronize_model_particles, synchronize_model_cameras).chain(),
        );
        let camera = app.world_mut().spawn_empty().id();
        let root = app
            .world_mut()
            .spawn((
                Playback {
                    shared_time: 3.,
                    driving_camera: Some(camera),
                    ..SlotConstruct::construct(&mut slot_values)
                },
                Transform::from_xyz(100., 200., 300.),
            ))
            .id();
        let owner = app.world_mut().spawn_empty().id();
        let child = child(&mut app, root, owner);
        app.update();
        assert_eq!(
            app.world().get::<Transform>(child).unwrap().translation,
            Vec3::new(7., 4., -3.)
        );
        let playback = app.world().get::<Playback>(child).unwrap();
        assert_eq!((playback.clip_time, playback.shared_time), (2., 2.));
        assert!(playback.paused);
        assert_eq!(playback.driving_camera, Some(camera));
        app.world_mut()
            .get_mut::<Playback>(root)
            .unwrap()
            .driving_camera = None;
        app.world_mut()
            .get_mut::<Transform>(root)
            .unwrap()
            .translation = Vec3::splat(-500.);
        app.update();
        assert_eq!(
            app.world().get::<Playback>(child).unwrap().driving_camera,
            None
        );
        assert_eq!(
            app.world().get::<Transform>(child).unwrap().translation,
            Vec3::new(7., 4., -3.)
        );
    }
    #[test]
    fn detached_children_are_cleaned_up_with_the_renderer_owner() {
        let mut slot_values = SlotValues::default();
        let mut app = App::new();
        app.add_systems(Update, synchronize_model_particles);
        let root = app
            .world_mut()
            .spawn(Playback {
                shared_time: 2.,
                ..SlotConstruct::construct(&mut slot_values)
            })
            .id();
        let owner = app.world_mut().spawn_empty().id();
        let child = child(&mut app, root, owner);
        app.world_mut().despawn(owner);
        app.update();
        assert!(app.world().get_entity(child).is_err());
    }
    #[test]
    fn detached_particle_requests_cannot_reload_an_ancestor_model() {
        use super::super::super::instances::{spawn_loaded_models, PreparedModelCache};
        use super::super::super::{
            compile_model, ModelInstanceError, ModelMaterial, ModelRig, ModelSource,
        };
        use crate::image_replacements::ReplacementImages;
        use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
        let model = Model::<wc3::model::profiles::Universal>::empty(ModelMetadata::Mdlx(
            Default::default(),
        ))
        .unwrap();
        let mut app = App::new();
        app.init_resource::<Assets<ModelAsset>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<SkinnedMeshInverseBindposes>>()
            .init_resource::<Assets<ModelMaterial>>()
            .init_resource::<PreparedModelCache>()
            .init_resource::<ReplacementImages>()
            .init_resource::<Assets<Image>>()
            .add_systems(Update, spawn_loaded_models);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<ModelAsset>>()
            .add(ModelAsset {
                compiled: compile_model(&model, |_| None).unwrap(),
                images: HashMap::new(),
                unresolved_images: Vec::new(),
            });
        let root = app.world_mut().spawn(ModelSource(handle.clone())).id();
        let owner = app.world_mut().spawn_empty().id();
        let particle = child(&mut app, root, owner);
        app.world_mut()
            .entity_mut(particle)
            .insert(ModelInstance(handle));
        app.update();
        assert!(
            matches!(app.world().get::<ModelInstanceError>(particle), Some(ModelInstanceError::CyclicDependency { ancestor, .. }) if *ancestor == root)
        );
        assert!(app.world().get::<ModelRig>(particle).is_none());
    }
}
