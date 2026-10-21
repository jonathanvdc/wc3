//! External-model particles with frozen world-space births and independent clocks.
use super::super::child_coordinates::ChildCoordinates;
use super::super::coordinates::basis;
use super::super::{ModelAsset, ModelInstance, Playback};
use super::atlas_sprites::retimed_selection;
use super::emission::{EffectChange, EmissionClock};
use super::frame::EffectStart;
use super::model_budget::ActorFrame;
use super::model_children::{ModelParticleChild, ModelParticleMotion, ModelParticleStart};
use super::model_ownership::spawn_owned_model;
use super::EffectFrame;
use crate::child_models::{ChildModelBinding, ChildModelKey};
use crate::diagnostics::{RenderDiagnosticKind, RenderIssue};
use crate::effects::simulation::{BirthSchedule, EmissionPhase, EmissionRounding, EmitterRng};
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::TAU;
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
    longitude: Property<f32>,
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
    ordinal: u64,
    scratch: Vec<usize>,
    previous_node: Mat4,
    omitted: u64,
    skipped_seconds: f64,
}
impl State {
    fn reset_emission(&mut self) {
        self.phase.reset();
        self.ordinal = 0;
    }
    fn schedule(&mut self, rate: f64, dt: f64, limit: usize) -> (BirthSchedule, u64) {
        let schedule = self
            .phase
            .continuous(rate, dt, limit, EmissionRounding::Model);
        let first = self.ordinal;
        self.ordinal = self.ordinal.wrapping_add(schedule.authored_count as u64);
        self.omitted = self
            .omitted
            .saturating_add((schedule.authored_count - schedule.count).max(0.) as u64);
        (schedule, first)
    }
}
#[derive(Default)]
pub(crate) struct ModelParticleInstances {
    states: Vec<State>,
    owner: Option<Entity>,
    previous_time: Option<f64>,
    previous_basis: Mat4,
    previous_clip: Option<ClipId>,
}

