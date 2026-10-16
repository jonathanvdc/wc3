//! Deterministic M2 age-driven sprite simulation, independent of native codecs.
use super::super::material::{particle_material, ModelMaterial, SurfaceAnimation, SurfacePass};
use super::atlas_sprites::retimed_selection;
use super::emission::{EffectChange, EmissionClock};
use super::frame::EffectFrame;
use super::programmed::{ProgrammedAppearance, ProgrammedSprite, SquirtSchedule};
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::mesh::{MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use std::array::from_fn;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::values::model::{
    ParticleBlend, ParticleFlags, ParticleGenerator, ParticleHeadTail,
};
use wc3::model::{
    Animated, ClipId, Curve, CurveValue, FeatureId, MaterialId, Model, NodeFeature, NodeId,
    ParticleActivation, ParticleAgeCurves, ParticleAppearance, ParticleDetails, ParticleEmitter,
    ParticleGravity, ParticleScalar, ParticleTrigger, RetimedAnimation, TextureId,
    Time as ModelTime,
};

const MAX_PARTICLES: usize = 8192;
const MAX_BIRTHS_PER_UPDATE: usize = 8192;

/// Observable state and source identity of an age-driven emitter.
#[derive(Component, Clone, Debug)]
pub struct SpriteEmitter {
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
    follow: [[f32; 2]; 2],
    spin: [f32; 4],
    appearance: ParticleAgeCurves<'static>,
    atlas: [u16; 2],
    texture_scale: f32,
    head_tail: ParticleHeadTail,
    tail_length: f32,
    seed: u64,
    material: ModelMaterial,
    material_animation: Option<Arc<SurfaceAnimation>>,
    programmed: Option<ProgrammedSprite>,
}
#[derive(Clone, Default)]
pub(super) struct RuntimeAgeSprites {
    clips: HashMap<ClipId, (f64, bool)>,
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
    appearance_index: u32,
}
struct EmitterInstance {
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<ModelMaterial>,
    births: Vec<Birth>,
    squirts: SquirtSchedule,
    appearances: Vec<ProgrammedAppearance>,
    order: Vec<(usize, f32)>,
    fraction: f64,
    ordinal: u64,
    previous_node: Mat4,
    pose_scratch: Vec<usize>,
    dropped: u64,
    skipped_seconds: f64,
}
impl EmitterInstance {
    fn retire(&mut self, now: f64) {
        let mut retained = 0;
        let appearances = &mut self.appearances;
        self.births.retain_mut(|birth| {
            if now - birth.time >= f64::from(birth.lifetime) {
                return false;
            }
            if birth.appearance_index != u32::MAX {
                let index = birth.appearance_index as usize;
                if index != retained {
                    appearances.swap(index, retained);
                }
                birth.appearance_index = retained as u32;
                retained += 1;
            }
            true
        });
        appearances.truncate(retained);
    }
}
#[derive(Default)]
pub(super) struct AgeSpriteInstances {
    emitters: Vec<EmitterInstance>,
    previous_time: Option<f64>,
    previous_camera: Option<Mat4>,
    previous_basis: Option<Mat4>,
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

impl RuntimeAgeSprites {
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
                value.node_index = indices[value.node_index];

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
        node_indices: &HashMap<NodeId, usize>,
        images: &HashMap<TextureId, Handle<Image>>,
        materials: &HashMap<MaterialId, Vec<SurfacePass>>,
    ) -> Self {
        let mut result = Self {
            clips: model
                .clips()
                .map(|(id, c)| (id, (c.duration().as_seconds(), c.is_looping())))
                .collect(),
            ..Self::default()
        };
        for (feature, value) in model.features() {
            let NodeFeature::Particle(ParticleEmitter::Simulated(definition)) = &value.payload
            else {
                continue;
            };
            if let ParticleDetails::Programmed(details) = &definition.details {
                let programme = match ProgrammedSprite::compile(definition) {
                    Ok((programme, diagnostics)) => {
                        result
                            .diagnostics
                            .extend(diagnostics.into_iter().map(|message| (feature, message)));
                        programme
                    }
                    Err(message) => {
                        result.diagnostics.push((feature, message));
                        continue;
                    }
                };
                let Some(passes) = details.material.and_then(|id| materials.get(&id)) else {
                    result.diagnostics.push((
                        feature,
                        "Programmed billboard material does not resolve to a compiled PBR surface"
                            .into(),
                    ));
                    continue;
                };
                let Some(pass) = passes.first() else {
                    continue;
                };
                if passes.len() > 1 {
                    result.diagnostics.push((feature,"Programmed billboards currently draw the primary material pass; supplementary passes are not instantiated".into()));
                }
                let mut material = ModelMaterial {
                    base: pass.material.clone(),
                    extension: pass.state.clone(),
                };
                material.base.cull_mode = None;
                material.base.unlit |= !programme.lit;
                result.emitters.push(Emitter::programmed(
                    feature,
                    value.node,
                    node_indices[&value.node],
                    programme,
                    material,
                    Arc::new(pass.animation.clone()),
                ));
                continue;
            }
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
                || details.twinkle_speed != 0.
                || details.twinkle_percent != 0.
            {
                diagnostic("M2 wind timing and twinkle controls are retained but not evaluated");
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
            diagnostic("M2 particle motion interpolates application root transforms between updates; inherited velocity uses a frame-interval estimate");
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
                material_animation: None,
                programmed: None,
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
                follow: details.follow,
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
        materials: &mut Assets<ModelMaterial>,
        basis: Entity,
    ) -> AgeSpriteInstances {
        let emitters = self
            .emitters
            .iter()
            .map(|emitter| {
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                );
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.; 3]; 3]);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; 3]);
                mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.; 2]; 3]);
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.; 4]; 3]);
                let mesh = meshes.add(mesh);
                let material = materials.add(emitter.material.clone());
                let entity = commands
                    .spawn((
                        SpriteEmitter {
                            feature: emitter.feature,
                            node: emitter.node,
                            alive: 0,
                            dropped: 0,
                            skipped_seconds: 0.,
                        },
                        Mesh3d(mesh.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::default(),
                        Visibility::default(),
                        NoFrustumCulling,
                        ChildOf(basis),
                    ))
                    .id();
                EmitterInstance {
                    entity,
                    mesh,
                    material,
                    births: Vec::new(),
                    squirts: SquirtSchedule::default(),
                    appearances: Vec::new(),
                    order: Vec::new(),
                    fraction: 0.,
                    ordinal: 0,
                    previous_node: Mat4::IDENTITY,
                    pose_scratch: Vec::new(),
                    dropped: 0,
                    skipped_seconds: 0.,
                }
            })
            .collect();
        AgeSpriteInstances {
            emitters,
            previous_time: None,
            previous_camera: None,
            previous_basis: None,
        }
    }
    pub fn update(
        &self,
        instances: &mut AgeSpriteInstances,
        frame: EffectFrame<'_, '_, '_>,
        meshes: &mut Assets<Mesh>,
        statistics: &mut Query<&mut SpriteEmitter>,
        materials: &mut Assets<ModelMaterial>,
    ) {
        if !frame.time.is_finite() {
            return;
        }
        let previous = instances.previous_time.unwrap_or(frame.time);
        let backwards = frame.time < previous || frame.change == EffectChange::Reset;
        let start = if backwards { frame.time } else { previous };
        let delta = frame.time - start;
        for (emitter, state) in self.emitters.iter().zip(&mut instances.emitters) {
            if let Some(animation) = &emitter.material_animation {
                animation.apply_to_assets(frame.sample, &state.material, materials);
            }
            let had_births = !state.births.is_empty();
            let node = frame.basis_world * frame.feature_matrix(emitter.node_index);
            if instances.previous_time.is_none() || backwards {
                state.births.clear();
                state.appearances.clear();
                state.fraction = 0.;
                state.ordinal = 0;
                state.squirts.initialized = false;
                state.previous_node = node;
                state.dropped = 0;
                state.skipped_seconds = 0.;
            }
            state.retire(frame.time);
            if frame.change == EffectChange::Seek {
                state.previous_node = node;
            }
            let follow = emitter.follow_displacement(state.previous_node, node, delta);
            if follow != Vec3::ZERO {
                for birth in &mut state.births {
                    birth.origin += follow;
                }
            }
            let inherited = if delta > 0. {
                (node.w_axis.truncate() - state.previous_node.w_axis.truncate()) / delta as f32
                    * emitter.inherit_velocity
            } else {
                Vec3::ZERO
            };
            let mut births_this_update = 0;
            let births = BirthContext {
                emitter,
                clips: &self.clips,
                frame,
                previous_basis: instances.previous_basis.unwrap_or(frame.basis_world),
                node,
                start,
                inherited,
                follow,
            };
            if let Some(programme) = &emitter.programmed {
                let include_origin =
                    !state.squirts.initialized && frame.change != EffectChange::Seek;
                state.squirts.collect(
                    &programme.squirt,
                    &frame,
                    &self.clips,
                    delta,
                    include_origin,
                );
            }
            let mut squirt_cursor = 0;
            let mut clock = EmissionClock::new(start, frame.time);
            for step in clock.by_ref() {
                let at = step.start;
                let end = step.end;
                let tick = step.tick;
                let rate_time = step.sample_time;
                let Some((clip, timeline, shared)) =
                    retimed_selection(&frame, rate_time, &self.clips)
                else {
                    continue;
                };
                let sample = frame.sample.at_times(clip, timeline, shared);
                let ended = frame
                    .selection
                    .clip
                    .and_then(|id| self.clips.get(&id))
                    .is_some_and(|(duration, looping)| !*looping && clip.as_seconds() >= *duration);
                let visible = if let Some(pose) = frame.pose {
                    pose.sample_feature_node(emitter.node_index, &sample, &mut state.pose_scratch)
                        .1
                } else {
                    frame
                        .node_visible
                        .get(emitter.node_index)
                        .copied()
                        .unwrap_or(true)
                };
                let active = if visible && !ended {
                    match &emitter.activation {
                        ParticleActivation::Boolean(value) => {
                            if sample.animated(value) {
                                1.
                            } else {
                                0.
                            }
                        }
                        ParticleActivation::Fractional(value) => {
                            sample.animated(value).clamp(0., 1.)
                        }
                        ParticleActivation::Programmed => 0.,
                    }
                } else {
                    0.
                };
                let base_rate = sample.animated(&emitter.rate).max(0.) * active;
                let mut rate_seed = emitter.seed ^ (tick as u64).wrapping_mul(0x517cc1b727220a95);
                let rate = (base_rate + signed(&mut rate_seed) * emitter.rate_variation * active)
                    .max(0.) as f64;
                if rate > 0. {
                    let prior = state.fraction;
                    let amount = prior + (end - at) * rate;
                    let count = amount.floor() as usize;
                    state.fraction = amount - count as f64;
                    let available = MAX_BIRTHS_PER_UPDATE
                        .saturating_sub(births_this_update)
                        .min(emitter.capacity().saturating_sub(state.births.len()));
                    let accepted = count.min(available);
                    let omitted = count - accepted;
                    state.dropped = state.dropped.saturating_add(omitted as u64);
                    for index in 0..accepted {
                        let time = at + (index as f64 + 1. - prior) / rate;
                        births.squirts_until(
                            state,
                            time,
                            &mut squirt_cursor,
                            &mut births_this_update,
                        );
                        births.push(state, time, &mut births_this_update);
                    }
                    state.ordinal = state.ordinal.wrapping_add(omitted as u64);
                }
            }
            state.skipped_seconds += clock.omitted_seconds();
            births.squirts_until(
                state,
                frame.time,
                &mut squirt_cursor,
                &mut births_this_update,
            );

            let node_changed = state.previous_node != node;
            state.previous_node = node;
            let redraw = backwards
                || delta != 0.
                || instances.previous_camera != frame.camera_world
                || instances.previous_basis != Some(frame.basis_world)
                || node_changed
                || (!had_births && !state.births.is_empty());
            if redraw && (had_births || !state.births.is_empty()) {
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

/// Common occurrence-pose evaluation for continuous and keyed emission.
struct BirthContext<'a, 'view, 'data> {
    emitter: &'a Emitter,
    clips: &'a HashMap<ClipId, (f64, bool)>,
    frame: EffectFrame<'a, 'view, 'data>,
    previous_basis: Mat4,
    node: Mat4,
    start: f64,
    inherited: Vec3,
    follow: Vec3,
}
impl BirthContext<'_, '_, '_> {
    fn squirts_until(
        &self,
        state: &mut EmitterInstance,
        end: f64,
        cursor: &mut usize,
        births: &mut usize,
    ) {
        while let Some(&(time, count)) = state.squirts.occurrences.get(*cursor) {
            if time > end {
                break;
            }
            *cursor += 1;
            let available = MAX_BIRTHS_PER_UPDATE
                .saturating_sub(*births)
                .min(self.emitter.capacity().saturating_sub(state.births.len()));
            let accepted = usize::from(count).min(available);
            let omitted = usize::from(count) - accepted;
            state.dropped = state.dropped.saturating_add(omitted as u64);
            for _ in 0..accepted {
                self.push(state, time, births);
            }
            state.ordinal = state.ordinal.wrapping_add(omitted as u64);
        }
    }
    fn push(&self, state: &mut EmitterInstance, time: f64, births: &mut usize) {
        let frame = &self.frame;
        let emitter = self.emitter;
        let delta = frame.time - self.start;
        let fraction = if delta > 0. {
            ((time - self.start) / delta) as f32
        } else {
            1.
        };
        let Some((clip, timeline, shared)) = retimed_selection(frame, time, self.clips) else {
            return;
        };
        if frame
            .selection
            .clip
            .and_then(|id| self.clips.get(&id))
            .is_some_and(|(duration, looping)| !*looping && clip.as_seconds() >= *duration)
        {
            return;
        }
        let sample = frame.sample.at_times(clip, timeline, shared);
        let birth_node = if let Some(pose) = frame.pose {
            let (authored, visible) = pose.at_offset(time - frame.time).sample_feature_node(
                emitter.node_index,
                &sample,
                &mut state.pose_scratch,
            );
            if !visible || !frame.root_visible {
                return;
            }
            interpolated_transform(self.previous_basis, frame.basis_world, fraction) * authored
        } else {
            if !frame.root_visible
                || !frame
                    .node_visible
                    .get(emitter.node_index)
                    .copied()
                    .unwrap_or(true)
            {
                return;
            }
            interpolated_transform(state.previous_node, self.node, fraction)
        };
        let ordinal = state.ordinal;
        state.ordinal = state.ordinal.wrapping_add(1);
        if state.births.len() >= emitter.capacity() || *births >= MAX_BIRTHS_PER_UPDATE {
            state.dropped = state.dropped.saturating_add(1);
            return;
        }
        *births += 1;
        if let Some((mut birth, appearance)) = emitter.birth_in(
            &sample,
            time,
            birth_node,
            frame.basis_world,
            self.inherited,
            ordinal,
        ) {
            if frame.time - birth.time < f64::from(birth.lifetime) {
                // Newborns follow only emitter translation after occurrence.
                birth.origin += self.follow * (1. - fraction.clamp(0., 1.));
                if let Some(appearance) = appearance {
                    birth.appearance_index = state.appearances.len() as u32;
                    state.appearances.push(appearance);
                }
                state.births.push(birth);
            }
        }
    }
}

fn interpolated_transform(a: Mat4, b: Mat4, t: f32) -> Mat4 {
    let (sa, ra, ta) = a.to_scale_rotation_translation();
    let (sb, rb, tb) = b.to_scale_rotation_translation();
    Mat4::from_scale_rotation_translation(sa.lerp(sb, t), ra.slerp(rb, t), ta.lerp(tb, t))
}
impl Emitter {
    /// Native follow pairs describe the fraction of emitter translation at two
    /// emitter speeds. Rotation remains detached for world-space particles.
    fn follow_displacement(&self, previous: Mat4, current: Mat4, delta: f64) -> Vec3 {
        if self.flags.model_space()
            || (self.follow[0][1] == 0. && self.follow[1][1] == 0.)
            || delta <= 0.
            || !delta.is_finite()
        {
            return Vec3::ZERO;
        }
        let displacement = current.w_axis.truncate() - previous.w_axis.truncate();
        let speed = displacement.length() / delta as f32;
        let [[speed_a, scale_a], [speed_b, scale_b]] = self.follow;
        let scale = if speed_a == speed_b {
            // Equal knots describe a constant rather than an undefined slope.
            scale_a
        } else {
            let fraction = ((speed - speed_a) / (speed_b - speed_a)).clamp(0., 1.);
            scale_a + (scale_b - scale_a) * fraction
        };
        displacement * scale
    }
    fn capacity(&self) -> usize {
        self.programmed
            .as_ref()
            .map_or(MAX_PARTICLES, |programme| programme.max_particles)
    }
    fn programmed(
        feature: FeatureId,
        node: NodeId,
        node_index: usize,
        programme: ProgrammedSprite,
        material: ModelMaterial,
        animation: Arc<SurfaceAnimation>,
    ) -> Self {
        let mut flags = ParticleFlags::default();
        flags.set_billboard(true);
        flags.set_model_space(programme.model_space);
        Self {
            feature,
            node,
            node_index,
            rate: programme.rate.clone(),
            rate_variation: 0.,
            activation: ParticleActivation::Boolean(Animated::constant(true)),
            speed: programme.speed.clone(),
            speed_variation: Animated::constant(0.),
            gravity: ParticleGravity::Vector(Animated::constant([0., 0., programme.gravity])),
            lifespan: programme.lifetime.clone(),
            lifespan_variation: Animated::constant(0.),
            vertical: Animated::constant(0.),
            horizontal: Animated::constant(0.),
            width: Animated::constant(0.),
            length: Animated::constant(0.),
            z_source: Animated::constant(0.),
            generator: ParticleGenerator::Plane,
            flags,
            drag: 0.,
            wind: Vec3::ZERO,
            inherit_velocity: 0.,
            follow: [[0.; 2]; 2],
            spin: [0.; 4],
            appearance: ParticleAgeCurves::default(),
            atlas: programme.atlas,
            texture_scale: 1.,
            head_tail: ParticleHeadTail::Head,
            tail_length: 0.,
            seed: node_index as u64 + 1,
            material,
            material_animation: animation.is_animated().then_some(animation),
            programmed: Some(programme),
        }
    }
    #[cfg(test)]
    fn birth(
        &self,
        sample: &RetimedAnimation<'_, '_, '_>,
        time: f64,
        node: Mat4,
        inherited: Vec3,
        ordinal: u64,
    ) -> Option<Birth> {
        self.birth_in(sample, time, node, node, inherited, ordinal)
            .map(|(birth, _)| birth)
    }
    fn birth_in(
        &self,
        sample: &RetimedAnimation<'_, '_, '_>,
        time: f64,
        node: Mat4,
        basis: Mat4,
        inherited: Vec3,
        ordinal: u64,
    ) -> Option<(Birth, Option<ProgrammedAppearance>)> {
        if !node.is_finite() || node.determinant() == 0. {
            return None;
        }
        let enabled = match &self.activation {
            ParticleActivation::Boolean(value) => sample.animated(value),
            ParticleActivation::Fractional(value) => sample.animated(value) > 0.,
            ParticleActivation::Programmed => false,
        };
        if !enabled {
            return None;
        }
        let mut seed = self.seed ^ ordinal.wrapping_mul(0xd6e8feb86659fd93);
        if let Some(programme) = &self.programmed {
            let random = from_fn(|_| random(&mut seed));
            let birth = programme.birth(sample, node, basis, random)?;
            return Some((
                Birth {
                    time,
                    lifetime: birth.lifetime,
                    origin: birth.origin,
                    velocity: birth.velocity,
                    acceleration: birth.acceleration,
                    spin: 0.,
                    spin_rate: 0.,
                    scale_variation: [1.; 2],
                    random_cell: 0,
                    size_scale: [
                        node.x_axis.truncate().length(),
                        node.y_axis.truncate().length(),
                    ],
                    appearance_index: u32::MAX,
                },
                Some(birth.appearance),
            ));
        }
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
        Some((
            Birth {
                appearance_index: u32::MAX,
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
            },
            None,
        ))
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
        frame: &EffectFrame<'_, '_, '_>,
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
        let mut buffers = SpriteMeshBuffers {
            positions: take_vec3(mesh, Mesh::ATTRIBUTE_POSITION),
            normals: take_vec3(mesh, Mesh::ATTRIBUTE_NORMAL),
            uv: take_vec2(mesh, Mesh::ATTRIBUTE_UV_0),
            colors: take_vec4(mesh, Mesh::ATTRIBUTE_COLOR),
        };
        for &(index, _) in &state.order {
            let birth = &state.births[index];
            let age = ((frame.time - birth.time) / f64::from(birth.lifetime)).clamp(0., 1.);
            let style = state
                .appearances
                .get(birth.appearance_index as usize)
                .map(|appearance| appearance.sample(age as f32));
            let age = ModelTime::seconds(age).unwrap();
            let rgb = style
                .map(|style| [style.color[0], style.color[1], style.color[2]])
                .unwrap_or_else(|| {
                    self.appearance
                        .color
                        .as_ref()
                        .and_then(|v| v.sample(age))
                        .unwrap_or([1.; 3])
                });
            let alpha = style.map(|style| style.color[3]).unwrap_or_else(|| {
                self.appearance
                    .alpha
                    .as_ref()
                    .and_then(|v| v.sample(age))
                    .unwrap_or(1.)
                    .clamp(0., 1.)
            });
            let mut size = style.map(|style| style.size).unwrap_or_else(|| {
                self.appearance
                    .scale
                    .as_ref()
                    .and_then(|v| v.sample(age))
                    .unwrap_or([1.; 2])
            });
            for (size, variation) in size.iter_mut().zip(birth.scale_variation) {
                *size *= variation;
            }
            let (mut position, mut velocity) = self.motion(birth, frame.time);
            if self.flags.model_space() {
                position = node.transform_point3(position);
                velocity = node.transform_vector3(velocity);
            }
            let spin = style.map_or_else(
                || birth.spin + birth.spin_rate * (frame.time - birth.time) as f32,
                |style| style.rotation,
            );
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
            let head = style.map(|style| style.cell).unwrap_or_else(|| {
                self.appearance
                    .head_uv
                    .as_ref()
                    .and_then(|v| v.sample(age))
                    .unwrap_or(0) as u32
                    + birth.random_cell
            });
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
                buffers.quad(
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
                buffers.quad(
                    [end - side, end + side, position + side, position - side],
                    tail,
                    self.atlas,
                    self.texture_scale,
                    [rgb[0], rgb[1], rgb[2], alpha],
                    inverse_basis,
                );
            }
        }
        // Bevy 0.19 skips allocation for zero-byte vertex buffers but still
        // attempts their upload. Keep a zero-area, transparent sentinel so an
        // emitter remains a valid resident mesh before births and after expiry.
        let SpriteMeshBuffers {
            mut positions,
            mut normals,
            mut uv,
            mut colors,
        } = buffers;
        if positions.is_empty() {
            positions.resize(3, [0.; 3]);
            normals.resize(3, [0., 0., 1.]);
            uv.resize(3, [0.; 2]);
            colors.resize(3, [0.; 4]);
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
struct SpriteMeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
}
impl SpriteMeshBuffers {
    fn quad(
        &mut self,
        corners: [Vec3; 4],
        cell: u32,
        atlas: [u16; 2],
        scale: f32,
        color: [f32; 4],
        inverse: Mat4,
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
            self.positions.push(transformed[index].to_array());
            self.normals.push(normal);
            self.uv.push(coordinates[index].to_array());
            self.colors.push(color);
        }
    }
}

#[cfg(test)]
#[path = "age_sprites_tests.rs"]
mod tests;
