//! Shared M2 particle motion and sprite/model transports, independent of native codecs.
use super::super::material::SurfacePass;
use super::age_appearance::{AgeAppearance, AgeParticleAppearance};
use super::age_sprites_gpu::{
    AgeSpriteComposition, AgeSpriteData, AgeSpriteRecord, AgeSpriteStyle, AgeSpriteUniform,
    SampleBuffers,
};
use super::atlas_sprites::retimed_selection;
use super::bones::{BoneCandidate, BoneSource};
use super::emission::{EffectChange, EmissionClock};
use super::frame::EffectFrame;
use super::frame::EffectStart;
use super::generators::spline;
use super::implosion::{bounded_end, outward_crossing};
use super::m2_model_geometry::{tumble, ModelBasis};
use super::m2_models::{
    ModelChildren, ModelOutputs, ModelProjection, ModelProjectionContext, ModelTemplate,
};
use super::model_budget::{ActorFrame, ModelActorLease};
use super::render::SortPolicy;
use super::ModelEmitter;
use crate::child_coordinates::ChildCoordinates;
use crate::child_models::{ChildModelBinding, ChildModelKey};
use crate::diagnostics::{RenderDiagnosticKind, RenderIssue};
use crate::effects::render::{effect_quad, EffectGeometry};
use crate::layers::ModelOwnedRenderEntity;
#[cfg(test)]
use crate::mesh_attributes::EXTRA_TEXCOORD;
use crate::textures::{TextureImages, TextureRole};
#[cfg(test)]
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::light::NotShadowCaster;
use bevy::math::{Vec2, Vec3};
#[cfg(test)]
use bevy::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
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
// A living particle can emit one head and one tail, each with four corners.
const _: () = assert!(MAX_PARTICLES * 8 <= u16::MAX as usize + 1);

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
    /// Time omitted from emission or sphere-retirement checks by work budgets.
    /// Overlapping omitted intervals are counted once; clocks still advance.
    pub skipped_seconds: f64,
}

#[derive(Clone)]
struct AgeDrivenControls {
    activation: Property<bool>,
    speed_variation: Property<f32>,
    gravity: Property<Vec3>,
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
    spline: Vec<Vec3>,
    bones: Option<BoneSource>,
    flags: ParticleFlags,
    model: Option<GeometryEmitter>,
    drag: f32,
    wind: Vec3,
    inherit_velocity: f32,
    follow: [Vec2; 2],
    spin: [f32; 4],
    appearance: M2ParticleAppearance<'static>,
    atlas: [u16; 2],
    age_appearance: Option<AgeAppearance>,
    #[cfg(test)]
    multi_texture: bool,
    #[cfg(test)]
    head_tail: ParticleHeadTail,
    tail_length: f32,
    seed: u64,
    gpu_style: AgeSpriteStyle,
}
#[derive(Clone, Default)]
pub(super) struct RuntimeM2Particles {
    clips: HashMap<ClipId, (f64, bool)>,
    emitters: Vec<Emitter>,
    diagnostics: Vec<(FeatureId, RenderIssue)>,
}

enum BirthAppearance {
    Age(AgeParticleAppearance),
}
#[derive(Clone)]
struct GeometryEmitter {
    template: ModelTemplate,
    minimum: Vec3,
    maximum: Vec3,
}
struct GeometryBirth {
    lease: Option<ModelActorLease>,
    initial_transform: Transform,
    ordinal: u64,
    basis: ModelBasis,
    omega: Vec3,
    appearance: AgeParticleAppearance,
}
#[derive(Default)]
struct GeometryInstances {
    births: Vec<GeometryBirth>,
    children: ModelChildren,
    projections: Vec<ModelProjection>,
}
pub(crate) struct GeometryWorld<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub outputs: &'a mut ModelOutputs<'w, 's>,
    pub statistics: &'a mut Query<'w, 's, &'static mut ModelEmitter>,
    pub root: Entity,
    pub driving_camera: Option<Entity>,
    pub allow_births: bool,
    pub actor: Option<&'a ActorFrame>,
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
    scale_variation: Vec2,
    random_cell: u32,
    size_scale: Vec2,
    appearance_index: u32,
}
struct EmitterInstance {
    samples: SampleBuffers,
    #[cfg(test)]
    mesh: Handle<Mesh>,
    passes: Vec<MaterialPassInstance>,
    births: Vec<Birth>,

    age_appearances: Vec<AgeParticleAppearance>,
    models: Option<GeometryInstances>,
    #[cfg(test)]
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
}
impl EmitterInstance {
    fn retire(&mut self, now: f64) {
        self.retain(|birth| now - birth.time < f64::from(birth.lifetime));
    }
    fn retain(&mut self, mut keep: impl FnMut(&Birth) -> bool) {
        let mut retained = 0;
        let age_appearances = &mut self.age_appearances;
        let mut model_births = self.models.as_mut().map(|state| &mut state.births);
        self.births.retain_mut(|birth| {
            if !keep(birth) {
                return false;
            }
            if birth.appearance_index != u32::MAX {
                let index = birth.appearance_index as usize;
                if index != retained {
                    if let Some(models) = model_births.as_mut() {
                        models.swap(index, retained);
                    } else {
                        age_appearances.swap(index, retained);
                    }
                }
                birth.appearance_index = retained as u32;
                retained += 1;
            }
            true
        });
        if let Some(models) = model_births {
            models.truncate(retained);
        } else {
            age_appearances.truncate(retained);
        }
    }
}
#[derive(Default)]
pub(super) struct M2ParticleInstances {
    emitters: Vec<EmitterInstance>,
    previous_time: Option<f64>,
    previous_basis: Option<Mat4>,
}

