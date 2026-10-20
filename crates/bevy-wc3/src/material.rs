//! Direct semantic surface preparation for the Bevy PBR pipeline.
mod animation;
mod bindings;
mod compile;
mod m2;

#[path = "ribbon_material.rs"]
mod ribbon_material;
mod wmo;
#[cfg(test)]
use self::wmo::compile_wmo;
use crate::diagnostics::RenderIssue;
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
#[cfg(test)]
use compile::{blend, depth};
#[cfg(test)]
use native_fixtures::wmo as wmo_shader;
pub(crate) use ribbon_material::effect_state;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::materials::{m2::GuildPalette, mdlx};
use wc3::model::{Property, Texture, TextureTransform};

use super::mesh_animation::MeshAnimation;
use super::mesh_attributes::specialize_mesh_attributes;
use super::textures::TextureImages;

/// One authored draw pass, including any limits of its PBR interpretation.
pub(crate) struct SurfacePass {
    pub material: StandardMaterial,
    pub coordinate: u32,
    pub diagnostics: Vec<RenderIssue>,
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

    blend: Option<Blend>,
    depth_test: bool,
    depth_write: Option<bool>,
    cast_shadows: bool,
    receive_shadows: bool,
    fourth_uv: bool,
}

impl ModelMaterialExtension {
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
            background: self.combiner.guild_background.truncate(),
            emblem: self.combiner.guild_emblem.truncate(),
            border: self.combiner.guild_border.truncate(),
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
        self.combiner.guild_background = palette.background.extend(1.);
        self.combiner.guild_emblem = palette.emblem.extend(1.);
        self.combiner.guild_border = palette.border.extend(1.);
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
}

impl From<&ModelMaterialExtension> for ModelMaterialKey {
    fn from(state: &ModelMaterialExtension) -> Self {
        Self {
            fourth_uv: state.fourth_uv,
            world_layer_alpha: matches!(
                state.world.composition.x,
                7 | 10 | 12 | 13 | 14 | 16 | 18 | 19 | 20
            ),
            edge_fade: state.combiner.clip.y != 0.0,
            third_uv: matches!(state.combiner.coordinates.w, 1 | 3)
                || state.combiner.additional_coordinates.x == 3
                || state.world.composition.y > 2,
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

#[derive(Clone, Copy, Default)]
enum UvConvention {
    #[default]
    Origin,
    Layered,
    M2,
}

/// Owned bindings shared by instances; sampling allocates no texture tables.
#[derive(Clone, Default)]
pub(crate) struct SurfaceAnimation {
    animated: bool,
    untinted: Option<LinearRgba>,
    untinted_alpha_mode: Option<AlphaMode>,
    mesh: Option<MeshAnimation>,
    color: Option<Property<Vec3>>,
    rgba: Option<Property<Vec4>>,
    opacity: Option<Property<f32>>,
    weight: Option<Property<f32>>,
    pass_weights: Vec<Property<f32>>,
    unweighted_opacity: Option<f32>,
    texture: Option<Property<Option<Texture>>>,
    transform: Option<TextureTransform>,
    secondary_transform: Option<TextureTransform>,
    uv_scroll: [Vec2; 2],
    third_transform: Option<TextureTransform>,
    third_uv_scroll: Vec2,
    fourth_transform: Option<TextureTransform>,
    input_weights: [Option<Property<f32>>; 4],
    emissive_gain: Option<Property<f32>>,
    emission: Option<Property<Vec4>>,
    emission_scale: f32,
    images: Arc<HashMap<Texture, Handle<Image>>>,
    uv_convention: UvConvention,
    hd: Option<HdAnimation>,
}

#[derive(Clone)]
struct HdAnimation {
    slots: [Option<Property<Option<Texture>>>; 6],
    fresnel: Option<mdlx::Fresnel>,
    gain: Option<Property<f32>>,
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

/// Substitute symbolic image identities without changing sampled material state.
pub(crate) fn map_material_images(
    material: &mut ModelMaterial,
    map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
) {
    let base = &mut material.base;
    for image in [
        &mut base.base_color_texture,
        &mut base.emissive_texture,
        &mut base.metallic_roughness_texture,
        &mut base.normal_map_texture,
        &mut base.occlusion_texture,
        &mut base.depth_map,
    ] {
        *image = image.as_ref().and_then(&mut *map);
    }
    let extension = &mut material.extension;
    for image in [
        &mut extension.secondary,
        &mut extension.orm,
        &mut extension.team_color,
        &mut extension.environment,
        &mut extension.tertiary,
        &mut extension.fourth,
    ] {
        *image = image.as_ref().and_then(&mut *map);
    }
    if extension.hd.maps.x != 0 {
        base.metallic = if extension.orm.is_some() { 1. } else { 0. };
        extension.hd.maps.y = u32::from(extension.orm.is_some());
        extension.hd.maps.z = u32::from(extension.team_color.is_some());
        extension.hd.maps.w = u32::from(extension.environment.is_some());
        if base.emissive_texture.is_none() {
            base.emissive = LinearRgba::BLACK;
        }
    }
}
impl SurfaceAnimation {
    pub(crate) fn map_images(
        &mut self,
        map: &mut impl FnMut(&Handle<Image>) -> Option<Handle<Image>>,
    ) {
        self.images = Arc::new(
            self.images
                .iter()
                .filter_map(|(key, image)| map(image).map(|image| (*key, image)))
                .collect(),
        );
        if let Some(hd) = &mut self.hd {
            Arc::make_mut(&mut hd.images).map_images(map);
        }
    }
}

#[cfg(test)]
mod image_rebinding_tests {
    use super::*;

    #[test]
    fn missing_hd_images_restore_presence_dependent_material_defaults() {
        let mut material = ModelMaterial::default();
        material.extension.hd.maps = UVec4::ONE;
        material.extension.orm = Some(Handle::default());
        material.extension.team_color = Some(Handle::default());
        material.extension.environment = Some(Handle::default());
        material.base.emissive_texture = Some(Handle::default());
        material.base.metallic = 1.;
        material.base.emissive = LinearRgba::WHITE;
        map_material_images(&mut material, &mut |_| None);
        assert_eq!(material.base.metallic, 0.);
        assert_eq!(material.base.emissive, LinearRgba::BLACK);
        assert_eq!(material.extension.hd.maps, UVec4::new(1, 0, 0, 0));
    }
}
