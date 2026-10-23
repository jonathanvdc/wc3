//! Direct staged sprite simulation using the resident immutable GPU transport.
use super::emission::{EffectChange, EmissionClock};
use super::frame::EffectStart;
use super::model_budget::ActorFrame;
use super::{EffectFrame, SpriteEmitter};
use crate::diagnostics::{RenderDiagnosticKind, RenderIssue};
use crate::effects::blend::EffectBlend;
use crate::effects::render::{effect_quad, EffectGeometry};
use crate::effects::simulation::{EmissionPhase, EmissionRounding, EmitterRng, LiveRecords};
use crate::effects::sprites::{
    ParticleEmitterUniform, ParticleInstance, ParticleInstances as SpriteInstances,
    StagedSpriteParameters,
};
use crate::layers::ModelOwnedRenderEntity;
use crate::textures::{TextureImages, TextureRole};
use bevy::camera::visibility::NoFrustumCulling;
use bevy::math::{Quat, Vec3};
use bevy::prelude::*;
use std::array::from_fn;
use std::cell::Cell;
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8, PI};
use std::mem::take;
use std::sync::Arc;
use wc3::model::materials::state::BlendOperations;
use wc3::model::profiles::mdlx::Frames;
#[cfg(test)]
use wc3::model::ModelMetadata;
use wc3::model::{
    ClipId, FeatureId, Model, NodeFeature, NodeId, PackedArray, ParticleEmitter, Property,
    SampledTrack, Time as ModelTime, TimeSource,
};

enum BurstTrack<'sample, 'data> {
    Scalar(SampledTrack<'sample, 'data, f32>),
    Boolean(SampledTrack<'sample, 'data, bool>),
}
impl<'data> BurstTrack<'_, 'data> {
    fn clock(&self) -> (TimeSource, Option<ModelTime>, ModelTime) {
        match self {
            Self::Scalar(sample) => (sample.track.clock, sample.period, sample.origin),
            Self::Boolean(sample) => (sample.track.clock, sample.period, sample.origin),
        }
    }
    fn times(&self) -> &PackedArray<'data, ModelTime> {
        match self {
            Self::Scalar(sample) => sample.track.curve.times(),
            Self::Boolean(sample) => sample.track.curve.times(),
        }
    }
    fn value(&self, index: usize) -> f32 {
        match self {
            Self::Scalar(sample) => sample.track.curve.values().get(index).unwrap(),
            Self::Boolean(sample) => {
                if sample.track.curve.values().get(index).unwrap() {
                    1.
                } else {
                    0.
                }
            }
        }
    }
}

