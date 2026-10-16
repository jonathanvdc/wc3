//! Direct staged sprite simulation using the resident immutable GPU transport.
use super::emission::{EffectChange, EmissionClock};
use super::{EffectFrame, SpriteEmitter};
use crate::effects::blend::EffectBlend;
use crate::effects::simulation::{EmissionPhase, EmissionRounding, EmitterRng, LiveRecords};
use crate::effects::sprites::{
    ParticleEmitterUniform, ParticleInstance, ParticleInstances as SpriteInstances,
    StagedSpriteParameters,
};
use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use std::array::from_fn;
use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8, PI};
use std::mem::take;
use std::sync::Arc;
use wc3::model::materials::state::BlendOperations;
use wc3::model::scene::keyframed::Frames;
use wc3::model::{
    Animated, ClipId, Clock, FeatureId, Model, NodeFeature, NodeId, ParticleActivation,
    ParticleAppearance, ParticleDetails, ParticleEmitter, ParticleGravity, ParticleScalar,
    ParticleTrigger, TextureId, Time as ModelTime,
};

const CAPACITY: usize = 8192;
const MAX_BURST_OCCURRENCES: usize = 4096;
#[derive(Clone)]
struct AtlasEmitter {
    feature: FeatureId,
    node: NodeId,
    anchor: usize,
    rate: Animated<f32>,
    speed: Animated<f32>,
    variation: Animated<f32>,
    gravity: Animated<f32>,
    lifetime: Animated<f32>,
    width: Animated<f32>,
    length: Animated<f32>,
    latitude: Animated<f32>,
    activation: ParticleActivation,
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
    diagnostics: Vec<(FeatureId, String)>,
}
struct Expiration {
    time: f64,
    lifetime: f32,
}
struct State {
    entity: Entity,
    live: LiveRecords<ParticleInstance, Expiration>,
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
#[derive(Default)]
pub(crate) struct AtlasParticleInstances {
    states: Vec<State>,
    previous_time: Option<f64>,
}
fn scalar(value: &ParticleScalar) -> Option<Animated<f32>> {
    if let ParticleScalar::Animated(value) = value {
        Some(value.clone())
    } else {
        None
    }
}
impl RuntimeAtlasParticles {
    pub(super) fn is_empty(&self) -> bool {
        self.emitters.is_empty()
    }

