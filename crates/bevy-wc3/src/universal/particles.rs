//! Deterministic M2 age-driven sprite simulation, independent of native codecs.
use super::atlas_particles::{AtlasParticleInstances, RuntimeAtlasParticles};
use super::material::{particle_material, UniversalMaterial};
use crate::effects::particle_emitter2::ParticleInstances as SpriteInstances;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::{MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::values::model::{
    ParticleBlend, ParticleFlags, ParticleGenerator, ParticleHeadTail,
};
use wc3::model::{
    Animated, AnimationSample, Curve, CurveValue, FeatureId, Model, NodeFeature, NodeId,
    ParticleActivation, ParticleAgeCurves, ParticleAppearance, ParticleDetails, ParticleEmitter,
    ParticleGravity, ParticleScalar, ParticleTrigger, SampledAnimation, TextureId,
    Time as ModelTime,
};

const MAX_PARTICLES: usize = 8192;
const MAX_BIRTHS_PER_UPDATE: usize = 8192;
const EMISSION_QUANTUM: f64 = 1.0 / 60.0;
const MAX_EMISSION_STEPS: usize = 4096;

/// Observable state and source identity of an age-driven emitter.
#[derive(Component, Clone, Debug)]
pub struct UniversalParticleEmitter {
    /// Semantic emitter identity.
    pub feature: FeatureId,
    /// Source hierarchy anchor.
    pub node: NodeId,
    /// Number of living birth records.
    pub alive: usize,
    /// Births omitted because the explicit CPU transport budget was reached.
    pub dropped: u64,
    /// Simulation time omitted by the explicit long-update emission budget.
    pub skipped_seconds: f64,
}

#[derive(Clone)]
struct Emitter {
    feature: FeatureId,
    node: NodeId,
    node_index: usize,
    rate: Animated<f32>,
    rate_variation: f32,
    activation: ParticleActivation,
    speed: Animated<f32>,
    speed_variation: Animated<f32>,
    gravity: ParticleGravity,
    lifespan: Animated<f32>,
    lifespan_variation: Animated<f32>,
    vertical: Animated<f32>,
    horizontal: Animated<f32>,
    width: Animated<f32>,
    length: Animated<f32>,
    z_source: Animated<f32>,
    generator: ParticleGenerator,
    flags: ParticleFlags,
    drag: f32,
    wind: Vec3,
    inherit_velocity: f32,
    spin: [f32; 4],
    appearance: ParticleAgeCurves<'static>,
    atlas: [u16; 2],
    texture_scale: f32,
    head_tail: ParticleHeadTail,
    tail_length: f32,
    seed: u64,
    material: UniversalMaterial,
}
#[derive(Clone, Default)]
pub(super) struct RuntimeParticles {
    atlas: RuntimeAtlasParticles,
    emitters: Vec<Emitter>,
    diagnostics: Vec<(FeatureId, String)>,
}

#[derive(Clone)]
struct Birth {
    time: f64,
    lifetime: f32,
    origin: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    spin: f32,
    spin_rate: f32,
    scale_variation: [f32; 2],
    random_cell: u32,
    size_scale: [f32; 2],
}
struct EmitterInstance {
    entity: Entity,
    mesh: Handle<Mesh>,
    births: Vec<Birth>,
    order: Vec<(usize, f32)>,
    fraction: f64,
    ordinal: u64,
    previous_node: Mat4,
    dropped: u64,
    skipped_seconds: f64,
}
#[derive(Default)]
pub(super) struct ParticleInstances {
    atlas: AtlasParticleInstances,
    emitters: Vec<EmitterInstance>,
    previous_time: Option<f64>,
    previous_camera: Option<Mat4>,
    previous_basis: Option<Mat4>,
}

#[derive(Clone, Copy)]
pub(super) struct ParticleFrame<'a, 'view, 'data> {
    pub time: f64,
    pub selection: AnimationSample<'a>,
    pub sample: &'a SampledAnimation<'view, 'data>,
    /// Current model-space hierarchy matrices, in compiler node order.
    pub node_poses: &'a [Mat4],
    /// Current inherited node visibility, in the same order.
    pub node_visible: &'a [bool],
    pub basis_world: Mat4,
    pub camera_world: Option<Mat4>,
}

