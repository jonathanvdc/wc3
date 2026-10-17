//! Direct semantic surface preparation for the Bevy PBR pipeline.
mod animation;
mod bindings;
mod compile;
mod m2;
mod particle;
pub(crate) use particle::{particle_material_layers, ParticleComposition};
mod m3;
use m3::layers::{M3Animation, M3Uniform};
pub use m3::TeamColors;
#[path = "ribbon_material.rs"]
mod ribbon_material;
mod wmo;
#[cfg(test)]
use self::wmo::compile_wmo;
use crate::ModelError;
use animation::layered_uv;
pub(super) use animation::mesh_alpha_mode;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::render::render_resource::{
    BlendComponent, BlendFactor, BlendOperation, BlendState, CompareFunction,
    RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
#[cfg(test)]
pub(crate) use compile::compile_material_with_images;
pub(crate) use compile::compile_materials_with_images;
use compile::rasterization;
#[cfg(test)]
use compile::{blend, depth};
#[cfg(test)]
use native_fixtures::wmo as wmo_shader;
pub(crate) use ribbon_material::effect_state;
use std::array::from_fn;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::materials::{m2::GuildPalette, mdlx, state::Lighting};
use wc3::model::values::model::ParticleBlend;
use wc3::model::{Animated, Texture, TextureTransform};

use super::mesh_animation::MeshAnimation;
use super::mesh_attributes::specialize_mesh_attributes;
use super::textures::TextureImages;

/// One authored draw pass, including any limits of its PBR interpretation.
pub(crate) struct SurfacePass {
    pub material: StandardMaterial,
    pub coordinate: u32,
    pub diagnostics: Vec<String>,
    pub animation: SurfaceAnimation,
    pub state: ModelMaterialExtension,
    pub secondary_coordinate: Option<u32>,
    pub third_coordinate: Option<u32>,
    pub fourth_coordinate: Option<u32>,
    pub exterior_lit: bool,
    pub sort_far: bool,
    pub sort_near: bool,
}

/// A semantic material retaining Bevy PBR shading and authored draw state.
pub type ModelMaterial = ExtendedMaterial<StandardMaterial, ModelMaterialExtension>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum Blend {
    #[default]
    Opaque,
    Alpha,
    Additive,
    AdditiveAlpha,
    Modulate,
    Modulate2x,
    ModulateAdd,
    InverseSourceAlphaAdd,
    Premultiplied,
}

/// Semantic fixed-function state without a dependency on any native model codec.
#[derive(Asset, TypePath, Clone, Debug)]
pub struct ModelMaterialExtension {
    combiner: CombinerUniform,
    secondary: Option<Handle<Image>>,
    hd: HdUniform,
    orm: Option<Handle<Image>>,
    team_color: Option<Handle<Image>>,
    environment: Option<Handle<Image>>,
    tertiary: Option<Handle<Image>>,
    fourth: Option<Handle<Image>>,
    world: WorldUniform,
    particle: ParticleUniform,
    m3: M3Uniform,
    m3_samplers: [Option<Handle<Image>>; 6],
    normal_secondary: Option<Handle<Image>>,
    normal_secondary_mask: Option<Handle<Image>>,
    light_map: Option<Handle<Image>>,
    ambient_occlusion: Option<Handle<Image>>,
    blend: Option<Blend>,
    depth_test: bool,
    depth_write: Option<bool>,
    cast_shadows: bool,
    receive_shadows: bool,
    fourth_uv: bool,
}

impl ModelMaterialExtension {
    pub(crate) fn enable_effect_color(&mut self) {
        if self.m3.settings.x != 0 {
            self.m3.settings.y = 1;
            self.m3.settings.z = 1;
        }
    }

    /// Application colours used by an M3 standard material's team operations.
    pub fn team_colors(&self) -> Option<TeamColors> {
        (self.m3.settings.x != 0).then(|| TeamColors {
            diffuse: self.m3.diffuse_team.truncate(),
            emissive: self.m3.emissive_team.truncate(),
            diffuse_exponent: self.m3.diffuse_team.w,
            emissive_exponent: self.m3.emissive_team.w,
        })
    }

    /// Updates team colours without changing shader pipelines or model resources.
    ///
    /// Returns `true` when a standard material's colours change; returns `false`
    /// when the colours are unchanged or this surface has no team-colour program.
    pub fn set_team_colors(&mut self, colors: TeamColors) -> Result<bool, ModelError> {
        if !colors.diffuse.is_finite()
            || !colors.emissive.is_finite()
            || !colors.diffuse_exponent.is_finite()
            || colors.diffuse_exponent < 0.
            || !colors.emissive_exponent.is_finite()
            || colors.emissive_exponent < 0.
        {
            return Err(ModelError::new(
                "team colours must be finite and both mask exponents nonnegative",
            ));
        }
        if self.m3.settings.x == 0 {
            return Ok(false);
        }
        if self.team_colors() == Some(colors) {
            return Ok(false);
        }
        self.m3.diffuse_team = colors.diffuse.extend(colors.diffuse_exponent);
        self.m3.emissive_team = colors.emissive.extend(colors.emissive_exponent);
        Ok(true)
    }

    /// Whether the authored surface casts ordinary Bevy shadows.
    pub fn casts_shadows(&self) -> bool {
        self.cast_shadows
    }

    /// Whether the authored surface receives ordinary Bevy shadows.
    pub fn receives_shadows(&self) -> bool {
        self.receive_shadows
    }

    pub(crate) fn set_world_illumination(&mut self, ambient: Vec3, lighting: u32) -> [u32; 4] {
        if self.world.flags.x == 0 {
            return [0; 4];
        }
        self.world.ambient = ambient.extend(0.);
        self.world.flags.z = lighting;
        [
            ambient.x.to_bits(),
            ambient.y.to_bits(),
            ambient.z.to_bits(),
            lighting,
        ]
    }

    /// Current application palette for a guild programme, or `None` for other surfaces.
    pub fn guild_palette(&self) -> Option<GuildPalette> {
        matches!(self.combiner.mode.x, 24..=26).then(|| GuildPalette {
            background: self.combiner.guild_background.truncate().to_array(),
            emblem: self.combiner.guild_emblem.truncate().to_array(),
            border: self.combiner.guild_border.truncate().to_array(),
        })
    }

    /// Update guild colours in an existing Bevy material without recompiling it.
    ///
    /// Returns `false` for a non-guild surface. Non-finite values are rejected
    /// without changing the material. Multipliers are linear RGB and may exceed one.
    pub fn set_guild_palette(&mut self, palette: GuildPalette) -> Result<bool, ModelError> {
        if !palette.is_finite() {
            return Err(ModelError::new("guild colours must be finite"));
        }
        if self.guild_palette().is_none() {
            return Ok(false);
        }
        self.combiner.guild_background = Vec3::from_array(palette.background).extend(1.);
        self.combiner.guild_emblem = Vec3::from_array(palette.emblem).extend(1.);
        self.combiner.guild_border = Vec3::from_array(palette.border).extend(1.);
        Ok(true)
    }
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
struct WorldUniform {
    // x: diffuse composition programme, y: sampled image count.
    composition: UVec4,
    // x = world vertex illumination, y = ignore diffuse-map alpha, z = lighting class (transition/interior/exterior).
    flags: UVec4,
    ambient: Vec4,
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct HdUniform {
    fresnel_color: Vec4,
    fresnel: Vec4,
    maps: UVec4,
}
impl Default for HdUniform {
    fn default() -> Self {
        Self {
            fresnel_color: Vec4::ONE,
            fresnel: Vec4::ZERO,
            maps: UVec4::ZERO,
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct CombinerUniform {
    uv_transform: Mat3,
    third_transform: Mat3,
    fourth_transform: Mat3,
    // x: fourth route, using the same raw-stream selectors as coordinates.w.
    additional_coordinates: UVec4,
    // xy: environment selectors; z: input count. Third route w:
    // 0 = raw primary, 1 = environment, 2 = raw secondary, 3 = distinct stream.
    coordinates: UVec4,
    weights: Vec4,
    guild_background: Vec4,
    guild_emblem: Vec4,
    guild_border: Vec4,
    // x: enabled, y: first alpha, z: second alpha, w: RGB/alpha doubling.
    mode: UVec4,
    clip: Vec4,
}
#[derive(Clone, Copy, Debug, Default, ShaderType)]
struct ParticleUniform {
    mode: UVec4,
}

impl Default for CombinerUniform {
    fn default() -> Self {
        Self {
            uv_transform: Mat3::IDENTITY,
            third_transform: Mat3::IDENTITY,
            fourth_transform: Mat3::IDENTITY,
            additional_coordinates: UVec4::ZERO,
            coordinates: UVec4::ZERO,
            weights: Vec4::ONE,
            guild_background: Vec4::ONE,
            guild_emblem: Vec4::ONE,
            guild_border: Vec4::ONE,
            mode: UVec4::ZERO,
            clip: Vec4::ZERO,
        }
    }
}

pub(crate) fn particle_material(
    blend: ParticleBlend,
    texture: Option<Handle<Image>>,
    lit: bool,
) -> ModelMaterial {
    let mut pass = SurfacePass::default();
    pass.material.base_color_texture = texture;
    rasterization(
        &mut pass,
        true,
        Lighting {
            unlit: !lit,
            unfogged: false,
        },
    );
    match blend {
        ParticleBlend::Opaque => {}
        ParticleBlend::AlphaKey => {
            pass.material.alpha_mode = AlphaMode::Mask(128.0 / 255.0);
        }
        ParticleBlend::NoAlphaAdd => {
            pass.material.alpha_mode = AlphaMode::Blend;
            pass.state.blend = Some(Blend::Additive);
        }
        ParticleBlend::Add => {
            pass.material.alpha_mode = AlphaMode::Blend;
            pass.state.blend = Some(Blend::AdditiveAlpha);
        }
        ParticleBlend::Modulate => {
            pass.material.alpha_mode = AlphaMode::Blend;
            pass.state.blend = Some(Blend::Modulate);
        }
        ParticleBlend::Modulate2x => {
            pass.material.alpha_mode = AlphaMode::Blend;
            pass.state.blend = Some(Blend::Modulate2x);
        }
        ParticleBlend::BlendAdd => {
            // The authored shader color is already the source term for
            // (One, OneMinusSrcAlpha); Bevy's Premultiplied mode preserves it.
            pass.material.alpha_mode = AlphaMode::Premultiplied;
            pass.state.blend = Some(Blend::Premultiplied);
        }
        ParticleBlend::Alpha | ParticleBlend::Unknown(_) => {
            pass.material.alpha_mode = AlphaMode::Blend;
            pass.state.blend = Some(Blend::Alpha);
        }
    }
    pass.state.depth_write = Some(matches!(
        blend,
        ParticleBlend::Opaque | ParticleBlend::AlphaKey
    ));
    ModelMaterial {
        base: pass.material,
        extension: pass.state,
    }
}

impl Default for ModelMaterialExtension {
    fn default() -> Self {
        Self {
            combiner: CombinerUniform::default(),
            secondary: None,
            tertiary: None,
            fourth: None,
            hd: HdUniform::default(),
            orm: None,
            team_color: None,
            environment: None,
            world: WorldUniform::default(),
            particle: ParticleUniform::default(),
            m3: M3Uniform::default(),
            m3_samplers: from_fn(|_| None),
            normal_secondary: None,
            normal_secondary_mask: None,
            light_map: None,
            ambient_occlusion: None,
            blend: None,
            depth_test: true,
            depth_write: None,
            cast_shadows: true,
            receive_shadows: true,
            fourth_uv: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Pipeline identity for semantic blending and depth operations.
pub struct ModelMaterialKey {
    blend: Option<Blend>,
    depth_test: bool,
    depth_write: Option<bool>,
    generated_coordinates: bool,
    third_uv: bool,
    fourth_uv: bool,
    world_layer_alpha: bool,
    edge_fade: bool,
    m3: bool,
}

impl From<&ModelMaterialExtension> for ModelMaterialKey {
    fn from(state: &ModelMaterialExtension) -> Self {
        Self {
            m3: state.m3.settings.x != 0,
            fourth_uv: state.fourth_uv,
            world_layer_alpha: matches!(
                state.world.composition.x,
                7 | 10 | 12 | 13 | 14 | 16 | 18 | 19 | 20
            ),
            edge_fade: state.combiner.clip.y != 0.0,
            third_uv: state.particle.mode.x != 0
                || (matches!(state.combiner.coordinates.w, 1 | 3)
                    || state.combiner.additional_coordinates.x == 3
                    || state.world.composition.y > 2
                    || state
                        .m3
                        .layers
                        .iter()
                        .any(|layer| layer.flags.y != 0 && layer.route.x == 2)),
            generated_coordinates: state.combiner.coordinates.x != 0
                || state.combiner.coordinates.y != 0
                || state.combiner.coordinates.w == 1,
            blend: state.blend,
            depth_test: state.depth_test,
            depth_write: state
                .depth_write
                .or_else(|| state.blend.map(|blend| blend == Blend::Opaque)),
        }
    }
}

impl ModelMaterialKey {
    fn apply_blend(self, target: &mut Option<BlendState>) {
        let Some(blend) = self.blend else { return };
        let factors = match blend {
            Blend::Opaque => None,
            Blend::Alpha => Some((BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha)),
            Blend::Additive => Some((BlendFactor::One, BlendFactor::One)),
            Blend::AdditiveAlpha => Some((BlendFactor::SrcAlpha, BlendFactor::One)),
            Blend::Modulate => Some((BlendFactor::Zero, BlendFactor::Src)),
            Blend::Modulate2x => Some((BlendFactor::Dst, BlendFactor::Src)),
            Blend::ModulateAdd => Some((BlendFactor::Dst, BlendFactor::One)),
            Blend::InverseSourceAlphaAdd => Some((BlendFactor::OneMinusSrcAlpha, BlendFactor::One)),
            Blend::Premultiplied => Some((BlendFactor::One, BlendFactor::OneMinusSrcAlpha)),
        };
        *target = factors.map(|(src_factor, dst_factor)| BlendState {
            color: BlendComponent {
                src_factor,
                dst_factor,
                operation: BlendOperation::Add,
            },
            alpha: match self.blend {
                Some(Blend::ModulateAdd) => BlendComponent {
                    src_factor: BlendFactor::DstAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                Some(Blend::InverseSourceAlphaAdd) => BlendComponent {
                    src_factor: BlendFactor::OneMinusSrcAlpha,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                _ => BlendComponent::OVER,
            },
        });
    }
}

impl MaterialExtension for ModelMaterialExtension {
    fn vertex_shader() -> ShaderRef {
        "embedded://bevy_wc3/shaders/wc3_mesh.wgsl".into()
    }
    fn prepass_vertex_shader() -> ShaderRef {
        "embedded://bevy_wc3/shaders/wc3_prepass.wgsl".into()
    }
    fn deferred_vertex_shader() -> ShaderRef {
        Self::prepass_vertex_shader()
    }
    fn fragment_shader() -> ShaderRef {
        "embedded://bevy_wc3/shaders/material.wgsl".into()
    }
    fn deferred_fragment_shader() -> ShaderRef {
        Self::fragment_shader()
    }
    fn prepass_fragment_shader() -> ShaderRef {
        "embedded://bevy_wc3/shaders/material_prepass.wgsl".into()
    }
    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        specialize_mesh_attributes(
            descriptor,
            layout,
            key.bind_group_data.generated_coordinates || key.bind_group_data.edge_fade,
            key.bind_group_data.third_uv,
            key.bind_group_data.world_layer_alpha,
            key.bind_group_data.fourth_uv,
        )?;
        if key.bind_group_data.m3 {
            if let Some(fragment) = &mut descriptor.fragment {
                fragment.shader_defs.push("M3_MATERIAL".into());
            }
        }
        if key.bind_group_data.edge_fade {
            descriptor.vertex.shader_defs.push("MODEL_EDGE_FADE".into());
            if let Some(fragment) = &mut descriptor.fragment {
                fragment.shader_defs.push("MODEL_EDGE_FADE".into());
            }
        }
        let state = key.bind_group_data;
        if let Some(depth) = &mut descriptor.depth_stencil {
            if !state.depth_test {
                depth.depth_compare = Some(CompareFunction::Always);
            }
            if let Some(write) = state.depth_write {
                depth.depth_write_enabled = Some(write);
            }
        }
        if let Some(target) = descriptor
            .fragment
            .as_mut()
            .and_then(|fragment| fragment.targets.first_mut())
            .and_then(Option::as_mut)
        {
            state.apply_blend(&mut target.blend);
        }
        Ok(())
    }
}

/// Owned bindings shared by instances; sampling allocates no texture tables.
#[derive(Clone, Default)]
pub(crate) struct SurfaceAnimation {
    animated: bool,
    untinted: Option<LinearRgba>,
    untinted_alpha_mode: Option<AlphaMode>,
    mesh: Option<MeshAnimation>,
    color: Option<Animated<[f32; 3]>>,
    rgba: Option<Animated<[f32; 4]>>,
    opacity: Option<Animated<f32>>,
    weight: Option<Animated<f32>>,
    pass_weights: Vec<Animated<f32>>,
    unweighted_opacity: Option<f32>,
    texture: Option<Animated<Option<Texture>>>,
    transform: Option<TextureTransform>,
    secondary_transform: Option<TextureTransform>,
    uv_scroll: [[f32; 2]; 2],
    third_transform: Option<TextureTransform>,
    third_uv_scroll: [f32; 2],
    fourth_transform: Option<TextureTransform>,
    input_weights: [Option<Animated<f32>>; 4],
    emissive_gain: Option<Animated<f32>>,
    emission: Option<Animated<[f32; 4]>>,
    emission_scale: f32,
    images: Arc<HashMap<Texture, Handle<Image>>>,
    centered_transform: bool,
    hd: Option<HdAnimation>,
    m3: Option<M3Animation>,
}

#[derive(Clone)]
struct HdAnimation {
    slots: [Option<Animated<Option<Texture>>>; 6],
    fresnel: Option<mdlx::Fresnel>,
    gain: Option<Animated<f32>>,
    images: Arc<TextureImages>,
}
impl Default for SurfacePass {
    fn default() -> Self {
        Self {
            material: StandardMaterial {
                perceptual_roughness: 1.0,
                reflectance: 0.0,
                ..Default::default()
            },
            coordinate: 0,
            diagnostics: Vec::new(),
            animation: SurfaceAnimation::default(),
            state: ModelMaterialExtension {
                blend: Some(Blend::Opaque),
                ..Default::default()
            },
            secondary_coordinate: None,
            third_coordinate: None,
            fourth_coordinate: None,
            exterior_lit: false,
            sort_far: false,
            sort_near: false,
        }
    }
}

#[cfg(test)]
#[path = "material_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "material/static_binding_tests.rs"]
mod static_binding_tests;

#[cfg(test)]
pub(crate) mod native_fixtures;