    pub fn compile(
        model: &Model<'_>,
        nodes: &HashMap<NodeId, usize>,
        images: &HashMap<TextureId, Handle<Image>>,
    ) -> Self {
        let mut result = Self {
            clips: model
                .clips()
                .map(|(id, c)| (id, (c.duration().as_seconds(), c.is_looping())))
                .collect(),
            ..Default::default()
        };
        let basis = super::super::coordinates::basis(model)
            .map(|basis| basis.to_matrix())
            .unwrap_or(Mat4::IDENTITY);
        result.gravity_axis = basis.transform_vector3(Vec3::Z);
        for (feature, value) in model.features() {
            let NodeFeature::Particle(ParticleEmitter::Simulated(definition)) = &value.payload
            else {
                continue;
            };
            let ParticleDetails::Atlas(details) = &definition.details else {
                continue;
            };
            let mut diagnostic =
                |message: &str| result.diagnostics.push((feature, message.to_string()));
            let ParticleAppearance::Staged(appearance) = &definition.appearance else {
                diagnostic("Staged sprite emitter requires its authored three-stage appearance");
                continue;
            };
            let (Some(rate), Some(speed), Some(variation), Some(lifetime)) = (
                scalar(&definition.emission.rate),
                scalar(&definition.motion.speed),
                scalar(&definition.motion.variation),
                scalar(&definition.lifetime.duration),
            ) else {
                diagnostic("Programmed scalar controls cannot drive a staged sprite emitter");
                continue;
            };
            let ParticleGravity::Directional(gravity) = &definition.motion.gravity else {
                diagnostic("Staged sprite emitter requires directional gravity");
                continue;
            };
            if matches!(details.rendering.frames, Frames::Unknown(_)) {
                diagnostic("Unknown staged sprite head/tail selection is not rendered");
                continue;
            }
            if details.unclassified.bits() != 0 {
                diagnostic("Additional staged sprite flag bits are retained without a visual interpretation");
            }
            if matches!(definition.emission.trigger, ParticleTrigger::KeyedBurst) {
                diagnostic("Keyed sprite emission processes at most 4096 crossed key occurrences per update; extreme clock jumps are bounded");
            }
            if !details.rendering.unfogged {
                diagnostic("Staged GPU sprites do not yet apply Bevy fog");
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
                        "Unknown sprite blend operation uses an explicit alpha-blend approximation",
                    );
                    EffectBlend::Alpha
                }
            };
            let texture = details
                .rendering
                .texture
                .and_then(|id| images.get(&id))
                .cloned();
            if texture.is_none() {
                diagnostic(
                    "Staged sprite image is unresolved; texture-provider integration is required",
                );
            }
            result.emitters.push(AtlasEmitter {
                feature,
                node: value.node,
                anchor: nodes[&value.node],
                rate,
                speed,
                variation,
                gravity: gravity.clone(),
                lifetime,
                width: details.shape.width.clone(),
                length: details.shape.length.clone(),
                latitude: details.shape.latitude.clone(),
                activation: definition.emission.activation.clone(),
                burst: matches!(definition.emission.trigger, ParticleTrigger::KeyedBurst),
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
                    scales: appearance.scale,
                    middle: appearance.middle_time,
                    intervals: appearance.uv_intervals,
                    atlas: details.rendering.atlas,
                    model_space: details.model_space,
                    xy_quad: details.rendering.xy_quad,
                    tail_length: details.rendering.tail_length,
                    unlit: details.rendering.unlit,
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
    pub fn diagnostics(&self) -> &[(FeatureId, String)] {
        &self.diagnostics
    }
    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        root: Entity,
    ) -> AtlasParticleInstances {
        let quad = (!self.emitters.is_empty()).then(|| meshes.add(Rectangle::new(2., 2.)));
        AtlasParticleInstances {
            states: self
                .emitters
                .iter()
                .map(|emitter| {
                    let entity = commands
                        .spawn((
                            Mesh3d(quad.as_ref().unwrap().clone()),
                            SpriteInstances {
                                records: Default::default(),
                                live_indices: Default::default(),
                                uniform: Default::default(),
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
    pub fn update(
        &self,
        instances: &mut AtlasParticleInstances,
        frame: &EffectFrame<'_, '_, '_>,
        render: &mut Query<&mut SpriteInstances>,
        stats: &mut Query<&mut SpriteEmitter>,
    ) {
        let previous = instances.previous_time.unwrap_or(frame.time);
        let reset = frame.time < previous || frame.change == EffectChange::Reset;
        let dt = (frame.time - previous).max(0.);
        for (emitter, state) in self.emitters.iter().zip(&mut instances.states) {
            let node = frame.basis_world * frame.feature_matrix(emitter.anchor);
            if reset || instances.previous_time.is_none() {
                state.live = Default::default();
                state.indices = Arc::default();
                state.phase.reset();
                state.rng = EmitterRng::new(emitter.anchor as u32);
                state.previous_node = node;
                state.previous_basis = frame.basis_world;
                state.include_origin = true;
                state.previous_track = 0;
            }
            state.seek_pending |= frame.change == EffectChange::Seek;
            state.birth_start = previous;
            state
                .live
                .retire(|birth| frame.time - birth.time >= f64::from(birth.lifetime));
            let enabled = match &emitter.activation {
                ParticleActivation::Fractional(v) => frame.sample.animated(v) > 0.,
                ParticleActivation::Boolean(v) => frame.sample.animated(v),
                ParticleActivation::Programmed => false,
            } && frame
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
                    );
                } else if frame.pose.is_some() || (enabled && !ended) {
                    let mut clock = EmissionClock::new(previous, frame.time);
                    let mut remaining = CAPACITY;
                    for step in clock.by_ref() {
                        let Some((clip, timeline, shared)) =
                            retimed_selection(frame, step.sample_time, &self.clips)
                        else {
                            continue;
                        };
                        let sample = frame.sample.at_times(clip, timeline, shared);
                        let active = match &emitter.activation {
                            ParticleActivation::Fractional(value) => sample.animated(value) > 0.,
                            ParticleActivation::Boolean(value) => sample.animated(value),
                            ParticleActivation::Programmed => false,
                        };
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
                    state.skipped_seconds += clock.omitted_seconds();
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
        let selection = retimed_selection(frame, time, &self.clips);
        let Some((clip, timeline, shared)) = selection else {
            return;
        };
        let sample = frame.sample.at_times(clip, timeline, shared);
        let enabled = match &emitter.activation {
            ParticleActivation::Fractional(value) => sample.animated(value) > 0.,
            ParticleActivation::Boolean(value) => sample.animated(value),
            ParticleActivation::Programmed => false,
        };
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
        let lifetime = sample.animated(&emitter.lifetime);
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
        state
            .live
            .retire(|birth| time - birth.time >= f64::from(birth.lifetime));
        let mut push = |record| {
            if state.live.len() < CAPACITY {
                state
                    .live
                    .push(record, Expiration { time, lifetime }, CAPACITY);
            } else {
                state.dropped = state.dropped.saturating_add(1);
            }
        };
        push(record);
        if matches!(emitter.frames, Frames::Both) {
            push(record.as_tail());
        }
    }
    fn bursts(
        &self,
        emitter: &AtlasEmitter,
        state: &mut State,
        frame: &EffectFrame<'_, '_, '_>,
        node: Mat4,
        dt: f64,
        emit: bool,
    ) {
        // Effective track selection and clock bounds are shared with ordinary
        // property sampling, so inactive programmes cannot produce bursts.
        let track = emitter
            .rate
            .property
            .as_ref()
            .and_then(|property| frame.sample.track(property));
        let clock = track
            .as_ref()
            .map_or(Clock::Clip, |value| value.track.clock);
        let period = track
            .as_ref()
            .and_then(|value| value.period)
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
        let origin = if matches!(clock, Clock::Timeline) {
            frame
                .selection
                .timeline_interval
                .map_or(0., |(start, _)| start.as_seconds())
        } else {
            0.
        };
        let raw = match clock {
            Clock::Clip => frame.selection.clip_time.as_seconds(),
            Clock::Shared(_) => frame.selection.shared_time.as_seconds(),
            Clock::Timeline => {
                if frame.selection.timeline_interval.is_some() {
                    origin + frame.selection.clip_time.as_seconds()
                } else {
                    frame.selection.timeline_time.as_seconds()
                }
            }
        };
        let identity = track.as_ref().map_or(1, |value| {
            value.track.curve.times().as_bytes().as_ptr() as usize
        });
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
        let mut add = |key: f64, value: f32| {
            if occurrences.len() >= MAX_BURST_OCCURRENCES {
                return;
            }
            let mut cycle = period.map_or(0., |period| ((previous - key) / period).ceil());
            for _ in 0..MAX_BURST_OCCURRENCES {
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
            let times = track.track.curve.times();
            for (index, key) in times.iter().enumerate() {
                if times.get(index + 1) == Some(key) {
                    continue;
                }
                if let Some((start, end)) = frame.selection.timeline_interval {
                    if matches!(clock, Clock::Timeline) && (key < start || key > end) {
                        continue;
                    }
                }
                if period.is_some_and(|period| (key.as_seconds() - origin - period).abs() < 1e-9) {
                    continue;
                }
                add(
                    key.as_seconds(),
                    track.track.curve.values().get(index).unwrap(),
                );
            }
        } else {
            add(origin, emitter.rate.default);
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
    clips: &HashMap<ClipId, (f64, bool)>,
) -> Option<(ModelTime, ModelTime, ModelTime)> {
    let delta = frame.time - time;
    let clip = frame.selection.clip_time.as_seconds() - delta;
    let timeline = if let Some((start, _)) = frame.selection.timeline_interval {
        let local = frame.selection.clip.and_then(|id| clips.get(&id)).map_or(
            clip,
            |(duration, looping)| {
                if *duration <= 0. {
                    0.
                } else if *looping {
                    clip.rem_euclid(*duration)
                } else {
                    clip.clamp(0., *duration)
                }
            },
        );
        start.as_seconds() + local
    } else {
        frame.selection.timeline_time.as_seconds() - delta
    };
    Some((
        ModelTime::seconds(clip).ok()?,
        ModelTime::seconds(timeline).ok()?,
        ModelTime::seconds(frame.selection.shared_time.as_seconds() - delta).ok()?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::CommandQueue;
    use wc3::model::conversion::ImportOptions;
    use wc3::model::formats::mdlx::mdl::Read as _;
    use wc3::model::formats::mdlx::{DynamicDocument, NoExtensions};
    use wc3::model::io::mdlx as semantic_mdlx;
    use wc3::model::profiles::mdlx::properties::particle2;
    use wc3::model::{AnimationProgram, AnimationSample, Channel, Curve, SharedClock, Track};
    fn time(value: f64) -> ModelTime {
        ModelTime::seconds(value).unwrap()
    }
    fn fixture(keys: Vec<(f64, f32)>) -> (RuntimeAtlasParticles, AnimationProgram, State) {
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
            .find(|(_, value)| matches!(value.payload, NodeFeature::Particle(_)))
            .map(|(id, value)| (id, value.payload.clone()))
            .unwrap();
        let property = particle2::EMISSION_RATE.bind(feature);
        let NodeFeature::Particle(ParticleEmitter::Simulated(definition)) = &mut payload else {
            unreachable!()
        };
        definition.emission.trigger = ParticleTrigger::KeyedBurst;
        definition.emission.rate = ParticleScalar::Animated(Animated::new(0., property.clone()));
        model
            .edit(|builder| {
                builder.set_feature(feature, payload)?;
                let clock = builder.add_clock(SharedClock { period: time(1.) })?;
                builder.add_shared_channel(Channel::Scalar(Track {
                    target: property.widen(),
                    curve: Curve::step(
                        keys.iter().map(|(t, _)| time(*t)).collect::<Vec<_>>(),
                        keys.iter().map(|(_, v)| *v).collect::<Vec<_>>(),
                    )?,
                    clock: Clock::Shared(clock),
                }))
            })
            .unwrap();
        let nodes = model
            .nodes()
            .enumerate()
            .map(|(index, (id, _))| (id, index))
            .collect();
        let mut runtime = RuntimeAtlasParticles::compile(&model, &nodes, &HashMap::new());
        runtime.emitters.retain(|value| value.feature == feature);
        runtime.emitters[0].frames = Frames::Head;
        runtime.emitters[0].lifetime = Animated::constant(10000.);
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
            camera_world: None,
        };
        runtime.bursts(
            &runtime.emitters[0],
            state,
            &frame,
            Mat4::IDENTITY,
            dt,
            emit,
        );
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
}