fn scalar(value: &ParticleScalar) -> Option<Animated<f32>> {
    if let ParticleScalar::Animated(value) = value {
        Some(value.clone())
    } else {
        None
    }
}
fn owned_curve<T: CurveValue>(value: &Option<Curve<'_, T>>) -> Option<Curve<'static, T>> {
    value.clone().map(Curve::into_owned)
}
fn owned_age(value: &ParticleAgeCurves<'_>) -> ParticleAgeCurves<'static> {
    ParticleAgeCurves {
        legacy_atlas: value.legacy_atlas.clone(),
        color: owned_curve(&value.color),
        alpha: owned_curve(&value.alpha),
        scale: owned_curve(&value.scale),
        scale_variation: value.scale_variation,
        head_uv: owned_curve(&value.head_uv),
        tail_uv: owned_curve(&value.tail_uv),
    }
}
fn random(seed: &mut u64) -> f32 {
    *seed = seed.wrapping_add(0x9e3779b97f4a7c15);
    let mut value = *seed;
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((value ^ (value >> 31)) >> 40) as f32 / 16777216.0
}
fn signed(seed: &mut u64) -> f32 {
    2.0 * random(seed) - 1.0
}

impl RuntimeParticles {
    pub(super) fn is_empty(&self) -> bool {
        self.emitters.is_empty() && self.atlas.is_empty()
    }

    pub fn compile(
        model: &Model<'_>,
        node_indices: &HashMap<NodeId, usize>,
        images: &HashMap<TextureId, Handle<Image>>,
    ) -> Self {
        let atlas = RuntimeAtlasParticles::compile(model, node_indices, images);
        let mut result = Self {
            diagnostics: atlas.diagnostics().to_vec(),
            atlas,
            ..Self::default()
        };
        for (feature, value) in model.features() {
            let NodeFeature::Particle(ParticleEmitter::Simulated(definition)) = &value.payload
            else {
                continue;
            };
            let ParticleDetails::AgeDriven(details) = &definition.details else {
                continue;
            };
            let mut diagnostic =
                |message: &str| result.diagnostics.push((feature, message.to_string()));
            let ParticleAppearance::Curves(appearance) = &definition.appearance else {
                diagnostic("M2 emitter has no normalized-age appearance programme; sprite simulation is unavailable");
                continue;
            };
            if details.flags.packed_gravity() {
                diagnostic("M2 packed directional gravity requires decoding before simulation; this emitter is not approximated as scalar gravity");
                continue;
            }
            if !matches!(
                details.shape.generator,
                ParticleGenerator::Plane | ParticleGenerator::Sphere
            ) {
                diagnostic("M2 spline, bone and unknown generators require a dedicated emission interpreter");
                continue;
            }
            if !matches!(definition.emission.trigger, ParticleTrigger::Continuous) {
                diagnostic("M2 keyed or programmed burst timing is not yet simulated");
                continue;
            }
            let (
                Some(rate),
                Some(speed),
                Some(speed_variation),
                Some(lifespan),
                Some(lifespan_variation),
            ) = (
                scalar(&definition.emission.rate),
                scalar(&definition.motion.speed),
                scalar(&definition.motion.variation),
                scalar(&definition.lifetime.duration),
                scalar(&definition.lifetime.variation),
            )
            else {
                diagnostic(
                    "Programmed controls cannot be evaluated as M2 animated scalar controls",
                );
                continue;
            };
            if !matches!(
                definition.motion.gravity,
                ParticleGravity::Directional(_) | ParticleGravity::Vector(_)
            ) {
                diagnostic("Unresolved encoded gravity controls are retained but not simulated");
                continue;
            }
            if details.rendering.model.is_some() || details.rendering.child_model.is_some() {
                diagnostic("M2 model particles and child emitter resources are not instantiated by the sprite transport");
                continue;
            }
            if details.rendering.textures.len() > 1 {
                diagnostic(
                    "M2 multi-texture particles currently render their first texture layer only",
                );
            }
            if let ParticleBlend::Unknown(_) = details.rendering.blend {
                diagnostic(
                    "Unknown M2 particle blending uses an explicit alpha-blend approximation",
                );
            }
            if details.wind_time != 0.
                || details.follow.iter().flatten().any(|v| *v != 0.)
                || details.twinkle_speed != 0.
                || details.twinkle_percent != 0.
            {
                diagnostic("M2 wind timing, emitter-follow and twinkle controls are retained but not evaluated");
            }
            if details.rendering.scroll.is_some() || details.rendering.texture_tile_rotation != 0 {
                diagnostic("M2 particle texture scrolling and tile rotation are not evaluated");
            }
            if appearance.legacy_atlas.is_some() {
                diagnostic("Legacy particle atlas segments currently use normalized-age cell curves when present");
            }
            if !matches!(details.rendering.head_tail, ParticleHeadTail::Head) {
                diagnostic("M2 tails use velocity times the authored tail parameter; exact client time-versus-distance fidelity remains unverified");
            }
            diagnostic("M2 sprite births interpolate frame endpoint poses and sample model controls at the current update; highly curved subframe emitter motion remains approximate");
            diagnostic("CPU sprite transport limits each emitter to 8192 living particles, 8192 births and 4096 emission intervals per update; omitted births and long-update seconds are reported");
            let image = details
                .rendering
                .textures
                .first()
                .and_then(|id| images.get(id))
                .cloned();
            if image.is_none() && !details.rendering.textures.is_empty() {
                diagnostic(
                    "Unresolved primary particle image uses a visible white sprite fallback",
                );
            }
            let supported_flags = (1 << 0)
                | (1 << 2)
                | (1 << 3)
                | (1 << 5)
                | (1 << 7)
                | (1 << 12)
                | (1 << 16)
                | (1 << 17)
                | (1 << 18);
            if details.flags.bits() & !supported_flags != 0 {
                diagnostic("Additional M2 generator flags are retained but not evaluated by the sprite transport");
            }

            result.emitters.push(Emitter {
                feature,
                node: value.node,
                node_index: node_indices[&value.node],
                rate,
                rate_variation: definition.emission.variation,
                activation: definition.emission.activation.clone(),
                speed,
                speed_variation,
                gravity: definition.motion.gravity.clone(),
                lifespan,
                lifespan_variation,
                vertical: details.shape.vertical_range.clone(),
                horizontal: details.shape.horizontal_range.clone(),
                width: details.shape.area_width.clone(),
                length: details.shape.area_length.clone(),
                z_source: details.shape.z_source.clone(),
                generator: details.shape.generator,
                flags: details.flags,
                drag: details.drag.max(0.),
                wind: Vec3::from(details.wind),
                inherit_velocity: details.inherit_velocity,
                spin: details.spin,
                appearance: owned_age(appearance),
                atlas: details.rendering.atlas,
                texture_scale: details.rendering.texture_scale[0],
                head_tail: details.rendering.head_tail,
                tail_length: details.rendering.tail_length,
                seed: u64::from(details.id)
                    .wrapping_add(result.emitters.len() as u64)
                    .wrapping_add(1),
                material: particle_material(
                    details.rendering.blend,
                    image,
                    details.flags.lit_by_light() && !details.flags.unlit(),
                ),
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
        materials: &mut Assets<UniversalMaterial>,
        basis: Entity,
    ) -> ParticleInstances {
        let emitters = self
            .emitters
            .iter()
            .map(|emitter| {
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, Vec::<[f32; 3]>::new());
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
                let mesh = meshes.add(mesh);
                let entity = commands
                    .spawn((
                        UniversalParticleEmitter {
                            feature: emitter.feature,
                            node: emitter.node,
                            alive: 0,
                            dropped: 0,
                            skipped_seconds: 0.,
                        },
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(materials.add(emitter.material.clone())),
                        Transform::default(),
                        Visibility::default(),
                        NoFrustumCulling,
                        ChildOf(basis),
                    ))
                    .id();
                EmitterInstance {
                    entity,
                    mesh,
                    births: Vec::new(),
                    order: Vec::new(),
                    fraction: 0.,
                    ordinal: 0,
                    previous_node: Mat4::IDENTITY,
                    dropped: 0,
                    skipped_seconds: 0.,
                }
            })
            .collect();
        ParticleInstances {
            atlas: self.atlas.spawn(commands, meshes, basis),
            emitters,
            previous_time: None,
            previous_camera: None,
            previous_basis: None,
        }
    }
    pub fn update(
        &self,
        instances: &mut ParticleInstances,
        frame: ParticleFrame<'_, '_, '_>,
        meshes: &mut Assets<Mesh>,
        statistics: &mut Query<&mut UniversalParticleEmitter>,
        sprites: &mut Query<&mut SpriteInstances>,
    ) {
        if !frame.time.is_finite() {
            return;
        }
        self.atlas
            .update(&mut instances.atlas, &frame, sprites, statistics);
        let previous = instances.previous_time.unwrap_or(frame.time);
        let backwards = frame.time < previous;
        let start = if backwards { frame.time } else { previous };
        let delta = frame.time - start;
        for (emitter, state) in self.emitters.iter().zip(&mut instances.emitters) {
            let node = frame.basis_world * frame.node_poses[emitter.node_index];
            if instances.previous_time.is_none() || backwards {
                state.births.clear();
                state.fraction = 0.;
                state.ordinal = 0;
                state.previous_node = node;
                state.dropped = 0;
                state.skipped_seconds = 0.;
            }
            state
                .births
                .retain(|birth| frame.time - birth.time < f64::from(birth.lifetime));
            let active = match &emitter.activation {
                ParticleActivation::Boolean(v) => {
                    if frame.sample.animated(v) {
                        1.
                    } else {
                        0.
                    }
                }
                ParticleActivation::Fractional(v) => frame.sample.animated(v).clamp(0., 1.),
                ParticleActivation::Programmed => 0.,
            };
            let active = if frame
                .node_visible
                .get(emitter.node_index)
                .copied()
                .unwrap_or(true)
            {
                active
            } else {
                0.
            };
            let base_rate = frame.sample.animated(&emitter.rate).max(0.) * active;
            let inherited = if delta > 0. {
                (node.w_axis.truncate() - state.previous_node.w_axis.truncate()) / delta as f32
                    * emitter.inherit_velocity
            } else {
                Vec3::ZERO
            };
            let mut at = start;
            let mut births_this_update = 0;
            let mut emission_steps = 0;
            // A fixed variation clock prevents random emission-rate variance
            // from depending on update frequency or screenshot readback time.
            while at < frame.time && emission_steps < MAX_EMISSION_STEPS && active > 0. {
                emission_steps += 1;
                let tick = (at / EMISSION_QUANTUM + 1e-9).floor();
                let end = frame.time.min((tick + 1.) * EMISSION_QUANTUM);
                if end <= at {
                    break;
                }
                let mut rate_seed =
                    emitter.seed ^ (tick as i64 as u64).wrapping_mul(0x517cc1b727220a95);
                let rate = (base_rate + signed(&mut rate_seed) * emitter.rate_variation * active)
                    .max(0.) as f64;
                if rate > 0. {
                    let prior = state.fraction;
                    let amount = prior + (end - at) * rate;
                    let count = amount.floor() as usize;
                    state.fraction = amount - count as f64;
                    let available = MAX_BIRTHS_PER_UPDATE
                        .saturating_sub(births_this_update)
                        .min(MAX_PARTICLES.saturating_sub(state.births.len()));
                    let accepted = count.min(available);
                    let omitted = count - accepted;
                    state.dropped = state.dropped.saturating_add(omitted as u64);
                    for index in 0..accepted {
                        let time = at + (index as f64 + 1. - prior) / rate;
                        let fraction = if delta > 0. {
                            ((time - start) / delta) as f32
                        } else {
                            1.
                        };
                        let birth_node =
                            interpolated_transform(state.previous_node, node, fraction);
                        let ordinal = state.ordinal;
                        state.ordinal = state.ordinal.wrapping_add(1);
                        if state.births.len() >= MAX_PARTICLES
                            || births_this_update >= MAX_BIRTHS_PER_UPDATE
                        {
                            state.dropped = state.dropped.saturating_add(1);
                            continue;
                        }
                        births_this_update += 1;
                        if let Some(birth) =
                            emitter.birth(frame.sample, time, birth_node, inherited, ordinal)
                        {
                            if frame.time - birth.time < f64::from(birth.lifetime) {
                                state.births.push(birth);
                            }
                        }
                    }
                    state.ordinal = state.ordinal.wrapping_add(omitted as u64);
                }
                at = end;
            }
            if at < frame.time && active > 0. {
                state.skipped_seconds += frame.time - at;
            }
            let node_changed = state.previous_node != node;
            state.previous_node = node;
            let redraw = backwards
                || delta != 0.
                || instances.previous_camera != frame.camera_world
                || instances.previous_basis != Some(frame.basis_world)
                || node_changed;
            if redraw {
                if let Some(mut mesh) = meshes.get_mut(&state.mesh) {
                    emitter.draw(state, &frame, node, &mut mesh);
                }
            }
            if let Ok(mut stats) = statistics.get_mut(state.entity) {
                if stats.alive != state.births.len() {
                    stats.alive = state.births.len();
                }
                if stats.dropped != state.dropped {
                    stats.dropped = state.dropped;
                }
                if stats.skipped_seconds != state.skipped_seconds {
                    stats.skipped_seconds = state.skipped_seconds;
                }
            }
        }
        instances.previous_time = Some(frame.time);
        instances.previous_camera = frame.camera_world;
        instances.previous_basis = Some(frame.basis_world);
    }
}

fn interpolated_transform(a: Mat4, b: Mat4, t: f32) -> Mat4 {
    let (sa, ra, ta) = a.to_scale_rotation_translation();
    let (sb, rb, tb) = b.to_scale_rotation_translation();
    Mat4::from_scale_rotation_translation(sa.lerp(sb, t), ra.slerp(rb, t), ta.lerp(tb, t))
}
impl Emitter {
    fn birth(
        &self,
        sample: &SampledAnimation<'_, '_>,
        time: f64,
        node: Mat4,
        inherited: Vec3,
        ordinal: u64,
    ) -> Option<Birth> {
        if !node.is_finite() || node.determinant() == 0. {
            return None;
        }
        let mut seed = self.seed ^ ordinal.wrapping_mul(0xd6e8feb86659fd93);
        let lifetime = sample.animated(&self.lifespan)
            + signed(&mut seed) * sample.animated(&self.lifespan_variation);
        if !lifetime.is_finite() || lifetime <= 0. {
            return None;
        }
        let speed = sample.animated(&self.speed)
            * (1. + signed(&mut seed) * sample.animated(&self.speed_variation));
        let vertical = sample.animated(&self.vertical);
        let horizontal = sample.animated(&self.horizontal);
        let theta = signed(&mut seed) * vertical;
        let phi = signed(&mut seed) * horizontal;
        let mut direction = Vec3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        );
        let width = sample.animated(&self.width);
        let length = sample.animated(&self.length);
        let origin = match self.generator {
            ParticleGenerator::Plane => Vec3::new(
                signed(&mut seed) * length * 0.5,
                signed(&mut seed) * width * 0.5,
                0.,
            ),
            ParticleGenerator::Sphere => {
                let radius = length + (width - length) * random(&mut seed);
                direction * radius
            }
            _ => return None,
        };
        let z_source = sample.animated(&self.z_source);
        if z_source != 0. {
            direction = (origin - Vec3::Z * z_source)
                .try_normalize()
                .unwrap_or(direction);
        }
        if self.flags.outward() {
            direction = origin.try_normalize().unwrap_or(direction);
        }
        if self.flags.inward() {
            direction = -origin.try_normalize().unwrap_or(direction);
        }
        let gravity = match &self.gravity {
            ParticleGravity::Directional(v) => Vec3::NEG_Z * sample.animated(v),
            ParticleGravity::Vector(v) => Vec3::from(sample.animated(v)),
            _ => Vec3::ZERO,
        };
        let local = self.flags.model_space();
        let (origin, mut velocity, acceleration) = if local {
            (origin, direction * speed, gravity + self.wind)
        } else {
            (
                node.transform_point3(origin),
                node.transform_vector3(direction * speed),
                node.transform_vector3(gravity) + self.wind,
            )
        };
        if self.flags.travel_up_world_space() {
            velocity = if local {
                node.inverse().transform_vector3(Vec3::Z * speed)
            } else {
                Vec3::Z * speed
            };
        }
        velocity += if local {
            node.inverse().transform_vector3(inherited)
        } else {
            inherited
        };
        let scale_variation = self
            .appearance
            .scale_variation
            .map(|v| 1. + signed(&mut seed) * v);
        Some(Birth {
            time,
            lifetime,
            origin,
            velocity,
            acceleration,
            spin: self.spin[0] + signed(&mut seed) * self.spin[1],
            spin_rate: self.spin[2] + signed(&mut seed) * self.spin[3],
            scale_variation,
            size_scale: [
                node.x_axis.truncate().length(),
                node.y_axis.truncate().length(),
            ],
            random_cell: if self.flags.random_texture() {
                (random(&mut seed)
                    * f32::from(self.atlas[0].max(1))
                    * f32::from(self.atlas[1].max(1))) as u32
            } else {
                0
            },
        })
    }
    fn motion(&self, birth: &Birth, time: f64) -> (Vec3, Vec3) {
        let age = (time - birth.time).max(0.) as f32;
        if self.drag > 0.00001 {
            let decay = (-self.drag * age).exp();
            let vscale = (1. - decay) / self.drag;
            let ascale = (age - vscale) / self.drag;
            (
                birth.origin + birth.velocity * vscale + birth.acceleration * ascale,
                birth.velocity * decay + birth.acceleration * vscale,
            )
        } else {
            (
                birth.origin + birth.velocity * age + birth.acceleration * (0.5 * age * age),
                birth.velocity + birth.acceleration * age,
            )
        }
    }
    fn draw(
        &self,
        state: &mut EmitterInstance,
        frame: &ParticleFrame<'_, '_, '_>,
        node: Mat4,
        mesh: &mut Mesh,
    ) {
        if !frame.basis_world.is_finite() || frame.basis_world.determinant() == 0. {
            return;
        }
        let inverse_basis = frame.basis_world.inverse();
        let camera = frame.camera_world.unwrap_or(frame.basis_world);
        let camera_position = camera.w_axis.truncate();
        let right = camera.x_axis.truncate().normalize_or_zero();
        let up = camera.y_axis.truncate().normalize_or_zero();
        let camera_back = camera.z_axis.truncate().normalize_or_zero();
        state.order.clear();
        state
            .order
            .extend(state.births.iter().enumerate().map(|(index, birth)| {
                let mut position = self.motion(birth, frame.time).0;
                if self.flags.model_space() {
                    position = node.transform_point3(position);
                }
                (index, (position - camera_position).dot(camera_back))
            }));
        state
            .order
            .sort_unstable_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        let mut positions = take_vec3(mesh, Mesh::ATTRIBUTE_POSITION);
        let mut normals = take_vec3(mesh, Mesh::ATTRIBUTE_NORMAL);
        let mut uv = take_vec2(mesh, Mesh::ATTRIBUTE_UV_0);
        let mut colors = take_vec4(mesh, Mesh::ATTRIBUTE_COLOR);
        for &(index, _) in &state.order {
            let birth = &state.births[index];
            let age = ((frame.time - birth.time) / f64::from(birth.lifetime)).clamp(0., 1.);
            let age = ModelTime::seconds(age).unwrap();
            let rgb = self
                .appearance
                .color
                .as_ref()
                .and_then(|v| v.sample(age))
                .unwrap_or([1.; 3]);
            let alpha = self
                .appearance
                .alpha
                .as_ref()
                .and_then(|v| v.sample(age))
                .unwrap_or(1.)
                .clamp(0., 1.);
            let mut size = self
                .appearance
                .scale
                .as_ref()
                .and_then(|v| v.sample(age))
                .unwrap_or([1.; 2]);
            for (size, variation) in size.iter_mut().zip(birth.scale_variation) {
                *size *= variation;
            }
            let (mut position, mut velocity) = self.motion(birth, frame.time);
            if self.flags.model_space() {
                position = node.transform_point3(position);
                velocity = node.transform_vector3(velocity);
            }
            let spin = birth.spin + birth.spin_rate * (frame.time - birth.time) as f32;
            let (sin, cos) = spin.sin_cos();
            let (r, u) = if self.flags.align_xy() || !self.flags.billboard() {
                (
                    node.x_axis.truncate().normalize_or_zero(),
                    node.y_axis.truncate().normalize_or_zero(),
                )
            } else {
                (right, up)
            };
            let scale = if self.flags.model_space() {
                [
                    node.x_axis.truncate().length(),
                    node.y_axis.truncate().length(),
                ]
            } else {
                birth.size_scale
            };
            let horizontal = (r * cos + u * sin) * size[0] * scale[0] * 0.5;
            let vertical = (u * cos - r * sin) * size[1] * scale[1] * 0.5;
            let (r, u) = (horizontal, vertical);
            let head = self
                .appearance
                .head_uv
                .as_ref()
                .and_then(|v| v.sample(age))
                .unwrap_or(0) as u32
                + birth.random_cell;
            let tail = self
                .appearance
                .tail_uv
                .as_ref()
                .and_then(|v| v.sample(age))
                .unwrap_or(0) as u32
                + birth.random_cell;
            if matches!(
                self.head_tail,
                ParticleHeadTail::Head | ParticleHeadTail::Both
            ) {
                quad(
                    [
                        position - r - u,
                        position + r - u,
                        position + r + u,
                        position - r + u,
                    ],
                    head,
                    self.atlas,
                    self.texture_scale,
                    [rgb[0], rgb[1], rgb[2], alpha],
                    inverse_basis,
                    &mut positions,
                    &mut normals,
                    &mut uv,
                    &mut colors,
                );
            }
            if matches!(
                self.head_tail,
                ParticleHeadTail::Tail | ParticleHeadTail::Both
            ) {
                let end = position - velocity * self.tail_length;
                let side = (position - end)
                    .cross(camera_position - position)
                    .normalize_or_zero()
                    * size[0]
                    * 0.5;
                quad(
                    [end - side, end + side, position + side, position - side],
                    tail,
                    self.atlas,
                    self.texture_scale,
                    [rgb[0], rgb[1], rgb[2], alpha],
                    inverse_basis,
                    &mut positions,
                    &mut normals,
                    &mut uv,
                    &mut colors,
                );
            }
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
}
fn take_vec3(mesh: &mut Mesh, attribute: MeshVertexAttribute) -> Vec<[f32; 3]> {
    let mut values = match mesh.remove_attribute(attribute) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => Vec::new(),
    };
    values.clear();
    values
}
fn take_vec2(mesh: &mut Mesh, attribute: MeshVertexAttribute) -> Vec<[f32; 2]> {
    let mut values = match mesh.remove_attribute(attribute) {
        Some(VertexAttributeValues::Float32x2(values)) => values,
        _ => Vec::new(),
    };
    values.clear();
    values
}
fn take_vec4(mesh: &mut Mesh, attribute: MeshVertexAttribute) -> Vec<[f32; 4]> {
    let mut values = match mesh.remove_attribute(attribute) {
        Some(VertexAttributeValues::Float32x4(values)) => values,
        _ => Vec::new(),
    };
    values.clear();
    values
}
#[allow(clippy::too_many_arguments)]
fn quad(
    corners: [Vec3; 4],
    cell: u32,
    atlas: [u16; 2],
    scale: f32,
    color: [f32; 4],
    inverse: Mat4,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uv: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
) {
    let rows = u32::from(atlas[0].max(1));
    let columns = u32::from(atlas[1].max(1));
    let cell = cell % (rows * columns);
    let origin = Vec2::new(
        (cell % columns) as f32 / columns as f32,
        (cell / columns) as f32 / rows as f32,
    );
    let extent = Vec2::new(scale / columns as f32, scale / rows as f32);
    let coordinates = [
        origin,
        origin + Vec2::X * extent.x,
        origin + extent,
        origin + Vec2::Y * extent.y,
    ];
    let transformed = corners.map(|v| inverse.transform_point3(v));
    let normal = (transformed[1] - transformed[0])
        .cross(transformed[2] - transformed[0])
        .normalize_or_zero()
        .to_array();
    for index in [0, 1, 2, 0, 2, 3] {
        positions.push(transformed[index].to_array());
        normals.push(normal);
        uv.push(coordinates[index].to_array());
        colors.push(color);
    }
}

#[cfg(test)]
#[path = "particles_tests.rs"]
mod tests;