fn particle_image(
    images: &TextureImages,
    id: Texture,
    role: TextureRole,
) -> Option<&Handle<Image>> {
    images.get_with_sampler(id, id.color_space, role, id.sampler)
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

impl RuntimeM2Particles {
    pub(super) fn seed_owned(
        &self,
        instances: &mut M2ParticleInstances,
        start: &EffectStart<'_, '_, '_>,
    ) {
        if instances.previous_time.is_some() {
            return;
        }
        instances.previous_time = Some(0.);
        instances.previous_basis = Some(start.frame.basis_world);
        for (emitter, state) in self.emitters.iter().zip(&mut instances.emitters) {
            state.previous_node =
                emitter.emitter_frame(start.feature(emitter.node_index, &mut state.pose_scratch));
        }
    }

    pub(super) fn append_remapped(
        &mut self,
        source: &Self,
        indices: &[usize],
        nodes: &HashMap<NodeId, NodeId>,
        clips: &HashMap<ClipId, ClipId>,
    ) {
        let mut bone_remaps = HashMap::new();
        self.emitters
            .extend(source.emitters.iter().cloned().map(|mut value| {
                value.node = nodes[&value.node];
                value.node_index = indices[value.node_index];
                value.bones = value
                    .bones
                    .as_ref()
                    .map(|bones| bones.remap_cached(indices, &mut bone_remaps));

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

    #[cfg(test)]
    pub fn compile(
        model: &Model<'_>,
        nodes: &HashMap<NodeId, usize>,
        images: &TextureImages,
        materials: &HashMap<MaterialId, Vec<SurfacePass>>,
    ) -> Self {
        Self::compile_with_children(model, nodes, images, materials, &HashMap::new())
    }
    pub fn has_models(&self) -> bool {
        self.emitters.iter().any(|emitter| emitter.model.is_some())
    }
    pub fn compile_with_children(
        model: &Model<'_>,
        node_indices: &HashMap<NodeId, usize>,
        images: &TextureImages,
        _materials: &HashMap<MaterialId, Vec<SurfacePass>>,
        children: &HashMap<ChildModelKey, ChildModelBinding>,
    ) -> Self {
        let mut result = Self {
            clips: model
                .clips()
                .iter()
                .map(|(id, c)| (id, (c.duration().as_seconds(), c.is_looping())))
                .collect(),
            ..Self::default()
        };
        let mut bone_source = None;
        for (feature, value) in model.features() {
            let NodeFeature::Particle(ParticleEmitter::M2(definition)) = &value.payload else {
                continue;
            };

            let details = &definition.details;
            let mut diagnostic = |kind, message: &str| {
                result
                    .diagnostics
                    .push((feature, RenderIssue::new(kind, message)));
            };
            let appearance = &definition.appearance;
            if !matches!(
                details.shape.generator,
                ParticleGenerator::Plane
                    | ParticleGenerator::Sphere
                    | ParticleGenerator::Spline
                    | ParticleGenerator::Bone
            ) {
                diagnostic(
                    RenderDiagnosticKind::Unsupported,
                    "Unknown M2 particle generators require a dedicated emission interpreter",
                );
                continue;
            }
            let bones = (details.shape.generator == ParticleGenerator::Bone).then(|| {
                bone_source
                    .get_or_insert_with(|| BoneSource::compile(model, node_indices))
                    .clone()
            });
            if let Some(bones) = &bones {
                diagnostic(
                    RenderDiagnosticKind::Unverified,
                    "M2 body particles sample posed joint spans in model space and convert once to the emitter frame; native Bone/Joint distinction and client frame fidelity remain unverified",
                );
                if bones.inferred {
                    diagnostic(
                        RenderDiagnosticKind::Approximation,
                        "Body-particle eligibility without a declared section palette is inferred from positive drawn skin influences",
                    );
                }
                if bones.is_empty() {
                    diagnostic(
                        RenderDiagnosticKind::Resource,
                        "M2 body particles have no eligible first-LOD section palette joints; births are omitted",
                    );
                }
            }
            if details.shape.generator == ParticleGenerator::Spline {
                diagnostic(
                    RenderDiagnosticKind::Unverified,
                    "M2 spline particles sample the normalized authored path; exact client shape and frame fidelity remain unverified",
                );
            }
            if details.shape.generator == ParticleGenerator::Sphere
                && details.flags.implosion_filter()
            {
                diagnostic(
                    RenderDiagnosticKind::Approximation,
                    "M2 sphere implosion retirement samples analytic movement at bounded 60 Hz intervals against the interpolated live emitter center; discrete native integration is approximated",
                );
                diagnostic(
                    RenderDiagnosticKind::Budget,
                    "M2 sphere implosion filtering bounds each emitter to 32768 particle-steps per update; emission and retirement omissions share one skipped-time counter and do not create a catch-up queue",
                );
            }
            let (rate, speed, speed_variation, lifespan, lifespan_variation) = (
                definition.emission.rate,
                definition.motion.speed,
                definition.motion.variation,
                definition.lifetime.duration,
                definition.lifetime.variation,
            );
            let geometry = if details.rendering.model.is_some() {
                let Some(binding) = children.get(&ChildModelKey::particle_geometry(feature)) else {
                    diagnostic(
                        RenderDiagnosticKind::Resource,
                        "M2 geometry particle child model is unresolved; births are omitted",
                    );
                    continue;
                };
                diagnostic(RenderDiagnosticKind::Unverified,"M2 model particles own ordinary child animation and materials; exact native model size, tumble and nonuniform-frame fidelity remains unverified");
                diagnostic(RenderDiagnosticKind::Approximation,"Model-particle orientation uses finite ordinary Bevy TRS; shear is orthogonalized and collapsed birth frames are omitted");
                diagnostic(RenderDiagnosticKind::Budget,"Nested model particles and their static descendants share ModelParticleLimits on the outer application root; omitted admissions and emission intervals are not queued");
                Some(GeometryEmitter {
                    template: ModelTemplate {
                        model: binding.model.clone(),
                        coordinates: ChildCoordinates::from_model(model, binding.coordinates),
                    },
                    minimum: details.tumble_minimum,
                    maximum: details.tumble_maximum,
                })
            } else {
                None
            };
            if details.rendering.child_model.is_some() {
                diagnostic(
                    RenderDiagnosticKind::Unsupported,
                    "M2 child emitter resources require the adopted-emitter interpreter",
                );
                if geometry.is_none() {
                    continue;
                }
            }
            let multi_texture =
                details.flags.multi_texture() && details.rendering.textures.len() == 3;
            let refraction = details.flags.refraction() && !details.flags.multi_texture();
            let image_role = if refraction {
                TextureRole::Data
            } else {
                TextureRole::Color
            };
            if geometry.is_none() {
                if refraction {
                    diagnostic(RenderDiagnosticKind::Unverified, "M2 particle refraction uses a per-view scene mask; exact native displacement and mask overlap fidelity remain unverified");
                }
                if details
                    .rendering
                    .texture_program
                    .is_some_and(|hints| hints != [0, 0])
                {
                    diagnostic(
                    RenderDiagnosticKind::Unsupported,
                    "M2 particle TXAC selectors are retained without interpretation; recognized native composition flags remain active",
                );
                }
                if details.rendering.textures.len() > 1 && !multi_texture {
                    diagnostic(
                    RenderDiagnosticKind::Unsupported,
                    "M2 extra particle textures require the three-input multi-texture programme",
                );
                }
                if let ParticleBlend::Unknown(_) = details.rendering.blend {
                    diagnostic(
                        RenderDiagnosticKind::Approximation,
                        "Unknown M2 particle blending uses an explicit alpha-blend approximation",
                    );
                }
                if !refraction
                    && matches!(
                        details.rendering.blend,
                        ParticleBlend::Opaque | ParticleBlend::AlphaKey
                    )
                {
                    diagnostic(
                    RenderDiagnosticKind::Approximation,
                    "M2 opaque and cutout sprites write main-view depth but do not cast shadows or join depth, normal, and motion-vector prepasses",
                );
                }
            }
            if details.wind_time != 0. {
                diagnostic(
                    RenderDiagnosticKind::Unsupported,
                    "M2 wind timing is retained but not evaluated",
                );
            }
            if geometry.is_none() && appearance.legacy_atlas.is_some() {
                diagnostic(
                    RenderDiagnosticKind::Approximation,
                    "Legacy particle atlas segments currently use normalized-age cell curves when present",
                );
            }
            if geometry.is_none()
                && matches!(
                    details.rendering.head_tail,
                    ParticleHeadTail::Tail | ParticleHeadTail::Both
                )
            {
                diagnostic(
                    RenderDiagnosticKind::Unverified,
                    "M2 tails use velocity times the authored tail parameter; exact client time-versus-distance fidelity remains unverified",
                );
            }
            diagnostic(
                RenderDiagnosticKind::Approximation,
                "M2 particle motion interpolates application root transforms between updates; inherited velocity uses a frame-interval estimate",
            );
            diagnostic(
                RenderDiagnosticKind::Budget,
                if geometry.is_some() {
                    "Model simulation limits each emitter to 1024 living models, 8192 births and 4096 emission intervals per update; excess births are omitted"
                } else {
                    "Sprite simulation limits each emitter to 8192 living particles, 8192 births and 4096 emission intervals per update; omitted births and long-update seconds are reported"
                },
            );
            let image = details.rendering.textures.first().and_then(|id| {
                particle_image(&images.particle(feature, 0), *id, image_role).cloned()
            });
            if geometry.is_none() && image.is_none() && !details.rendering.textures.is_empty() {
                diagnostic(
                    RenderDiagnosticKind::Resource,
                    "Unresolved primary particle image uses a visible white sprite fallback",
                );
            }
            let supported_flags = (1 << 0)
                | (1 << 1)
                | (1 << 2)
                | (1 << 3)
                | (1 << 4)
                | (1 << 5)
                | (1 << 8)
                | (1 << 6)
                | (1 << 7)
                | (1 << 10)
                | (1 << 12)
                | (1 << 16)
                | (1 << 17)
                | (1 << 18)
                | (1 << 19)
                | (1 << 20)
                | (1 << 23)
                | if geometry.is_some() { 1 << 9 } else { 0 }
                | (1 << 28)
                | (1 << 29)
                | (1 << 30);
            if details.flags.bits() & !supported_flags != 0 {
                diagnostic(
                    RenderDiagnosticKind::Unsupported,
                    &format!(
                        "M2 particle controls {:#010x} are retained but not evaluated by the sprite transport",
                        details.flags.bits() & !supported_flags
                    ),
                );
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
                spline: details.shape.spline.iter().collect(),
                bones,
                flags: details.flags,
                model: geometry,
                drag: details.drag.max(0.),
                wind: details.wind,
                inherit_velocity: if details.flags.inherit_velocity() {
                    details.inherit_velocity
                } else {
                    0.
                },
                follow: details.follow,
                spin: (details.spin).to_array(),
                appearance: owned_age(appearance),
                atlas: details.rendering.atlas,
                age_appearance: Some({
                    let appearance = AgeAppearance::new(
                        details,
                        details.id.wrapping_add(result.emitters.len() as u32),
                    );
                    if multi_texture || refraction {
                        appearance
                    } else {
                        appearance.primary_only()
                    }
                }),
                #[cfg(test)]
                multi_texture: multi_texture || refraction,
                #[cfg(test)]
                head_tail: details.rendering.head_tail,
                tail_length: details.rendering.tail_length,
                seed: u64::from(details.id)
                    .wrapping_add(result.emitters.len() as u64)
                    .wrapping_add(1),
                gpu_style: AgeSpriteStyle {
                    textures: from_fn(|index| {
                        details.rendering.textures.get(index).and_then(|id| {
                            particle_image(&images.particle(feature, index as u32), *id, image_role)
                                .cloned()
                        })
                    }),
                    blend: details.rendering.blend,
                    priority: f32::from(details.rendering.texture_tile_rotation),
                    sort: if details.flags.sort_particles() {
                        SortPolicy::Far
                    } else {
                        SortPolicy::None
                    },
                    composition: if multi_texture {
                        AgeSpriteComposition::Multi {
                            three_colors: details.flags.multi_texture_three_colors(),
                            alpha_modulate4x: details.flags.multi_texture_modulate4x(),
                        }
                    } else if refraction {
                        AgeSpriteComposition::Refraction
                    } else {
                        AgeSpriteComposition::Single
                    },
                    uv_flip_v: false,
                    unlit: multi_texture || refraction || details.flags.disable_lighting(),
                    head: matches!(
                        details.rendering.head_tail,
                        ParticleHeadTail::Head | ParticleHeadTail::Both
                    ),
                    tail: matches!(
                        details.rendering.head_tail,
                        ParticleHeadTail::Tail | ParticleHeadTail::Both
                    ),
                },
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
        basis: Entity,
    ) -> M2ParticleInstances {
        let quad = self
            .emitters
            .iter()
            .any(|emitter| emitter.model.is_none())
            .then(|| effect_quad(meshes));
        let emitters = self
            .emitters
            .iter()
            .map(|emitter| {
                let mesh = quad.as_ref().cloned().unwrap_or_default();
                let entity = if emitter.model.is_some() {
                    commands
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
                            ChildOf(basis),
                        ))
                        .id()
                } else {
                    commands
                        .spawn((
                            EffectGeometry(mesh.clone()),
                            ModelOwnedRenderEntity,
                            AgeSpriteData {
                                records: Default::default(),
                                live_indices: Default::default(),
                                uniform: Default::default(),
                                style: emitter.gpu_style.clone(),
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
                            NotShadowCaster,
                            ChildOf(basis),
                        ))
                        .id()
                };
                let passes = vec![MaterialPassInstance { entity }];
                EmitterInstance {
                    samples: Default::default(),
                    #[cfg(test)]
                    mesh,
                    passes,
                    births: Vec::new(),
                    age_appearances: Vec::new(),
                    models: emitter.model.as_ref().map(|_| GeometryInstances::default()),
                    #[cfg(test)]
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
        M2ParticleInstances {
            emitters,
            previous_time: None,
            previous_basis: None,
        }
    }
    #[cfg(test)]
    pub fn update(
        &self,
        instances: &mut M2ParticleInstances,
        frame: EffectFrame<'_, '_, '_>,
        meshes: &mut Assets<Mesh>,
        gpu_sprites: &mut Query<&mut AgeSpriteData>,
        statistics: &mut Query<&mut SpriteEmitter>,
    ) {
        let had_births: Vec<_> = instances
            .emitters
            .iter()
            .map(|state| !state.births.is_empty())
            .collect();
        self.update_bounded(instances, frame, gpu_sprites, statistics, None);
        for ((emitter, state), had_births) in self
            .emitters
            .iter()
            .zip(&mut instances.emitters)
            .zip(had_births)
        {
            // The CPU oracle must preserve the resident sentinel on empty-to-empty
            // updates, while publishing it when the final live birth expires.
            if state.passes.is_empty() && (had_births || !state.births.is_empty()) {
                if let Some(mut mesh) = meshes.get_mut(&state.mesh) {
                    let node = emitter.emitter_frame(
                        frame.basis_world * frame.feature_matrix(emitter.node_index),
                    );
                    emitter.draw(state, &frame, node, &mut mesh, Mat4::IDENTITY);
                }
            }
        }
    }
    pub fn update_bounded(
        &self,
        instances: &mut M2ParticleInstances,
        frame: EffectFrame<'_, '_, '_>,
        gpu_sprites: &mut Query<&mut AgeSpriteData>,
        statistics: &mut Query<&mut SpriteEmitter>,
        actor: Option<&ActorFrame>,
    ) {
        self.update_inner(instances, frame, gpu_sprites, statistics, None, actor);
    }
    pub fn update_with_models(
        &self,
        instances: &mut M2ParticleInstances,
        frame: EffectFrame<'_, '_, '_>,
        gpu_sprites: &mut Query<&mut AgeSpriteData>,
        statistics: &mut Query<&mut SpriteEmitter>,
        model_world: Option<GeometryWorld>,
    ) {
        let actor = model_world.as_ref().and_then(|world| world.actor).cloned();
        self.update_inner(
            instances,
            frame,
            gpu_sprites,
            statistics,
            model_world,
            actor.as_ref(),
        );
    }
    fn update_inner(
        &self,
        instances: &mut M2ParticleInstances,
        frame: EffectFrame<'_, '_, '_>,
        gpu_sprites: &mut Query<&mut AgeSpriteData>,
        statistics: &mut Query<&mut SpriteEmitter>,
        mut model_world: Option<GeometryWorld>,
        actor: Option<&ActorFrame>,
    ) {
        if !frame.time.is_finite() {
            return;
        }
        let previous = instances.previous_time.unwrap_or(frame.time);
        let backwards = frame.time < previous || frame.change == EffectChange::Reset;
        let start = if backwards { frame.time } else { previous };
        let delta = frame.time - start;
        for (emitter, state) in self.emitters.iter().zip(&mut instances.emitters) {
            let had_births = !state.births.is_empty();
            let node =
                emitter.emitter_frame(frame.basis_world * frame.feature_matrix(emitter.node_index));
            if instances.previous_time.is_none() || backwards {
                state.births.clear();
                state.age_appearances.clear();
                if let Some(models) = &mut state.models {
                    models.births.clear();
                    if let Some(world) = model_world.as_mut() {
                        models.children.reset(world.commands);
                    }
                }
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
                allow_models: model_world.as_ref().is_none_or(|world| world.allow_births),
                actor,
                previous_basis: instances.previous_basis.unwrap_or(frame.basis_world),
                node,
                start,
                inherited,
                follow,
            };

            let mut clock = EmissionClock::new(start, frame.time);
            let mut checked_end = start;
            for step in clock.by_ref() {
                if actor
                    .filter(|actor| actor.owned || emitter.model.is_some())
                    .is_some_and(|actor| !actor.step())
                {
                    break;
                }
                checked_end = step.end;
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
                    let local_accepted = count.min(available);
                    let accepted = if emitter.model.is_some() && births.allow_models {
                        births.actor.map_or(local_accepted, |actor| {
                            local_accepted.min(actor.remaining())
                        })
                    } else {
                        local_accepted
                    };
                    if let Some(actor) = births
                        .actor
                        .filter(|_| emitter.model.is_some() && births.allow_models)
                    {
                        actor.budget.omit_work((local_accepted - accepted) as u64);
                    }
                    let omitted = count - accepted;
                    state.dropped = state.dropped.saturating_add(omitted as u64);
                    for index in 0..accepted {
                        let time = at + (index as f64 + 1. - prior) / rate;
                        births.push(state, time, &mut births_this_update);
                    }
                    state.ordinal = state.ordinal.wrapping_add(omitted as u64);
                }
            }
            let emission_omitted = (frame.time - checked_end).max(0.);
            if let Some(actor) = actor.filter(|actor| actor.owned || emitter.model.is_some()) {
                actor.budget.skip_seconds(emission_omitted);
            }
            if emitter.generator == ParticleGenerator::Sphere && emitter.flags.implosion_filter() {
                checked_end = bounded_end(start, checked_end, state.births.len());
            }
            let retirement_omitted = (frame.time - checked_end).max(0.);
            state.skipped_seconds += emission_omitted.max(retirement_omitted);
            let centers = [
                state.previous_node.w_axis.truncate(),
                node.w_axis.truncate(),
            ];
            emitter.retire_implosion(
                state,
                ImplosionInterval {
                    start,
                    end: frame.time,
                    checked_end,
                    centers,
                    follow,
                },
            );

            let node_changed = state.previous_node != node;
            state.previous_node = node;
            let redraw = backwards
                || delta != 0.
                || instances.previous_basis != Some(frame.basis_world)
                || node_changed
                || (!had_births && !state.births.is_empty());
            if emitter.model.is_none() && redraw && (had_births || !state.births.is_empty()) {
                if let Some(pass) = state.passes.first() {
                    if let Ok(mut data) = gpu_sprites.get_mut(pass.entity) {
                        emitter.sample_gpu(state, &frame, node, &mut data);
                    }
                }
            }
            if let (Some(model), Some(models), Some(world), Some(pass)) = (
                &emitter.model,
                &mut state.models,
                model_world.as_mut(),
                state.passes.first(),
            ) {
                models.projections.clear();
                for birth in &state.births {
                    if let Some(sidecar) = models.births.get_mut(birth.appearance_index as usize) {
                        let mut projection =
                            emitter.model_projection(birth, sidecar, model, frame.time, node);
                        projection.lease = sidecar.lease.take();
                        models.projections.push(projection);
                    }
                }
                models.children.synchronize(
                    world.commands,
                    ModelProjectionContext {
                        root: world.root,
                        owner: pass.entity,
                        visible: frame.root_visible,
                        driving_camera: world.driving_camera,
                    },
                    &model.template,
                    world.outputs,
                    models.projections.drain(..),
                );
                if let Ok(mut stats) = world.statistics.get_mut(pass.entity) {
                    if stats.alive != state.births.len() {
                        stats.alive = state.births.len();
                    }
                    if stats.omitted != state.dropped {
                        stats.omitted = state.dropped;
                    }
                    if stats.skipped_seconds != state.skipped_seconds {
                        stats.skipped_seconds = state.skipped_seconds;
                    }
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
        instances.previous_basis = Some(frame.basis_world);
    }
}

/// Filtering shares the emission clock's examined prefix and omission counter.
struct ImplosionInterval {
    start: f64,
    end: f64,
    checked_end: f64,
    centers: [Vec3; 2],
    follow: Vec3,
}

/// Common occurrence-pose evaluation for continuous and keyed emission.
struct BirthContext<'a, 'view, 'data> {
    emitter: &'a Emitter,
    clips: &'a HashMap<ClipId, (f64, bool)>,
    allow_models: bool,
    actor: Option<&'a ActorFrame>,
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
        let ordinal = state.ordinal;
        state.ordinal = state.ordinal.wrapping_add(1);
        if emitter.model.is_some() && !self.allow_models {
            state.dropped = state.dropped.saturating_add(1);
            return;
        }
        let lease = if emitter.model.is_some() {
            match self.actor.map(ActorFrame::reserve).transpose() {
                Ok(lease) => lease,
                Err(_) => {
                    state.dropped = state.dropped.saturating_add(1);
                    return;
                }
            }
        } else {
            None
        };
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
        if state.births.len() >= emitter.capacity() || *births >= MAX_BIRTHS_PER_UPDATE {
            state.dropped = state.dropped.saturating_add(1);
            return;
        }
        *births += 1;
        if let Some((mut birth, appearance)) = emitter.birth_posed(
            &sample,
            time,
            birth_node,
            birth_basis,
            self.inherited,
            ordinal,
            |candidate| {
                if let Some(pose) = frame.pose {
                    return Some(pose.at_offset(time - frame.time).sample_joint_span(
                        candidate.node,
                        &sample,
                        &mut state.pose_scratch,
                    ));
                }
                let joint = |index: usize, pivot: Vec3| {
                    frame
                        .node_poses
                        .get(index)
                        .map(|matrix| matrix.transform_point3(pivot))
                };
                let child = joint(candidate.node, candidate.pivot)?;
                let parent = candidate
                    .parent
                    .and_then(|(index, pivot)| joint(index, pivot));
                Some((child, parent))
            },
        ) {
            if frame.time - birth.time < f64::from(birth.lifetime) {
                // Newborns follow only emitter translation after occurrence.
                birth.origin += self.follow * (1. - fraction.clamp(0., 1.));
                if let (Some(model), Some(models)) = (&emitter.model, &mut state.models) {
                    let Some(basis) = ModelBasis::from_frame(birth_node) else {
                        state.dropped = state.dropped.saturating_add(1);
                        return;
                    };
                    let mut seed = emitter.seed ^ ordinal.wrapping_mul(0x9e3779b97f4a7c15);
                    let unit = Vec3::new(random(&mut seed), random(&mut seed), random(&mut seed));
                    let negate = if emitter.flags.negate_spin_random() {
                        [
                            random(&mut seed) < 0.5,
                            random(&mut seed) < 0.5,
                            random(&mut seed) < 0.5,
                        ]
                    } else {
                        [false; 3]
                    };
                    let appearance = match appearance {
                        Some(BirthAppearance::Age(value)) => value,
                        None => Default::default(),
                    };
                    let initial_size = emitter
                        .appearance
                        .scale
                        .as_ref()
                        .and_then(|curve| curve.sample(ModelTime::seconds(0.).unwrap()))
                        .unwrap_or(Vec2::ONE);
                    let initial_twinkle = emitter
                        .age_appearance
                        .as_ref()
                        .map_or(1., |style| style.sample(&appearance, 0.).scale);
                    let original_origin =
                        birth.origin - self.follow * (1. - fraction.clamp(0., 1.));
                    let initial_position = if emitter.flags.attached() {
                        birth_node.transform_point3(original_origin)
                    } else {
                        original_origin
                    };
                    let Some(initial_transform) = basis.sample(
                        initial_position,
                        initial_size,
                        initial_twinkle,
                        model.template.coordinates.parent_basis.scale,
                        Vec3::ZERO,
                        0.,
                    ) else {
                        state.dropped = state.dropped.saturating_add(1);
                        return;
                    };
                    birth.appearance_index = models.births.len() as u32;
                    models.births.push(GeometryBirth {
                        lease,
                        initial_transform,
                        ordinal,
                        basis,
                        omega: tumble(model.minimum, model.maximum, unit, negate),
                        appearance,
                    });
                } else if let Some(appearance) = appearance {
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
    fn model_projection(
        &self,
        birth: &Birth,
        sidecar: &GeometryBirth,
        model: &GeometryEmitter,
        time: f64,
        node: Mat4,
    ) -> ModelProjection {
        let seconds = (time - birth.time).max(0.) as f32;
        let style = self
            .age_appearance
            .as_ref()
            .map(|appearance| appearance.sample(&sidecar.appearance, seconds))
            .unwrap_or_default();
        let transform = (|| {
            let age =
                ModelTime::seconds((f64::from(seconds) / f64::from(birth.lifetime)).clamp(0., 1.))
                    .ok()?;
            let size = self
                .appearance
                .scale
                .as_ref()
                .and_then(|curve| curve.sample(age))
                .unwrap_or(Vec2::ONE);
            let (position, _) = self.motion(birth, time);
            let position = if self.flags.attached() {
                node.transform_point3(position)
            } else {
                position
            };
            let basis = if self.flags.attached() {
                ModelBasis::from_frame(node)?
            } else {
                sidecar.basis
            };
            basis.sample(
                position,
                size,
                style.scale,
                model.template.coordinates.parent_basis.scale,
                sidecar.omega,
                seconds,
            )
        })();
        // Every living birth remains a pool member even when its attached frame
        // temporarily collapses. The pool retains its hidden, finite actor.
        ModelProjection {
            lease: None,
            ordinal: sidecar.ordinal,
            birth: birth.time,
            lifetime: f64::from(birth.lifetime),
            age: f64::from(seconds),
            transform,
            initial_transform: sidecar.initial_transform,
            visible: style.visible,
        }
    }
    fn retire_implosion(&self, state: &mut EmitterInstance, interval: ImplosionInterval) {
        if self.generator != ParticleGenerator::Sphere
            || !self.flags.implosion_filter()
            || interval.checked_end <= interval.start
        {
            return;
        }
        let fraction = |time: f64| {
            ((time - interval.start) / (interval.end - interval.start)).clamp(0., 1.) as f32
        };
        state.retain(|birth| {
            let result = outward_crossing(
                interval.start.max(birth.time),
                interval.checked_end,
                |time| {
                    // Origins already contain the full observed follow offset;
                    // reconstruct only the translation received by this time.
                    self.motion(birth, time).0 - interval.follow * (1. - fraction(time))
                },
                |time| {
                    if self.flags.attached() {
                        Vec3::ZERO
                    } else {
                        interval.centers[0].lerp(interval.centers[1], fraction(time))
                    }
                },
            );
            // Omitted seconds were reported once by the emitter's emission
            // clock, rather than being added again for each living particle.
            debug_assert_eq!(result.skipped_seconds, 0.);
            result.time.is_none()
        });
    }

    fn emitter_frame(&self, node: Mat4) -> Mat4 {
        // M2 emitter X/Y axes are rotated relative to the carrying bone.
        node * Mat4::from_rotation_z(FRAC_PI_2)
    }
    fn particle_scale(&self, node: Mat4, basis: Mat4) -> Vec2 {
        let inherited = if self.flags.inherit_bone_scale() {
            (basis.inverse() * node).x_axis.truncate().length()
        } else {
            1.
        };
        Vec2::new(
            basis.x_axis.truncate().length() * inherited,
            basis.y_axis.truncate().length() * inherited,
        )
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
        let [[speed_a, scale_a], [speed_b, scale_b]] = self.follow.map(|v| v.to_array());
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
        if self.model.is_some() {
            1024
        } else {
            MAX_PARTICLES
        }
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
    #[cfg(test)]
    fn birth_in(
        &self,
        sample: &impl ValueSample,
        time: f64,
        node: Mat4,
        basis: Mat4,
        inherited: Vec3,
        ordinal: u64,
    ) -> Option<(Birth, Option<BirthAppearance>)> {
        self.birth_posed(sample, time, node, basis, inherited, ordinal, |_| None)
    }
    #[allow(clippy::too_many_arguments)]
    fn birth_posed(
        &self,
        sample: &impl ValueSample,
        time: f64,
        node: Mat4,
        basis: Mat4,
        inherited: Vec3,
        ordinal: u64,
        bone_pose: impl FnMut(BoneCandidate) -> Option<(Vec3, Option<Vec3>)>,
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
            ParticleGenerator::Spline => {
                let (position, tangent) = spline(&self.spline, length, width, random(&mut seed));
                direction = tangent;
                position
            }
            ParticleGenerator::Bone => {
                let draws = [
                    random(&mut seed),
                    random(&mut seed),
                    random(&mut seed),
                    random(&mut seed),
                ];
                let model_origin = self
                    .bones
                    .as_ref()?
                    .sample(draws, width, length, bone_pose)?;
                direction = Vec3::Z;
                (node.inverse() * basis).transform_point3(model_origin)
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
        let gravity = sample.animated(&controls.gravity);
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
        let scale_variation = Vec2::new(x_variation, y_variation);
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
    fn sample_gpu(
        &self,
        state: &mut EmitterInstance,
        frame: &EffectFrame<'_, '_, '_>,
        node: Mat4,
        data: &mut AgeSpriteData,
    ) {
        let (record_snapshot, index_snapshot) = state.samples.begin();
        if (!data.style.head && !data.style.tail)
            || !frame.basis_world.is_finite()
            || frame.basis_world.determinant() == 0.
        {
            data.records = record_snapshot.clone();
            data.live_indices = index_snapshot.clone();
            return;
        }
        let records = Arc::get_mut(record_snapshot).unwrap();
        for birth in &state.births {
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
            let age = ModelTime::seconds(age).unwrap();
            let rgb = self
                .appearance
                .color
                .as_ref()
                .and_then(|v| v.sample(age))
                .unwrap_or(Vec3::from_array([1.; 3]));
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
                .unwrap_or(Vec2::from_array([1.; 2]));
            size *= birth.scale_variation * native_style.scale;
            let (mut position, mut velocity) = self.motion(birth, frame.time);
            if self.flags.attached() {
                position = node.transform_point3(position);
                velocity = node.transform_vector3(velocity);
            }
            let spin = birth.spin + birth.spin_rate * (frame.time - birth.time) as f32;
            let (sin, cos) = spin.sin_cos();
            let scale = if self.flags.attached() {
                self.particle_scale(node, frame.basis_world)
            } else {
                birth.size_scale
            };
            let tail_time = if self.flags.clamp_tail_to_age() {
                self.tail_length
                    .min((frame.time - birth.time).max(0.) as f32)
            } else {
                self.tail_length
            };
            let cell = |curve: &Option<Curve<'_, u16>>| {
                curve
                    .as_ref()
                    .and_then(|curve| curve.sample(age))
                    .unwrap_or(0) as u32
                    + birth.random_cell
            };
            let scales = self
                .age_appearance
                .as_ref()
                .map(AgeAppearance::texture_scales)
                .unwrap_or([1.; 2]);
            records.push(AgeSpriteRecord {
                position: position.extend(0.).to_array(),
                velocity_tail: velocity.extend(tail_time).to_array(),
                size_spin: [size.x * scale.x, size.y * scale.y, cos, sin],
                color: [rgb.x, rgb.y, rgb.z, alpha],
                offsets: [
                    native_style.offsets[0].x,
                    native_style.offsets[0].y,
                    native_style.offsets[1].x,
                    native_style.offsets[1].y,
                ],
                scales: [scales[0], scales[1], 0., 0.],
                cells: [
                    cell(&self.appearance.head_uv),
                    cell(&self.appearance.tail_uv),
                    0,
                    0,
                ],
            });
        }
        let indices = Arc::get_mut(index_snapshot).unwrap();
        for index in 0..records.len() as u32 {
            if data.style.head {
                indices.push(index);
            }
            if data.style.tail {
                indices.push(index | 0x8000_0000);
            }
        }
        data.records = record_snapshot.clone();
        data.live_indices = index_snapshot.clone();
        let axes = if self.flags.attached() {
            node
        } else {
            frame.basis_world
        };
        data.uniform = AgeSpriteUniform {
            axes: [axes.x_axis.truncate(), axes.y_axis.truncate()]
                .map(|axis| axis.normalize_or_zero().extend(0.).to_array()),
            flags: [
                u32::from(self.flags.align_xy()),
                u32::from(self.flags.velocity_oriented()),
                u32::from(data.style.unlit),
                u32::from(!self.flags.unfogged()),
            ],
            atlas: [
                u32::from(self.atlas[0].max(1)),
                u32::from(self.atlas[1].max(1)),
                data.style.composition.flags() | (u32::from(data.style.uv_flip_v) << 3),
                u32::from(data.style.blend.raw()),
            ],
        };
    }
    #[cfg(test)]
    fn draw(
        &self,
        state: &mut EmitterInstance,
        frame: &EffectFrame<'_, '_, '_>,
        node: Mat4,
        mesh: &mut Mesh,
        camera: Mat4,
    ) {
        if !frame.basis_world.is_finite() || frame.basis_world.determinant() == 0. {
            return;
        }
        let inverse_basis = frame.basis_world.inverse();
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
        match self.gpu_style.sort {
            SortPolicy::None => {}
            SortPolicy::Far => state
                .order
                .sort_unstable_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0))),
            SortPolicy::Near => state
                .order
                .sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0))),
        }
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
                .unwrap_or(Vec3::from_array([1.; 3]));
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
                .unwrap_or(Vec2::from_array([1.; 2]));
            size *= birth.scale_variation * native_style.scale;
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
        update_sprite_indices(mesh, positions.len());
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
/// Reuse the topology allocation; particle sorting changes corner attributes,
/// while the index pattern depends only on the number of visible quads.
#[cfg(test)]
fn update_sprite_indices(mesh: &mut Mesh, vertices: usize) {
    let mut indices = match mesh.remove_indices() {
        Some(Indices::U16(values)) => values,
        _ => Vec::new(),
    };
    if vertices == 3 {
        indices.clear();
        indices.extend_from_slice(&[0, 1, 2]);
    } else {
        let count = vertices / 4 * 6;
        // The empty sentinel has three indices and is not a quad prefix.
        indices.truncate(indices.len() / 6 * 6);
        indices.truncate(count);
        for quad in indices.len() / 6..vertices / 4 {
            let base = u16::try_from(quad * 4).expect("sprite vertex budget exceeded");
            indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    mesh.insert_indices(Indices::U16(indices));
}
#[cfg(test)]
fn take_vec3(mesh: &mut Mesh, attribute: MeshVertexAttribute) -> Vec<[f32; 3]> {
    let mut values = match mesh.remove_attribute(attribute) {
        Some(VertexAttributeValues::Float32x3(values)) => values,
        _ => Vec::new(),
    };
    values.clear();
    values
}
#[cfg(test)]
fn take_vec2(mesh: &mut Mesh, attribute: MeshVertexAttribute) -> Vec<[f32; 2]> {
    let mut values = match mesh.remove_attribute(attribute) {
        Some(VertexAttributeValues::Float32x2(values)) => values,
        _ => Vec::new(),
    };
    values.clear();
    values
}
#[cfg(test)]
fn take_vec4(mesh: &mut Mesh, attribute: MeshVertexAttribute) -> Vec<[f32; 4]> {
    let mut values = match mesh.remove_attribute(attribute) {
        Some(VertexAttributeValues::Float32x4(values)) => values,
        _ => Vec::new(),
    };
    values.clear();
    values
}
#[cfg(test)]
struct SpriteMeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    uv_secondary: Option<Vec<[f32; 2]>>,
    uv_tertiary: Option<Vec<[f32; 2]>>,
    colors: Vec<[f32; 4]>,
}
#[cfg(test)]
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
        for index in 0..4 {
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
#[path = "m2_particles_tests.rs"]
mod tests;

impl RuntimeM2Particles {
    pub(crate) fn map_images(
        &mut self,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        for emitter in &mut self.emitters {
            for image in emitter.gpu_style.textures.iter_mut().flatten() {
                if let Some(replacement) = map(image) {
                    *image = replacement;
                }
            }
        }
    }
}