impl RuntimeModelParticles {
    pub(super) fn seed_owned(
        &self,
        instances: &mut ModelParticleInstances,
        start: &EffectStart<'_, '_, '_>,
    ) {
        if instances.previous_time.is_some() {
            return;
        }
        instances.previous_time = Some(0.);
        instances.previous_clip = start.frame.selection.clip;
        instances.previous_basis = start.frame.basis_world;
        for (emitter, state) in self.emitters.iter().zip(&mut instances.states) {
            state.previous_node = start.feature(emitter.anchor, &mut state.scratch);
        }
    }

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
            diagnostic(RenderDiagnosticKind::Unverified, "Model-particle latitude and longitude bound raw-radian polar and azimuth angles; exact native game angular distributions remain unverified");
            diagnostic(RenderDiagnosticKind::Unverified, "Model-particle yaw, pitch, and spin use deterministic occurrence-indexed random packets; native game random sequences remain unverified");
            diagnostic(RenderDiagnosticKind::Budget, "External-model particles retain at most 1024 children and 1024 births per update; rates use the shared bounded 60 Hz emission clock");
            diagnostic(RenderDiagnosticKind::Budget, "Nested model particles and their static descendants share ModelParticleLimits on the outer application root; omitted admissions and emission intervals are not queued");
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
                longitude: details.longitude,
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
                    ordinal: 0,
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
        self.update_limited(
            instances, commands, root, frame, assets, statistics, true, None,
        );
    }
    #[allow(clippy::too_many_arguments)]
    pub fn update_limited(
        &self,
        instances: &mut ModelParticleInstances,
        commands: &mut Commands,
        root: Entity,
        frame: &EffectFrame<'_, '_, '_>,
        assets: &Assets<ModelAsset>,
        statistics: &mut Query<&mut ModelEmitter>,
        allow_births: bool,
        actor: Option<&ActorFrame>,
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
                state.reset_emission();
            }
            state.children.retain(|(entity, birth, lifetime)| {
                if frame.time - birth >= *lifetime {
                    commands.entity(*entity).try_despawn();
                    false
                } else {
                    true
                }
            });
            let asset = assets.get(&emitter.model);
            let start = if reset { frame.time } else { previous };
            let mut clock = EmissionClock::new(start, frame.time);
            let mut budget = CAPACITY;
            let mut checked_end = start;
            for step in clock.by_ref() {
                if actor.is_some_and(|actor| !actor.step()) {
                    break;
                }
                checked_end = step.end;
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
                let local_limit = budget;
                let limit = if allow_births && asset.is_some() {
                    actor.map_or(local_limit, |actor| local_limit.min(actor.remaining()))
                } else {
                    local_limit
                };
                let (schedule, first_ordinal) = state.schedule(rate, step.end - step.start, limit);
                let local_count = (schedule.authored_count as usize).min(local_limit);
                if let Some(actor) = actor.filter(|_| allow_births && asset.is_some()) {
                    actor
                        .budget
                        .omit_work(local_count.saturating_sub(schedule.count as usize) as u64);
                }
                budget = budget.saturating_sub(local_count);
                if !allow_births {
                    state.omitted = state.omitted.saturating_add(schedule.count as u64);
                    continue;
                }
                let Some(asset) = asset else {
                    continue;
                };
                for index in 0..schedule.count as usize {
                    if state.children.len() >= CAPACITY {
                        state.omitted = state.omitted.saturating_add(1);
                        continue;
                    }
                    let lease = match actor.map(ActorFrame::reserve).transpose() {
                        Ok(lease) => lease,
                        Err(reason) => {
                            let omitted = schedule.count as usize - index;
                            state.omitted = state.omitted.saturating_add(omitted as u64);
                            if let Some(actor) = actor {
                                actor.budget.omit(reason, omitted.saturating_sub(1) as u64);
                            }
                            break;
                        }
                    };
                    let time = step.start + schedule.offset(index);
                    let Some((clip, shared)) = retimed_selection(frame, time) else {
                        continue;
                    };
                    let sample = frame.sample.at_times(clip, shared);
                    let lifetime = f64::from(sample.animated(&emitter.lifetime));
                    let speed = sample.animated(&emitter.speed);
                    let latitude = sample.animated(&emitter.latitude);
                    let longitude = sample.animated(&emitter.longitude);
                    if !lifetime.is_finite()
                        || lifetime <= frame.time - time
                        || !speed.is_finite()
                        || !latitude.is_finite()
                        || !longitude.is_finite()
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
                    let mut rng = EmitterRng::at_occurrence(
                        emitter.anchor as u32,
                        first_ordinal.wrapping_add(index as u64),
                    );
                    let yaw = rng.signed() * longitude;
                    let pitch = rng.signed() * latitude;
                    let direction =
                        Quat::from_rotation_z(yaw) * Quat::from_rotation_y(pitch) * Vec3::Z;
                    let velocity = (rotation * direction) * speed * scale;
                    let acceleration =
                        -self.gravity_axis * sample.animated(&emitter.gravity) * scale.z;
                    let age = frame.time - time;
                    let (_, source_rotation, _) = frame.basis_world.to_scale_rotation_translation();
                    let spin = source_rotation * Quat::from_rotation_z(rng.random() * TAU);
                    let entity = spawn_owned_model(
                        commands,
                        instances.owner.unwrap(),
                        (
                            ModelInstance(emitter.model.clone()),
                            emitter.placement,
                            ModelParticleChild {
                                root,
                                birth: time,
                                lifetime,
                                visible: true,
                                first_clip_pending: false,
                            },
                            ModelParticleStart {
                                basis: Mat4::from_scale_rotation_translation(scale, spin, origin),
                                source: root,
                                observed_time: None,
                                interval: age,
                            },
                            ModelParticleMotion {
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
                        ),
                        lease,
                    );
                    state.children.push((entity, time, lifetime));
                }
            }
            let skipped = (frame.time - checked_end).max(0.);
            state.skipped_seconds += skipped;
            if let Some(actor) = actor {
                actor.budget.skip_seconds(skipped);
            }
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

#[cfg(test)]
#[path = "model_particles_tests.rs"]
mod tests;
