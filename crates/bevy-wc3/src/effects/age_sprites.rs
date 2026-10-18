//! Deterministic M2 age-driven sprite simulation, independent of native codecs.
use super::super::material::{particle_material, ModelMaterial, SurfaceAnimation, SurfacePass};
use super::age_appearance::{AgeAppearance, AgeParticleAppearance};
use super::atlas_sprites::retimed_selection;
use super::emission::{EffectChange, EmissionClock};
use super::frame::EffectFrame;
use crate::material::{particle_material_layers, ParticleComposition};
use crate::mesh_attributes::EXTRA_TEXCOORD;
use crate::textures::{TextureImages, TextureRole};
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use std::array::from_fn;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use std::sync::Arc;
use wc3::model::values::model::{
    ParticleBlend, ParticleFlags, ParticleGenerator, ParticleHeadTail,
};
use wc3::model::{
    ClipId, Curve, CurveValue, FeatureId, M2ParticleAppearance, MaterialId, Model, NodeFeature,
    NodeId, ParticleEmitter, Property, Texture, Time as ModelTime, ValueSample,
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
struct AgeDrivenControls {
    activation: Property<bool>,
    speed_variation: Property<f32>,
    gravity: Property<[f32; 3]>,
    lifespan_variation: f32,
    vertical: Property<f32>,
    horizontal: Property<f32>,
    width: Property<f32>,
    length: Property<f32>,
    z_source: Property<f32>,
}

#[derive(Clone)]
struct Emitter {
    feature: FeatureId,
    node: NodeId,
    node_index: usize,
    rate: Property<f32>,
    rate_variation: f32,
    controls: Option<AgeDrivenControls>,
    speed: Property<f32>,
    lifespan: Property<f32>,
    generator: ParticleGenerator,
    flags: ParticleFlags,
    drag: f32,
    wind: Vec3,
    inherit_velocity: f32,
    follow: [[f32; 2]; 2],
    spin: [f32; 4],
    appearance: M2ParticleAppearance<'static>,
    atlas: [u16; 2],
    age_appearance: Option<AgeAppearance>,
    multi_texture: bool,
    head_tail: ParticleHeadTail,
    tail_length: f32,
    seed: u64,
    passes: Vec<MaterialPass>,
}
#[derive(Clone)]
struct MaterialPass {
    material: ModelMaterial,
    animation: Option<Arc<SurfaceAnimation>>,
}
#[derive(Clone, Default)]
pub(super) struct RuntimeAgeSprites {
    clips: HashMap<ClipId, (f64, bool)>,
    emitters: Vec<Emitter>,
    diagnostics: Vec<(FeatureId, String)>,
}

enum BirthAppearance {
    Age(AgeParticleAppearance),
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
    mesh: Handle<Mesh>,
    passes: Vec<MaterialPassInstance>,
    births: Vec<Birth>,

    age_appearances: Vec<AgeParticleAppearance>,
    order: Vec<(usize, f32)>,
    fraction: f64,
    ordinal: u64,
    previous_node: Mat4,
    pose_scratch: Vec<usize>,
    dropped: u64,
    skipped_seconds: f64,
}
struct MaterialPassInstance {
    entity: Entity,
    material: Handle<ModelMaterial>,
}
impl EmitterInstance {
    fn retire(&mut self, now: f64) {
        let mut retained = 0;
        let age_appearances = &mut self.age_appearances;
        self.births.retain_mut(|birth| {
            if now - birth.time >= f64::from(birth.lifetime) {
                return false;
            }
            if birth.appearance_index != u32::MAX {
                let index = birth.appearance_index as usize;
                if index != retained {
                    age_appearances.swap(index, retained);
                }
                birth.appearance_index = retained as u32;
                retained += 1;
            }
            true
        });
        age_appearances.truncate(retained);
    }
}
#[derive(Default)]
pub(super) struct AgeSpriteInstances {
    emitters: Vec<EmitterInstance>,
    previous_time: Option<f64>,
    previous_camera: Option<Mat4>,
    previous_basis: Option<Mat4>,
}

fn particle_image<'a>(
    _model: &Model<'_>,
    images: &'a TextureImages,
    id: Texture,
) -> Option<&'a Handle<Image>> {
    let texture = id;
    images.get_with_sampler(id, texture.color_space, TextureRole::Color, texture.sampler)
}

