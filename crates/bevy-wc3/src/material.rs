//! Direct semantic surface preparation for the Bevy PBR pipeline.
mod animation;
mod compile;
mod model;
mod particle;
pub(crate) use particle::{particle_material_layers, ParticleComposition};
mod programmed;
#[path = "ribbon_material.rs"]
mod ribbon_material;
mod world;
#[cfg(test)]
use self::world::compile_world;
use animation::layered_uv;
pub(super) use animation::mesh_alpha_mode;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, CompareFunction,
    RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
pub(crate) use compile::compile_material_with_images;
use compile::rasterization;
#[cfg(test)]
use compile::{blend, depth};
#[cfg(test)]
use model::legacy_combiner;
pub(crate) use ribbon_material::effect_state;
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(test)]
use wc3::model::materials::world as world_surface;
use wc3::model::materials::{layered, state::Lighting};
use wc3::model::values::model::ParticleBlend;
use wc3::model::{Animated, TextureId, TextureTransform};

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
    Premultiplied,
}

/// Semantic fixed-function state without a dependency on any native model codec.
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(ModelMaterialKey)]
pub struct ModelMaterialExtension {
    #[uniform(100)]
    combiner: CombinerUniform,
    #[texture(101)]
    #[sampler(102)]
    secondary: Option<Handle<Image>>,
    #[uniform(103)]
    hd: HdUniform,
    #[texture(104)]
    #[sampler(105)]
    orm: Option<Handle<Image>>,
    #[texture(106)]
    #[sampler(107)]
    team_color: Option<Handle<Image>>,
    #[texture(108)]
    #[sampler(109)]
    environment: Option<Handle<Image>>,
    #[texture(111)]
    #[sampler(112)]
    tertiary: Option<Handle<Image>>,
    #[uniform(110)]
    world: WorldUniform,
    #[uniform(113)]
    particle: ParticleUniform,
    blend: Option<Blend>,
    depth_test: bool,
    depth_write: Option<bool>,
}

#[derive(Clone, Copy, Debug, Default, ShaderType)]
struct WorldUniform {
    // x = world vertex illumination, y = ignore diffuse-map alpha.
    flags: UVec4,
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
    coordinates: UVec4,
    weights: Vec4,
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
            coordinates: UVec4::ZERO,
            weights: Vec4::ONE,
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
            hd: HdUniform::default(),
            orm: None,
            team_color: None,
            environment: None,
            world: WorldUniform::default(),
            particle: ParticleUniform::default(),
            blend: None,
            depth_test: true,
            depth_write: None,
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
}

impl From<&ModelMaterialExtension> for ModelMaterialKey {
    fn from(state: &ModelMaterialExtension) -> Self {
        Self {
            third_uv: state.particle.mode.x != 0,
            generated_coordinates: state.combiner.coordinates.x != 0
                || state.combiner.coordinates.y != 0,
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
            Blend::Premultiplied => Some((BlendFactor::One, BlendFactor::OneMinusSrcAlpha)),
        };
        *target = factors.map(|(src_factor, dst_factor)| BlendState {
            color: BlendComponent {
                src_factor,
                dst_factor,
                operation: BlendOperation::Add,
            },
            alpha: BlendComponent::OVER,
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
            key.bind_group_data.generated_coordinates,
            key.bind_group_data.third_uv,
        )?;
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
    untinted: Option<LinearRgba>,
    untinted_alpha_mode: Option<AlphaMode>,
    mesh: Option<MeshAnimation>,
    color: Option<Animated<[f32; 3]>>,
    rgba: Option<Animated<[f32; 4]>>,
    opacity: Option<Animated<f32>>,
    weight: Option<Animated<f32>>,
    texture: Option<Animated<Option<TextureId>>>,
    transform: Option<TextureTransform>,
    secondary_transform: Option<TextureTransform>,
    uv_scroll: [[f32; 2]; 2],
    third_transform: Option<TextureTransform>,
    input_weights: [Option<Animated<f32>>; 3],
    emissive_gain: Option<Animated<f32>>,
    emission: Option<Animated<[f32; 4]>>,
    emission_scale: f32,
    images: Arc<HashMap<TextureId, Handle<Image>>>,
    centered_transform: bool,
    hd: Option<HdAnimation>,
}

#[derive(Clone)]
struct HdAnimation {
    slots: [Option<Animated<Option<TextureId>>>; 6],
    fresnel: Option<layered::Fresnel>,
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
        }
    }
}

#[cfg(test)]
#[path = "material_tests.rs"]
mod tests;