const CAPACITY: usize = 8192;
const MAX_BURST_OCCURRENCES: usize = 4096;
#[derive(Clone)]
struct AtlasEmitter {
    feature: FeatureId,
    node: NodeId,
    anchor: usize,
    rate: Property<f32>,
    animated_rate: bool,
    speed: Property<f32>,
    variation: Property<f32>,
    gravity: Property<f32>,
    lifetime: f32,
    width: Property<f32>,
    length: Property<f32>,
    latitude: Property<f32>,
    activation: Property<f32>,
    burst: bool,
    line: bool,
    frames: Frames,
    parameters: StagedSpriteParameters,
    texture: Option<Handle<Image>>,
    blend: EffectBlend,
    priority: u32,
    sort: bool,
}
#[derive(Clone, Default)]
pub(crate) struct RuntimeAtlasParticles {
    emitters: Vec<AtlasEmitter>,
    clips: HashMap<ClipId, (f64, bool)>,
    gravity_axis: Vec3,
    diagnostics: Vec<(FeatureId, RenderIssue)>,
}
struct Expiration {
    time: f64,
    lifetime: f32,
    stationary: bool,
}
struct State {
    entity: Entity,
    live: LiveRecords<ParticleInstance, Expiration>,
    dynamic_records: usize,
    indices: Arc<Vec<u32>>,
    phase: EmissionPhase,
    rng: EmitterRng,
    previous_node: Mat4,
    previous_basis: Mat4,
    birth_start: f64,
    pose_scratch: Vec<usize>,
    previous_raw: f64,
    previous_track: usize,
    previous_clip: Option<ClipId>,
    bursts: Vec<(f64, f32)>,
    include_origin: bool,
    dropped: u64,
    skipped_seconds: f64,
    seek_pending: bool,
}
impl State {
    fn reset_records(&mut self) {
        self.live = Default::default();
        self.indices = Arc::default();
        self.dynamic_records = 0;
    }
    fn retire(&mut self, time: f64) {
        let dynamic = &mut self.dynamic_records;
        self.live.retire(|birth| {
            let expired = time - birth.time >= f64::from(birth.lifetime);
            if expired && !birth.stationary {
                *dynamic -= 1;
            }
            expired
        });
    }
    fn push(&mut self, record: ParticleInstance, time: f64, lifetime: f32) {
        if self.live.len() >= CAPACITY {
            self.dropped = self.dropped.saturating_add(1);
            return;
        }
        let stationary = record.stationary();
        self.live.push(
            record,
            Expiration {
                time,
                lifetime,
                stationary,
            },
            CAPACITY,
        );
        self.dynamic_records += usize::from(!stationary);
    }
}
#[derive(Default)]
pub(crate) struct AtlasParticleInstances {
    states: Vec<State>,
    previous_time: Option<f64>,
}
impl RuntimeAtlasParticles {
    pub(super) fn seed_owned(
        &self,
        instances: &mut AtlasParticleInstances,
        start: &EffectStart<'_, '_, '_>,
    ) {
        if instances.previous_time.is_some() {
            return;
        }
        instances.previous_time = Some(0.);
        for (emitter, state) in self.emitters.iter().zip(&mut instances.states) {
            state.previous_node = start.feature(emitter.anchor, &mut state.pose_scratch);
            state.previous_basis = start.frame.basis_world;
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

    pub(super) fn is_empty(&self) -> bool {
        self.emitters.is_empty()
    }

    pub fn compile(
        model: &Model<'_>,
        nodes: &HashMap<NodeId, usize>,
        images: &TextureImages,
    ) -> Self {
        let mut result = Self {
            clips: model
                .clips()
                .iter()
                .map(|(id, c)| (id, (c.duration().as_seconds(), c.is_looping())))
                .collect(),
            ..Default::default()
        };
        let basis = super::super::coordinates::basis(model)
            .map(|basis| basis.to_matrix())
            .unwrap_or(Mat4::IDENTITY);
        result.gravity_axis = basis.transform_vector3(Vec3::Z);
        for (feature, value) in model.features() {
            let NodeFeature::Particle(ParticleEmitter::MdlxEmitter2(definition)) = &value.payload
            else {
                continue;
            };
            let details = &definition.details;
            let mut diagnostic = |kind, message: &str| {
                result
                    .diagnostics
                    .push((feature, RenderIssue::new(kind, message)))
            };
            let appearance = &definition.appearance;
            let (rate, speed, variation, lifetime) = (
                definition.emission.rate,
                definition.motion.speed,
                definition.motion.variation,
                definition.lifetime.duration,
            );
            let gravity = definition.motion.gravity;
            if matches!(details.rendering.frames, Frames::Unknown(_)) {
                diagnostic(
                    RenderDiagnosticKind::Unsupported,
                    "Unknown staged sprite head/tail selection is not rendered",
                );
                continue;
            }
            if definition.emission.trigger {
                diagnostic(RenderDiagnosticKind::Budget, "Keyed sprite emission processes at most 4096 crossed key occurrences per update; extreme clock jumps are bounded");
            }
            let blend = match details.rendering.blend {
                BlendOperations::Opaque => EffectBlend::Opaque,
                BlendOperations::Alpha => EffectBlend::Alpha,
                BlendOperations::AlphaKey(_) => EffectBlend::Cutout,
                BlendOperations::Additive => EffectBlend::Additive,
                BlendOperations::AdditiveAlpha(_) => EffectBlend::AdditiveAlpha,
                BlendOperations::Modulate => EffectBlend::Modulate,
                BlendOperations::Modulate2x => EffectBlend::Modulate2x,
                _ => {
                    diagnostic(
                        RenderDiagnosticKind::Approximation,
                        "Unknown sprite blend operation uses an explicit alpha-blend approximation",
                    );
                    EffectBlend::Alpha
                }
            };
            let texture = details.rendering.texture.and_then(|id| {
                images
                    .particle(feature, 0)
                    .for_role(id, TextureRole::Color)
                    .cloned()
            });
            if texture.is_none() {
                diagnostic(
                    RenderDiagnosticKind::Resource,
                    "Staged sprite image is unresolved; texture-provider integration is required",
                );
            }
            result.emitters.push(AtlasEmitter {
                feature,
                node: value.node,
                anchor: nodes[&value.node],
                rate,
                animated_rate: model.is_animated(rate),
                speed,
                variation,
                gravity,
                lifetime,
                width: details.shape.width,
                length: details.shape.length,
                latitude: details.shape.latitude,
                activation: definition.emission.activation,
                burst: definition.emission.trigger,
                line: details.shape.line_emitter,
                frames: details.rendering.frames,
                parameters: StagedSpriteParameters {
                    colors: from_fn(|i| {
                        [
                            appearance.colors[i][0],
                            appearance.colors[i][1],
                            appearance.colors[i][2],
                            f32::from(appearance.alpha[i]) / 255.,
                        ]
                    }),
                    scales: (appearance.scale).to_array(),
                    middle: appearance.middle_time,
                    intervals: appearance.uv_intervals,
                    atlas: details.rendering.atlas,
                    model_space: details.model_space,
                    xy_quad: details.rendering.xy_quad,
                    tail_length: details.rendering.tail_length,
                    unlit: details.rendering.unlit,
                    fog_enabled: !details.rendering.unfogged,
                    fog_blend: blend.fog_blend(),
                    plane_axes: [
                        basis.transform_vector3(Vec3::X).normalize_or_zero(),
                        basis.transform_vector3(Vec3::Y).normalize_or_zero(),
                    ],
                },
                texture,
                blend,
                priority: details.rendering.priority_plane,
                sort: details.rendering.sort_far_z,
            });
        }
        result
    }
    pub fn diagnostics(&self) -> &[(FeatureId, RenderIssue)] {
        &self.diagnostics
    }
    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        root: Entity,
    ) -> AtlasParticleInstances {
        let quad = (!self.emitters.is_empty()).then(|| effect_quad(meshes));
        AtlasParticleInstances {
            states: self
                .emitters
                .iter()
                .map(|emitter| {
                    let entity = commands
                        .spawn((
                            ModelOwnedRenderEntity,
                            EffectGeometry(quad.as_ref().unwrap().clone()),
                            SpriteInstances {
                                records: Default::default(),
                                live_indices: Default::default(),
                                uniform: Default::default(),
                                stationary: true,
                                texture: emitter.texture.clone(),
                                filter: emitter.blend,
                                priority_plane: emitter.priority,
                                sort_far: emitter.sort,
                            },
                            SpriteEmitter {
                                feature: emitter.feature,
                                node: emitter.node,
                                alive: 0,
                                dropped: 0,
                                skipped_seconds: 0.,
                            },
                            Transform::default(),
                            Visibility::default(),
                            NoFrustumCulling,
                            ChildOf(root),
                        ))
                        .id();
                    State {
                        entity,
                        live: Default::default(),
                        dynamic_records: 0,
                        indices: Arc::default(),
                        phase: Default::default(),
                        rng: EmitterRng::new(emitter.anchor as u32),
                        previous_node: Mat4::IDENTITY,
                        previous_basis: Mat4::IDENTITY,
                        birth_start: 0.,
                        pose_scratch: Vec::new(),
                        previous_raw: 0.,
                        previous_track: 0,
                        previous_clip: None,
                        bursts: Vec::new(),
                        include_origin: true,
                        dropped: 0,
                        skipped_seconds: 0.,
                        seek_pending: false,
                    }
                })
                .collect(),
            previous_time: None,
        }
    }
    pub fn update_bounded(
        &self,
        instances: &mut AtlasParticleInstances,
        frame: &EffectFrame<'_, '_, '_>,
        render: &mut Query<&mut SpriteInstances>,
        stats: &mut Query<&mut SpriteEmitter>,
        actor: Option<&ActorFrame>,
    ) {
        let previous = instances.previous_time.unwrap_or(frame.time);
        let reset = frame.time < previous || frame.change == EffectChange::Reset;
        let dt = (frame.time - previous).max(0.);
        for (emitter, state) in self.emitters.iter().zip(&mut instances.states) {
            let node = frame.basis_world * frame.feature_matrix(emitter.anchor);
            if reset || instances.previous_time.is_none() {
                state.reset_records();
                state.phase.reset();
                state.rng = EmitterRng::new(emitter.anchor as u32);
                state.previous_node = node;
                state.previous_basis = frame.basis_world;
                state.include_origin = true;
                state.previous_track = 0;
            }
            state.seek_pending |= frame.change == EffectChange::Seek;
            state.birth_start = previous;
            state.retire(frame.time);
            let enabled = frame.sample.animated(&emitter.activation) > 0.
                && frame
                    .node_visible
                    .get(emitter.anchor)
                    .copied()
                    .unwrap_or(true);
            let ended = frame
                .selection
                .clip
                .and_then(|id| self.clips.get(&id))
                .is_some_and(|(duration, looping)| {
                    !*looping && frame.selection.clip_time.as_seconds() >= *duration
                });
            if dt > 0. {
                if emitter.burst {
                    self.bursts(
                        emitter,
                        state,
                        frame,
                        node,
                        dt,
                        frame.pose.is_some() || (enabled && !ended),
                        actor,
                    );
                } else if frame.pose.is_some() || (enabled && !ended) {
                    let mut clock = EmissionClock::new(previous, frame.time);
                    let mut remaining = CAPACITY;
                    let mut checked_end = previous;
                    for step in clock.by_ref() {
                        if actor.is_some_and(|actor| !actor.step()) {
                            break;
                        }
                        checked_end = step.end;
                        let Some((clip, shared)) = retimed_selection(frame, step.sample_time)
                        else {
                            continue;
                        };
                        let sample = frame.sample.at_times(clip, shared);
                        let active = sample.animated(&emitter.activation) > 0.;
                        let rate = if active {
                            f64::from(sample.animated(&emitter.rate).max(0.))
                        } else {
                            0.
                        };
                        let schedule = state.phase.continuous(
                            rate,
                            step.end - step.start,
                            remaining,
                            EmissionRounding::Quad,
                        );
                        remaining = remaining.saturating_sub(schedule.count as usize);
                        for index in 0..schedule.count as usize {
                            let time = step.start + schedule.offset(index);
                            let transform = interpolated(
                                state.previous_node,
                                node,
                                ((time - previous) / dt) as f32,
                            );
                            self.birth(emitter, state, frame, time, transform);
                        }
                    }
                    let skipped = (frame.time - checked_end).max(0.);
                    state.skipped_seconds += skipped;
                    if let Some(actor) = actor {
                        actor.budget.skip_seconds(skipped);
                    }
                }
            }
            state.previous_node = node;
            state.previous_basis = frame.basis_world;
            if state.indices.len() != state.live.len()
                || (!state.indices.is_empty() && state.indices[0] != state.live.records.index(0))
            {
                state.indices = Arc::new(
                    (0..state.live.len())
                        .map(|i| state.live.records.index(i))
                        .collect(),
                );
            }
            if let Ok(mut data) = render.get_mut(state.entity) {
                data.records = state.live.records.clone();
                data.live_indices = state.indices.clone();
                data.stationary = state.dynamic_records == 0;
                data.uniform =
                    ParticleEmitterUniform::from_staged(&emitter.parameters, node, frame.time);
            }
            if let Ok(mut value) = stats.get_mut(state.entity) {
                if value.alive != state.live.len() {
                    value.alive = state.live.len();
                }
                if value.dropped != state.dropped {
                    value.dropped = state.dropped;
                }
                if value.skipped_seconds != state.skipped_seconds {
                    value.skipped_seconds = state.skipped_seconds;
                }
            }
        }
        instances.previous_time = Some(frame.time);
    }
    fn birth(
        &self,
        emitter: &AtlasEmitter,
        state: &mut State,
        frame: &EffectFrame<'_, '_, '_>,
        time: f64,
        node: Mat4,
    ) {
        let age = frame.time - time;
        let selection = retimed_selection(frame, time);
        let Some((clip, shared)) = selection else {
            return;
        };
        let sample = frame.sample.at_times(clip, shared);
        let enabled = sample.animated(&emitter.activation) > 0.;
        let ended = frame
            .selection
            .clip
            .and_then(|id| self.clips.get(&id))
            .is_some_and(|(duration, looping)| !*looping && clip.as_seconds() >= *duration);
        if !enabled || ended {
            return;
        }
        let node = if let Some(pose) = frame.pose {
            let (authored, visible) = pose.at_offset(time - frame.time).sample_feature_node(
                emitter.anchor,
                &sample,
                &mut state.pose_scratch,
            );
            if !visible {
                return;
            }
            let dt = frame.time - state.birth_start;
            let alpha = if dt > 0. {
                ((time - state.birth_start) / dt).clamp(0., 1.) as f32
            } else {
                1.
            };
            interpolated(state.previous_basis, frame.basis_world, alpha) * authored
        } else {
            node
        };
        let lifetime = emitter.lifetime;
        if !lifetime.is_finite() || lifetime <= 0. || age >= f64::from(lifetime) {
            return;
        }
        let width = sample.animated(&emitter.width).abs();
        let length = sample.animated(&emitter.length).abs();
        let local = Vec3::new(
            state.rng.signed() * width * 0.5,
            state.rng.signed() * length * 0.5,
            0.,
        );
        let latitude = sample.animated(&emitter.latitude).to_radians();
        let yaw = state.rng.signed() * latitude;
        let pitch = if emitter.line {
            0.
        } else {
            state.rng.signed() * latitude
        };
        let direction = Quat::from_rotation_z(FRAC_PI_2)
            * Quat::from_rotation_y(yaw)
            * Quat::from_rotation_x(pitch)
            * Vec3::Z;
        let speed = sample.animated(&emitter.speed)
            * (1. + state.rng.signed() * sample.animated(&emitter.variation));
        let gravity = sample.animated(&emitter.gravity);
        let (position, velocity, acceleration) = if emitter.parameters.model_space {
            (local, direction * speed, Vec3::NEG_Z * gravity)
        } else {
            (
                node.transform_point3(local),
                node.transform_vector3(direction * speed),
                -self.gravity_axis * gravity,
            )
        };
        let scale = Vec3::new(
            node.x_axis.truncate().length(),
            node.y_axis.truncate().length(),
            node.z_axis.truncate().length(),
        );
        let facing = if emitter.parameters.model_space {
            velocity.y.atan2(velocity.x)
        } else {
            velocity
                .dot(emitter.parameters.plane_axes[1])
                .atan2(velocity.dot(emitter.parameters.plane_axes[0]))
        } - PI
            + FRAC_PI_8;
        let record = ParticleInstance::new(
            position,
            velocity,
            0.,
            time,
            lifetime,
            scale,
            matches!(emitter.frames, Frames::Tail),
        )
        .with_acceleration(acceleration, 0.)
        .with_facing(facing);
        state.retire(time);
        state.push(record, time, lifetime);
        if matches!(emitter.frames, Frames::Both) {
            state.push(record.as_tail(), time, lifetime);
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn bursts(
        &self,
        emitter: &AtlasEmitter,
        state: &mut State,
        frame: &EffectFrame<'_, '_, '_>,
        node: Mat4,
        dt: f64,
        emit: bool,
        actor: Option<&ActorFrame>,
    ) {
        // Effective track selection and clock bounds are shared with ordinary
        // property sampling, so inactive programmes cannot produce bursts.
        let track = if let Some(source) = frame.sample.values().boolean_source(emitter.rate) {
            frame.sample.track(&source).map(BurstTrack::Boolean)
        } else {
            frame.sample.track(&emitter.rate).map(BurstTrack::Scalar)
        };
        let clock = track
            .as_ref()
            .map_or(TimeSource::Animation, |value| value.clock().0);
        let period = track
            .as_ref()
            .and_then(|value| value.clock().1)
            .map(|v| v.as_seconds())
            .or_else(|| {
                frame
                    .selection
                    .clip
                    .and_then(|id| self.clips.get(&id))
                    .and_then(|(duration, looping)| {
                        (*looping && *duration > 0.).then_some(*duration)
                    })
            });
        let origin = if matches!(clock, TimeSource::Animation) {
            frame
                .sample
                .source_interval()
                .map_or(0., |(start, _)| start.as_seconds())
        } else {
            0.
        };
        let raw = match clock {
            TimeSource::Shared(_) => frame.selection.shared_time.as_seconds(),
            TimeSource::Animation => origin + frame.selection.clip_time.as_seconds(),
        };
        let identity = track
            .as_ref()
            .map_or(1, |value| value.times().as_bytes().as_ptr() as usize);
        if identity != state.previous_track
            || raw < state.previous_raw
            || state.previous_clip != frame.selection.clip
        {
            state.previous_raw = raw - dt;
            state.previous_track = identity;
            state.include_origin = true;
        }
        if state.seek_pending {
            state.previous_raw = raw - dt;
            state.include_origin = false;
            state.seek_pending = false;
        }
        let previous = state.previous_raw;
        let mut occurrences = take(&mut state.bursts);
        occurrences.clear();
        let exhausted = Cell::new(false);
        let mut add = |key: f64, value: f32| {
            if occurrences.len() >= MAX_BURST_OCCURRENCES {
                return;
            }
            let mut cycle = period.map_or(0., |period| ((previous - key) / period).ceil());
            for _ in 0..MAX_BURST_OCCURRENCES {
                if actor.is_some_and(|actor| !actor.step()) {
                    exhausted.set(true);
                    break;
                }
                if occurrences.len() >= MAX_BURST_OCCURRENCES {
                    break;
                }
                let occurrence = key + period.unwrap_or(0.) * cycle;
                if occurrence > raw + 1e-9 {
                    break;
                }
                if occurrence > previous + 1e-9
                    || (state.include_origin && (occurrence - previous).abs() < 1e-9)
                {
                    occurrences.push((occurrence, value));
                }
                if period.is_none() {
                    break;
                }
                cycle += 1.;
            }
        };
        if let Some(track) = track {
            let times = track.times();
            for (index, key) in times.iter().enumerate() {
                if actor.is_some_and(|actor| !actor.step()) {
                    exhausted.set(true);
                    break;
                }
                if times.get(index + 1) == Some(key) {
                    continue;
                }
                if let Some((start, end)) = frame.sample.source_interval() {
                    if matches!(clock, TimeSource::Animation) && (key < start || key > end) {
                        continue;
                    }
                }
                if period.is_some_and(|period| (key.as_seconds() - origin - period).abs() < 1e-9) {
                    continue;
                }
                add(key.as_seconds(), track.value(index));
            }
        } else if !emitter.animated_rate {
            add(origin, frame.sample.animated(&emitter.rate));
        }
        if exhausted.get() {
            state.skipped_seconds += dt;
            if let Some(actor) = actor {
                actor.budget.skip_seconds(dt);
            }
        }
        occurrences.sort_by(|a, b| a.0.total_cmp(&b.0));
        for &(occurrence, count) in
            occurrences
                .iter()
                .take(if emit { MAX_BURST_OCCURRENCES } else { 0 })
        {
            let time = frame.time - (raw - occurrence);
            let transform = interpolated(
                state.previous_node,
                node,
                ((time - (frame.time - dt)) / dt).clamp(0., 1.) as f32,
            );
            let count = state.phase.burst(f64::from(count.max(0.)), CAPACITY);
            for _ in 0..count {
                self.birth(emitter, state, frame, time, transform);
            }
        }
        state.previous_raw = raw;
        state.previous_clip = frame.selection.clip;
        state.include_origin = false;
        state.bursts = occurrences;
    }
}
fn interpolated(a: Mat4, b: Mat4, t: f32) -> Mat4 {
    let (sa, ra, ta) = a.to_scale_rotation_translation();
    let (sb, rb, tb) = b.to_scale_rotation_translation();
    Mat4::from_scale_rotation_translation(sa.lerp(sb, t), ra.slerp(rb, t), ta.lerp(tb, t))
}
pub(super) fn retimed_selection(
    frame: &EffectFrame<'_, '_, '_>,
    time: f64,
) -> Option<(ModelTime, ModelTime)> {
    let delta = frame.time - time;
    let clip = frame.selection.clip_time.as_seconds() - delta;
    Some((
        ModelTime::seconds(clip).ok()?,
        ModelTime::seconds(frame.selection.shared_time.as_seconds() - delta).ok()?,
    ))
}

impl RuntimeAtlasParticles {
    pub(crate) fn map_images(
        &mut self,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        for emitter in &mut self.emitters {
            emitter.texture = emitter.texture.as_ref().and_then(&mut *map);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use bevy::ecs::world::CommandQueue;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::formats::mdlx::mdl::Read as _;
    use wc3::model::formats::mdlx::{DynamicDocument, NoExtensions};
    use wc3::model::io::mdlx as semantic_mdlx;

    use wc3::model::{AnimationProgram, AnimationSample, Curve, DynamicTrack, SharedClock, Track};
    fn time(value: f64) -> ModelTime {
        ModelTime::seconds(value).unwrap()
    }
    fn fixture(keys: Vec<(f64, f32)>) -> (RuntimeAtlasParticles, AnimationProgram, State) {
        fixture_with_source(keys, false)
    }
    fn fixture_with_source(
        keys: Vec<(f64, f32)>,
        boolean: bool,
    ) -> (RuntimeAtlasParticles, AnimationProgram, State) {
        let source = DynamicDocument::<NoExtensions>::decode_mdl(include_str!(
            "../../tests/fixtures/particle_capture.mdl"
        ))
        .unwrap();
        let mut model: Model =
            semantic_mdlx::import_dynamic_owned(&source, &ImportOptions::default())
                .unwrap()
                .value
                .widen();
        let (feature, mut payload) = model
            .features()
            .iter()
            .find(|(_, value)| matches!(value.payload, NodeFeature::Particle(_)))
            .map(|(id, value)| (id, value.payload.clone()))
            .unwrap();
        let mut slot_values = model.values().clone();
        let property = slot_values.add(0.);
        let NodeFeature::Particle(ParticleEmitter::MdlxEmitter2(definition)) = &mut payload else {
            unreachable!()
        };
        definition.emission.trigger = true;
        definition.emission.rate = property;
        let boolean_source = model.nodes().iter().next().unwrap().1.visible;
        if boolean {
            definition.emission.rate = slot_values.scalar_from_boolean(boolean_source).unwrap();
        }
        model
            .edit(|builder| {
                builder.import_values(&slot_values)?;
                builder.set_feature(feature, payload)?;
                let clock = builder.add_clock(SharedClock::Repeating { period: time(1.) })?;
                if boolean {
                    return builder.add_global_track(DynamicTrack::Boolean(Track {
                        target: boolean_source,
                        curve: Curve::step(
                            keys.iter().map(|(t, _)| time(*t)).collect::<Vec<_>>(),
                            keys.iter().map(|(_, v)| *v > 0.).collect::<Vec<_>>(),
                        )?,
                        clock: TimeSource::Shared(clock),
                    }));
                }
                builder.add_global_track(DynamicTrack::Scalar(Track {
                    target: property,
                    curve: Curve::step(
                        keys.iter().map(|(t, _)| time(*t)).collect::<Vec<_>>(),
                        keys.iter().map(|(_, v)| *v).collect::<Vec<_>>(),
                    )?,
                    clock: TimeSource::Shared(clock),
                }))
            })
            .unwrap();
        let nodes = model
            .nodes()
            .iter()
            .enumerate()
            .map(|(index, (id, _))| (id, index))
            .collect();
        let mut runtime = RuntimeAtlasParticles::compile(&model, &nodes, &TextureImages::default());
        runtime.emitters.retain(|value| value.feature == feature);
        runtime.emitters[0].frames = Frames::Head;
        runtime.emitters[0].lifetime = 10000.;
        let mut world = World::new();
        let mut meshes = Assets::default();
        let mut queue = CommandQueue::default();
        let root = world.spawn_empty().id();
        let mut instances =
            runtime.spawn(&mut Commands::new(&mut queue, &world), &mut meshes, root);
        queue.apply(&mut world);
        (
            runtime,
            AnimationProgram::compile(&model),
            instances.states.remove(0),
        )
    }
    fn advance(
        runtime: &RuntimeAtlasParticles,
        program: &AnimationProgram,
        state: &mut State,
        now: f64,
        dt: f64,
        emit: bool,
    ) {
        let selection = AnimationSample {
            shared_time: time(now),
            ..Default::default()
        };
        let sample = program.sample(selection).unwrap();
        let poses = vec![Mat4::IDENTITY; runtime.emitters[0].anchor + 1];
        let visible = vec![true; poses.len()];
        let frame = EffectFrame {
            traversal: Default::default(),
            change: Default::default(),
            root_visible: true,
            pose: None,
            time: now,
            selection,
            sample: &sample,
            node_poses: &poses,
            node_visible: &visible,
            basis_world: Mat4::IDENTITY,
        };
        runtime.bursts(
            &runtime.emitters[0],
            state,
            &frame,
            Mat4::IDENTITY,
            dt,
            emit,
            None,
        );
    }
    #[test]
    fn boolean_burst_sources_keep_key_times_and_shared_cycles() {
        let (runtime, program, mut state) = fixture_with_source(vec![(0., 1.), (0.5, 0.)], true);
        advance(&runtime, &program, &mut state, 1.25, 1.25, true);
        assert_eq!(state.live.len(), 2);
        let births: Vec<_> = state
            .live
            .metadata()
            .iter()
            .map(|birth| birth.time)
            .collect();
        assert_eq!(births, vec![0., 1.]);
    }
    #[test]
    fn missing_bound_burst_track_does_not_emit_default() {
        let (runtime, prior, mut state) = fixture(vec![(0., 2.)]);
        let mut builder = Model::<wc3::model::profiles::Universal>::builder(ModelMetadata::Mdlx(
            Default::default(),
        ));
        builder.import_values(prior.values()).unwrap();
        builder
            .values_mut()
            .set(runtime.emitters[0].rate, 5.)
            .unwrap();
        let model = builder.finish().unwrap();
        let program = AnimationProgram::compile(&model);
        advance(&runtime, &program, &mut state, 0.25, 0.25, true);
        assert_eq!(state.live.len(), 0);
    }
    #[test]
    fn shared_clock_bursts_cross_cycles_once_without_frame_duplicates() {
        let (runtime, program, mut state) = fixture(vec![(0., 2.), (0.5, 3.)]);
        advance(&runtime, &program, &mut state, 1.25, 1.25, true);
        assert_eq!(state.live.len(), 7);
        let births: Vec<_> = state
            .live
            .metadata()
            .iter()
            .map(|birth| birth.time)
            .collect();
        assert_eq!(births, vec![0., 0., 0.5, 0.5, 0.5, 1., 1.]);
        advance(&runtime, &program, &mut state, 1.25, 0.1, true);
        assert_eq!(state.live.len(), 7);
        advance(&runtime, &program, &mut state, 2., 0.75, true);
        assert_eq!(state.live.len(), 12);
    }
    #[test]
    fn hidden_emission_does_not_replay_old_keys_when_reenabled() {
        let (runtime, program, mut state) = fixture(vec![(0., 2.), (0.5, 3.)]);
        advance(&runtime, &program, &mut state, 1.25, 1.25, false);
        assert_eq!(state.live.len(), 0);
        advance(&runtime, &program, &mut state, 1.25, 0.1, true);
        assert_eq!(state.live.len(), 0);
        advance(&runtime, &program, &mut state, 1.5, 0.25, true);
        assert_eq!(state.live.len(), 3);
    }
    #[test]
    fn extreme_clock_jumps_bound_total_occurrence_scratch_across_all_keys() {
        let (runtime, program, mut state) =
            fixture((0..64).map(|i| (f64::from(i) / 64., 0.)).collect());
        advance(&runtime, &program, &mut state, 4096., 4096., true);
        assert_eq!(state.bursts.len(), MAX_BURST_OCCURRENCES);
        assert!(state.bursts.capacity() <= MAX_BURST_OCCURRENCES);
        assert_eq!(state.live.len(), 0);
    }
    #[test]
    fn keyed_bursts_resume_after_seek_without_replaying_skipped_cycles() {
        let (runtime, program, mut state) = fixture(vec![(0., 2.), (0.5, 3.)]);
        advance(&runtime, &program, &mut state, 0.75, 0.75, true);
        assert_eq!(state.live.len(), 5);
        // A paused seek leaves the pending boundary until the next advancing update.
        state.seek_pending = true;
        advance(&runtime, &program, &mut state, 10.35, 0.1, true);
        assert_eq!(state.live.len(), 5);
        advance(&runtime, &program, &mut state, 10.6, 0.25, true);
        assert_eq!(state.live.len(), 8);
    }

    #[test]
    fn accepted_physical_records_own_motion_certificate_through_partial_capacity_retirement_and_reset(
    ) {
        let (_, _, mut state) = fixture(Vec::new());
        let stationary =
            ParticleInstance::new(Vec3::ZERO, Vec3::ZERO, 0., 0., 1., Vec3::ONE, false);
        let moving = ParticleInstance::new(Vec3::ZERO, Vec3::X, 0., 0., 1., Vec3::ONE, false);
        for _ in 0..CAPACITY - 1 {
            state.push(stationary, 0., 1.);
        }
        state.push(moving, 0., 1.);
        state.push(moving.as_tail(), 0., 1.); // Both only admitted its head.
        assert_eq!(state.live.len(), CAPACITY);
        assert_eq!(state.dynamic_records, 1);
        assert_eq!(state.dropped, 1);
        state.retire(0.5);
        assert_eq!(state.dynamic_records, 1);
        state.retire(1.);
        assert_eq!(state.live.len(), 0);
        assert_eq!(state.dynamic_records, 0);
        state.push(moving, 1., 0.5);
        state.push(moving.as_tail(), 1., 0.5);
        state.push(stationary, 1., 5.);
        assert_eq!(state.dynamic_records, 2);
        state.retire(1.5); // Both moving records retire; stationary suffix survives.
        assert_eq!(state.dynamic_records, 0);
        assert_eq!(state.live.len(), 1);
        state.push(moving, 1.5, 2.);
        assert_eq!(state.dynamic_records, 1);
        state.reset_records();
        assert_eq!(state.dynamic_records, 0);
        assert_eq!(state.live.len(), 0);
        assert!(state.indices.is_empty());
    }
}