fn owned_curve<T: CurveValue>(value: &Option<Curve<'_, T>>) -> Option<Curve<'static, T>> {
    value.clone().map(Curve::into_owned)
}
fn owned_age(value: &M2ParticleAppearance<'_>) -> M2ParticleAppearance<'static> {
    M2ParticleAppearance {
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
        images: &TextureImages,
        _materials: &HashMap<MaterialId, Vec<SurfacePass>>,
    ) -> Self {
        let mut result = Self {
            clips: model
                .clips()
                .iter()
                .map(|(id, c)| (id, (c.duration().as_seconds(), c.is_looping())))
                .collect(),
            ..Self::default()
        };
        for (feature, value) in model.features() {
            let NodeFeature::Particle(ParticleEmitter::M2(definition)) = &value.payload else {
                continue;
            };

            let details = &definition.details;
            let mut diagnostic =
                |message: &str| result.diagnostics.push((feature, message.to_string()));
            if details.rendering.head_tail == ParticleHeadTail::Disabled {
                continue;
            }
            let appearance = &definition.appearance;
            if !matches!(
                details.shape.generator,
                ParticleGenerator::Plane | ParticleGenerator::Sphere
            ) {
                diagnostic(
                    "M2 spline, bone and unknown generators require a dedicated emission interpreter",
                );
                continue;
            }
            let (rate, speed, speed_variation, lifespan, lifespan_variation) = (
                definition.emission.rate,
                definition.motion.speed,
                definition.motion.variation,
                definition.lifetime.duration,
                definition.lifetime.variation,
            );
            if details.rendering.model.is_some() || details.rendering.child_model.is_some() {
                diagnostic(
                    "M2 model particles and child emitter resources are not instantiated by the sprite transport",
                );
                continue;
            }
            let basic_program = details
                .rendering
                .texture_program
                .is_none_or(|hints| hints == [0, 0]);
            let multi_texture = details.flags.multi_texture()
                && details.rendering.textures.len() == 3
                && basic_program;
            if !basic_program {
                diagnostic(
                    "M2 particle texture programme requires an advanced UV/shader interpreter; displaying its primary input",
                );
            }
            if details.rendering.textures.len() > 1 && !multi_texture {
                diagnostic(
                    "M2 extra particle textures require the three-input multi-texture programme",
                );
            }
            if details.flags.multi_texture_modulate4x() {
                diagnostic("M2 multi-texture modulation-gain control remains unresolved");
            }
            if let ParticleBlend::Unknown(_) = details.rendering.blend {
                diagnostic(
                    "Unknown M2 particle blending uses an explicit alpha-blend approximation",
                );
            }
            if details.wind_time != 0. {
                diagnostic("M2 wind timing is retained but not evaluated");
            }
            if appearance.legacy_atlas.is_some() {
                diagnostic(
                    "Legacy particle atlas segments currently use normalized-age cell curves when present",
                );
            }
            if !matches!(details.rendering.head_tail, ParticleHeadTail::Head) {
                diagnostic(
                    "M2 tails use velocity times the authored tail parameter; exact client time-versus-distance fidelity remains unverified",
                );
            }
            diagnostic(
                "M2 particle motion interpolates application root transforms between updates; inherited velocity uses a frame-interval estimate",
            );
            diagnostic(
                "CPU sprite transport limits each emitter to 8192 living particles, 8192 births and 4096 emission intervals per update; omitted births and long-update seconds are reported",
            );
            let image =
                details.rendering.textures.first().and_then(|id| {
                    particle_image(model, &images.particle(feature, 0), *id).cloned()
                });
            if image.is_none() && !details.rendering.textures.is_empty() {
                diagnostic(
                    "Unresolved primary particle image uses a visible white sprite fallback",
                );
            }
            let supported_flags = (1 << 0)
                | (1 << 2)
                | (1 << 4)
                | (1 << 5)
                | (1 << 8)
                | (1 << 6)
                | (1 << 10)
                | (1 << 12)
                | (1 << 16)
                | (1 << 17)
                | (1 << 18)
                | (1 << 19)
                | (1 << 23)
                | (1 << 28)
                | (1 << 30);
            if details.flags.bits() & !supported_flags != 0 {
                diagnostic(&format!(
                    "M2 particle controls {:#010x} are retained but not evaluated by the sprite transport",
                    details.flags.bits() & !supported_flags
                ));
            }

            result.emitters.push(Emitter {
                feature,

                node: value.node,
                node_index: node_indices[&value.node],
                rate,
                rate_variation: definition.emission.variation,
                speed,
                lifespan,
                controls: Some(AgeDrivenControls {
                    activation: definition.emission.activation,
                    speed_variation,
                    gravity: definition.motion.gravity,
                    lifespan_variation,
                    vertical: details.shape.vertical_range,
                    horizontal: details.shape.horizontal_range,
                    width: details.shape.area_width,
                    length: details.shape.area_length,
                    z_source: details.shape.z_source,
                }),
                generator: details.shape.generator,
                flags: details.flags,
                drag: details.drag.max(0.),
                wind: Vec3::from(details.wind),
                inherit_velocity: if details.flags.inherit_velocity() {
                    details.inherit_velocity
                } else {
                    0.
                },
                follow: details.follow,
                spin: details.spin,
                appearance: owned_age(appearance),
                atlas: details.rendering.atlas,
                age_appearance: Some({
                    let appearance = AgeAppearance::new(
                        details,
                        details.id.wrapping_add(result.emitters.len() as u32),
                    );
                    if multi_texture {
                        appearance
                    } else {
                        appearance.primary_only()
                    }
                }),
                multi_texture,
                head_tail: details.rendering.head_tail,
                tail_length: details.rendering.tail_length,
                seed: u64::from(details.id)
                    .wrapping_add(result.emitters.len() as u64)
                    .wrapping_add(1),
                passes: vec![MaterialPass {
                    animation: None,
                    material: {
                        let lit = !details.flags.disable_lighting();
                        let mut material = if multi_texture {
                            let layers = from_fn(|slot| {
                                details.rendering.textures.get(slot).and_then(|id| {
                                    particle_image(
                                        model,
                                        &images.particle(feature, slot as u32),
                                        *id,
                                    )
                                    .cloned()
                                })
                            });
                            particle_material_layers(
                                details.rendering.blend,
                                layers,
                                false,
                                if details.flags.multi_texture_three_colors() {
                                    ParticleComposition::ThreeColor
                                } else {
                                    ParticleComposition::TwoColor
                                },
                            )
                        } else {
                            particle_material(details.rendering.blend, image, lit)
                        };
                        material.base.depth_bias =
                            f32::from(details.rendering.texture_tile_rotation);
                        material
                    },
                }],
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
                if emitter.multi_texture {
                    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, vec![[0.; 2]; 3]);
                    mesh.insert_attribute(EXTRA_TEXCOORD, vec![[0.; 2]; 3]);
                }
                let mesh = meshes.add(mesh);
                let passes = emitter
                    .passes
                    .iter()
                    .enumerate()
                    .map(|(index, pass)| {
                        let material = materials.add(pass.material.clone());
                        let entity = commands
                            .spawn((
                                Mesh3d(mesh.clone()),
                                MeshMaterial3d(material.clone()),
                                Transform::default(),
                                Visibility::default(),
                                NoFrustumCulling,
                                ChildOf(basis),
                            ))
                            .id();
                        if index == 0 {
                            commands.entity(entity).insert(SpriteEmitter {
                                feature: emitter.feature,
                                node: emitter.node,
                                alive: 0,
                                dropped: 0,
                                skipped_seconds: 0.,
                            });
                        }
                        if !pass.material.extension.casts_shadows() {
                            commands.entity(entity).insert(NotShadowCaster);
                        }
                        if !pass.material.extension.receives_shadows() {
                            commands.entity(entity).insert(NotShadowReceiver);
                        }
                        MaterialPassInstance { entity, material }
                    })
                    .collect();
                EmitterInstance {
                    mesh,
                    passes,
                    births: Vec::new(),
                    age_appearances: Vec::new(),
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
            for (pass, instance) in emitter.passes.iter().zip(&state.passes) {
                if let Some(animation) = &pass.animation {
                    animation.apply_to_assets(frame.sample, &instance.material, materials);
                }
            }
            let had_births = !state.births.is_empty();
            let node =
                emitter.emitter_frame(frame.basis_world * frame.feature_matrix(emitter.node_index));
            if instances.previous_time.is_none() || backwards {
                state.births.clear();
                state.age_appearances.clear();
                state.fraction = 0.;
                state.ordinal = 0;
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

            let mut clock = EmissionClock::new(start, frame.time);
            for step in clock.by_ref() {
                let at = step.start;
                let end = step.end;
                let tick = step.tick;
                let rate_time = step.sample_time;
                let Some((clip, shared)) = retimed_selection(&frame, rate_time) else {
                    continue;
                };
                let sample = frame.sample.at_times(clip, shared);
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
                let controls = emitter.controls.as_ref().expect("M2 controls");
                let active = if visible && !ended && sample.animated(&controls.activation) {
                    1.
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
                        births.push(state, time, &mut births_this_update);
                    }
                    state.ordinal = state.ordinal.wrapping_add(omitted as u64);
                }
            }
            state.skipped_seconds += clock.omitted_seconds();

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
            if let Some(pass) = state.passes.first() {
                if let Ok(mut stats) = statistics.get_mut(pass.entity) {
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
    fn push(&self, state: &mut EmitterInstance, time: f64, births: &mut usize) {
        let frame = &self.frame;
        let emitter = self.emitter;
        let delta = frame.time - self.start;
        let fraction = if delta > 0. {
            ((time - self.start) / delta) as f32
        } else {
            1.
        };
        let birth_basis = interpolated_transform(self.previous_basis, frame.basis_world, fraction);
        let Some((clip, shared)) = retimed_selection(frame, time) else {
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
        let sample = frame.sample.at_times(clip, shared);
        let birth_node = if let Some(pose) = frame.pose {
            let (authored, visible) = pose.at_offset(time - frame.time).sample_feature_node(
                emitter.node_index,
                &sample,
                &mut state.pose_scratch,
            );
            if !visible || !frame.root_visible {
                return;
            }
            emitter.emitter_frame(birth_basis * authored)
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
            birth_basis,
            self.inherited,
            ordinal,
        ) {
            if frame.time - birth.time < f64::from(birth.lifetime) {
                // Newborns follow only emitter translation after occurrence.
                birth.origin += self.follow * (1. - fraction.clamp(0., 1.));
                if let Some(appearance) = appearance {
                    birth.appearance_index = match appearance {
                        BirthAppearance::Age(value) => {
                            let index = state.age_appearances.len();
                            state.age_appearances.push(value);
                            index as u32
                        }
                    };
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
    fn emitter_frame(&self, node: Mat4) -> Mat4 {
        // M2 emitter X/Y axes are rotated relative to the carrying bone.
        node * Mat4::from_rotation_z(FRAC_PI_2)
    }
    fn particle_scale(&self, node: Mat4, basis: Mat4) -> [f32; 2] {
        let inherited = if self.flags.inherit_bone_scale() {
            (basis.inverse() * node).x_axis.truncate().length()
        } else {
            1.
        };
        [
            basis.x_axis.truncate().length() * inherited,
            basis.y_axis.truncate().length() * inherited,
        ]
    }
    /// Native follow pairs describe the fraction of emitter translation at two
    /// emitter speeds. Rotation remains detached for world-space particles.
    fn follow_displacement(&self, previous: Mat4, current: Mat4, delta: f64) -> Vec3 {
        if self.flags.attached()
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
        MAX_PARTICLES
    }

    #[cfg(test)]
    fn birth(
        &self,
        sample: &impl ValueSample,
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
        sample: &impl ValueSample,
        time: f64,
        node: Mat4,
        basis: Mat4,
        inherited: Vec3,
        ordinal: u64,
    ) -> Option<(Birth, Option<BirthAppearance>)> {
        if !node.is_finite() || node.determinant() == 0. {
            return None;
        }
        let mut seed = self.seed ^ ordinal.wrapping_mul(0xd6e8feb86659fd93);

        let controls = self.controls.as_ref().expect("age-driven emitter controls");
        let enabled = sample.animated(&controls.activation);
        if !enabled {
            return None;
        }
        let lifetime =
            sample.animated(&self.lifespan) + signed(&mut seed) * controls.lifespan_variation;
        if !lifetime.is_finite() || lifetime <= 0. {
            return None;
        }
        let speed = sample.animated(&self.speed)
            * (1. + signed(&mut seed) * sample.animated(&controls.speed_variation));
        let vertical = sample.animated(&controls.vertical);
        let horizontal = sample.animated(&controls.horizontal);
        let theta = signed(&mut seed) * vertical;
        let phi = signed(&mut seed) * horizontal;
        let mut direction = if self.generator == ParticleGenerator::Sphere {
            Vec3::new(
                theta.cos() * phi.cos(),
                theta.cos() * phi.sin(),
                theta.sin(),
            )
        } else {
            Vec3::new(
                theta.sin() * phi.cos(),
                theta.sin() * phi.sin(),
                theta.cos(),
            )
        };
        let width = sample.animated(&controls.width);
        let length = sample.animated(&controls.length);
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
        let z_source = sample.animated(&controls.z_source);
        let aimed = if self.generator == ParticleGenerator::Plane {
            z_source > 0.001
        } else {
            z_source != 0.
        };
        if aimed {
            direction = (origin - Vec3::Z * z_source)
                .try_normalize()
                .unwrap_or(direction);
        }
        if !aimed && self.generator == ParticleGenerator::Sphere && self.flags.sphere_up() {
            direction = Vec3::Z;
        }
        let gravity = Vec3::from(sample.animated(&controls.gravity));
        let local = self.flags.attached();
        let (origin, mut velocity, acceleration) = if local {
            (origin, direction * speed, gravity + self.wind)
        } else {
            (
                node.transform_point3(origin),
                node.transform_vector3(direction * speed),
                basis.transform_vector3(gravity + self.wind),
            )
        };
        velocity += if local {
            node.inverse().transform_vector3(inherited)
        } else {
            inherited
        };
        let x_variation = (1. + signed(&mut seed) * self.appearance.scale_variation[0]).max(0.);
        let y_variation = if self.flags.independent_scale() {
            (1. + signed(&mut seed) * self.appearance.scale_variation[1]).max(0.)
        } else {
            x_variation
        };
        let scale_variation = [x_variation, y_variation];
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
                size_scale: self.particle_scale(node, basis),
                random_cell: if self.flags.random_texture() {
                    (random(&mut seed)
                        * f32::from(self.atlas[0].max(1))
                        * f32::from(self.atlas[1].max(1))) as u32
                } else {
                    0
                },
            },
            self.age_appearance
                .as_ref()
                .filter(|appearance| appearance.needs_particle_state())
                .map(|appearance| BirthAppearance::Age(appearance.birth(seed as u32))),
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
                if self.flags.attached() {
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
            uv_secondary: self
                .multi_texture
                .then(|| take_vec2(mesh, Mesh::ATTRIBUTE_UV_1)),
            uv_tertiary: self.multi_texture.then(|| take_vec2(mesh, EXTRA_TEXCOORD)),
            colors: take_vec4(mesh, Mesh::ATTRIBUTE_COLOR),
        };
        for &(index, _) in &state.order {
            let birth = &state.births[index];
            let age = ((frame.time - birth.time) / f64::from(birth.lifetime)).clamp(0., 1.);
            let native_style = self
                .age_appearance
                .as_ref()
                .map(|appearance| {
                    appearance.sample(
                        &state
                            .age_appearances
                            .get(birth.appearance_index as usize)
                            .copied()
                            .unwrap_or_default(),
                        (frame.time - birth.time).max(0.) as f32,
                    )
                })
                .unwrap_or_default();
            if !native_style.visible {
                continue;
            }
            let extra_uv = self.multi_texture.then(|| {
                (
                    native_style.offsets,
                    self.age_appearance.as_ref().unwrap().texture_scales(),
                )
            });
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
                *size *= variation * native_style.scale;
            }
            let (mut position, mut velocity) = self.motion(birth, frame.time);
            if self.flags.attached() {
                position = node.transform_point3(position);
                velocity = node.transform_vector3(velocity);
            }
            let spin = birth.spin + birth.spin_rate * (frame.time - birth.time) as f32;
            let (sin, cos) = spin.sin_cos();
            let (r, u) = if self.flags.align_xy() {
                let axes = if self.flags.attached() {
                    node
                } else {
                    frame.basis_world
                };
                (
                    axes.x_axis.truncate().normalize_or_zero(),
                    axes.y_axis.truncate().normalize_or_zero(),
                )
            } else if self.flags.velocity_oriented() {
                let projected = velocity - camera_back * velocity.dot(camera_back);
                let vertical = projected.try_normalize().unwrap_or(up);
                (vertical.cross(camera_back).normalize_or_zero(), vertical)
            } else {
                (right, up)
            };
            let scale = if self.flags.attached() {
                self.particle_scale(node, frame.basis_world)
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
                buffers.quad(
                    [
                        position - r - u,
                        position + r - u,
                        position + r + u,
                        position - r + u,
                    ],
                    head,
                    self.atlas,
                    extra_uv,
                    [rgb[0], rgb[1], rgb[2], alpha],
                    inverse_basis,
                );
            }
            if matches!(
                self.head_tail,
                ParticleHeadTail::Tail | ParticleHeadTail::Both
            ) {
                let tail_time = if self.flags.clamp_tail_to_age() {
                    self.tail_length
                        .min((frame.time - birth.time).max(0.) as f32)
                } else {
                    self.tail_length
                };
                let end = position - velocity * tail_time;
                let side = (position - end)
                    .cross(camera_position - position)
                    .normalize_or_zero()
                    * size[0]
                    * scale[0]
                    * 0.5;
                buffers.quad(
                    [end - side, end + side, position + side, position - side],
                    tail,
                    self.atlas,
                    extra_uv,
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
            mut uv_secondary,
            mut uv_tertiary,
            mut colors,
        } = buffers;
        if positions.is_empty() {
            positions.resize(3, [0.; 3]);
            normals.resize(3, [0., 0., 1.]);
            uv.resize(3, [0.; 2]);
            if let Some(uv) = &mut uv_secondary {
                uv.resize(3, [0.; 2]);
            }
            if let Some(uv) = &mut uv_tertiary {
                uv.resize(3, [0.; 2]);
            }
            colors.resize(3, [0.; 4]);
        }
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        if let Some(uv) = uv_secondary {
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv);
        }
        if let Some(uv) = uv_tertiary {
            mesh.insert_attribute(EXTRA_TEXCOORD, uv);
        }
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
    uv_secondary: Option<Vec<[f32; 2]>>,
    uv_tertiary: Option<Vec<[f32; 2]>>,
    colors: Vec<[f32; 4]>,
}
impl SpriteMeshBuffers {
    fn quad(
        &mut self,
        corners: [Vec3; 4],
        cell: u32,
        atlas: [u16; 2],
        extra_uv: Option<([Vec2; 2], [f32; 2])>,
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
        let extent = Vec2::new(1. / columns as f32, 1. / rows as f32);
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
            if let Some((offsets, scales)) = extra_uv {
                let corner = [Vec2::ZERO, Vec2::X, Vec2::ONE, Vec2::Y][index];
                if let Some(uv) = &mut self.uv_secondary {
                    uv.push((offsets[0] + corner * scales[0]).to_array());
                }
                if let Some(uv) = &mut self.uv_tertiary {
                    uv.push((offsets[1] + corner * scales[1]).to_array());
                }
            }
            self.colors.push(color);
        }
    }
}

#[cfg(test)]
#[path = "age_sprites_tests.rs"]
mod tests;
