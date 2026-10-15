//! Direct semantic surface preparation for the Bevy PBR pipeline.
use bevy::math::Affine2;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{
    ExtendedMaterial, MaterialExtension, MaterialExtensionKey, MaterialExtensionPipeline,
};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, CompareFunction, Face,
    RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
};
use bevy::shader::ShaderRef;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use wc3::model::materials::state::{BlendOperations, Lighting, Operation};
use wc3::model::materials::{diablo3, layered, model_shader, programmed, world, SurfaceSet};
use wc3::model::profiles::Universal;
use wc3::model::values::model::{ModelBlend, ParticleBlend};
use wc3::model::{
    Animated, ColorSpace, MaterialId, Model, SampledAnimation, TextureId, TextureTransform,
};

use super::mesh_animation::MeshAnimation;
use super::mesh_attributes::specialize_extra_joints;
use super::textures::{TextureImages, TextureRole};
use crate::assets::ModelError;

/// One authored draw pass, including any limits of its PBR interpretation.
pub(crate) struct SurfacePass {
    pub material: StandardMaterial,
    pub coordinate: u32,
    pub diagnostics: Vec<String>,
    pub animation: SurfaceAnimation,
    pub state: UniversalMaterialExtension,
    pub secondary_coordinate: Option<u32>,
}

/// A semantic material retaining Bevy PBR shading and authored draw state.
pub type UniversalMaterial = ExtendedMaterial<StandardMaterial, UniversalMaterialExtension>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum UniversalBlend {
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
#[bind_group_data(UniversalMaterialKey)]
pub struct UniversalMaterialExtension {
    #[uniform(100)]
    combiner: CombinerUniform,
    #[texture(101)]
    #[sampler(102)]
    secondary: Option<Handle<Image>>,
    blend: UniversalBlend,
    depth_test: bool,
    depth_write: Option<bool>,
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct CombinerUniform {
    uv_transform: Mat3,
    // x: enabled, y: first alpha, z: second alpha, w: RGB/alpha doubling.
    mode: UVec4,
    clip: Vec4,
}
impl Default for CombinerUniform {
    fn default() -> Self {
        Self {
            uv_transform: Mat3::IDENTITY,
            mode: UVec4::ZERO,
            clip: Vec4::ZERO,
        }
    }
}

pub(crate) fn particle_material(
    blend: ParticleBlend,
    texture: Option<Handle<Image>>,
    lit: bool,
) -> UniversalMaterial {
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
        ParticleBlend::Add => {
            pass.material.alpha_mode = AlphaMode::Blend;
            pass.state.blend = UniversalBlend::Additive;
        }
        ParticleBlend::AddAlphaKey => {
            pass.material.alpha_mode = AlphaMode::Blend;
            pass.state.blend = UniversalBlend::AdditiveAlpha;
            pass.state.combiner.clip.x = 1.0 / 255.0;
        }
        ParticleBlend::Alpha | ParticleBlend::Unknown(_) => {
            pass.material.alpha_mode = AlphaMode::Blend;
            pass.state.blend = UniversalBlend::Alpha;
        }
    }
    pass.state.depth_write = Some(matches!(
        blend,
        ParticleBlend::Opaque | ParticleBlend::AlphaKey
    ));
    UniversalMaterial {
        base: pass.material,
        extension: pass.state,
    }
}

impl Default for UniversalMaterialExtension {
    fn default() -> Self {
        Self {
            combiner: CombinerUniform::default(),
            secondary: None,
            blend: UniversalBlend::Opaque,
            depth_test: true,
            depth_write: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Pipeline identity for semantic blending and depth operations.
pub struct UniversalMaterialKey {
    blend: UniversalBlend,
    depth_test: bool,
    depth_write: bool,
}

impl From<&UniversalMaterialExtension> for UniversalMaterialKey {
    fn from(state: &UniversalMaterialExtension) -> Self {
        Self {
            blend: state.blend,
            depth_test: state.depth_test,
            depth_write: state
                .depth_write
                .unwrap_or(state.blend == UniversalBlend::Opaque),
        }
    }
}

impl MaterialExtension for UniversalMaterialExtension {
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
        "embedded://bevy_wc3/shaders/universal_material.wgsl".into()
    }
    fn deferred_fragment_shader() -> ShaderRef {
        Self::fragment_shader()
    }
    fn prepass_fragment_shader() -> ShaderRef {
        "embedded://bevy_wc3/shaders/universal_material_prepass.wgsl".into()
    }
    fn specialize(
        _pipeline: &MaterialExtensionPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialExtensionKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        specialize_extra_joints(descriptor, layout)?;
        let state = key.bind_group_data;
        if let Some(depth) = &mut descriptor.depth_stencil {
            if !state.depth_test {
                depth.depth_compare = Some(CompareFunction::Always);
            }
            depth.depth_write_enabled = Some(state.depth_write);
        }
        if let Some(target) = descriptor
            .fragment
            .as_mut()
            .and_then(|fragment| fragment.targets.first_mut())
            .and_then(Option::as_mut)
        {
            let factors = match state.blend {
                UniversalBlend::Opaque => None,
                UniversalBlend::Alpha => {
                    Some((BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha))
                }
                UniversalBlend::Additive => Some((BlendFactor::One, BlendFactor::One)),
                UniversalBlend::AdditiveAlpha => Some((BlendFactor::SrcAlpha, BlendFactor::One)),
                UniversalBlend::Modulate => Some((BlendFactor::Zero, BlendFactor::Src)),
                UniversalBlend::Modulate2x => Some((BlendFactor::Dst, BlendFactor::Src)),
                UniversalBlend::Premultiplied => {
                    Some((BlendFactor::One, BlendFactor::OneMinusSrcAlpha))
                }
            };
            target.blend = factors.map(|(src_factor, dst_factor)| BlendState {
                color: BlendComponent {
                    src_factor,
                    dst_factor,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent::OVER,
            });
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
    emissive_gain: Option<Animated<f32>>,
    emission: Option<Animated<[f32; 4]>>,
    emission_scale: f32,
    images: Arc<HashMap<TextureId, Handle<Image>>>,
}

pub(super) fn mesh_alpha_mode(authored: AlphaMode, opacity: f32) -> AlphaMode {
    if opacity > 0. && opacity < 1. && matches!(authored, AlphaMode::Opaque | AlphaMode::Mask(_)) {
        AlphaMode::AlphaToCoverage
    } else {
        authored
    }
}

impl SurfaceAnimation {
    pub(super) fn with_mesh(
        mut self,
        untinted: Color,
        alpha_mode: AlphaMode,
        mesh: Option<&MeshAnimation>,
    ) -> Self {
        self.untinted = Some(untinted.to_linear());
        self.untinted_alpha_mode = Some(alpha_mode);
        self.mesh = mesh.cloned();
        self
    }

    pub(crate) fn is_animated(&self) -> bool {
        self.mesh.as_ref().is_some_and(MeshAnimation::is_animated)
            || self
                .rgba
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .emission
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .color
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .opacity
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .weight
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .texture
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self
                .emissive_gain
                .as_ref()
                .is_some_and(|value| value.property.is_some())
            || self.transform.as_ref().is_some_and(|value| {
                value.translation.property.is_some()
                    || value.rotation.property.is_some()
                    || value.scale.property.is_some()
            })
            || self.secondary_transform.as_ref().is_some_and(|value| {
                value.translation.property.is_some()
                    || value.rotation.property.is_some()
                    || value.scale.property.is_some()
            })
    }
    pub(crate) fn apply_to_assets(
        &self,
        sample: &SampledAnimation<'_, '_>,
        handle: &Handle<UniversalMaterial>,
        materials: &mut Assets<UniversalMaterial>,
    ) -> bool {
        let Some(mut material) = materials.get_mut(handle) else {
            return false;
        };
        let changed = self.apply(sample, &mut material.bypass_change_detection().base);
        let mut changed = changed;
        if let Some(transform) = &self.secondary_transform {
            let next = sampled_transform(sample, transform);
            let next = Mat3::from(next);
            if material.extension.combiner.uv_transform != next {
                material
                    .bypass_change_detection()
                    .extension
                    .combiner
                    .uv_transform = next;
                changed = true;
            }
        }
        if changed {
            let _ = material.into_inner();
        }
        changed
    }
    /// Updates only changed values and reports whether the material needs a write.
    pub(crate) fn apply(
        &self,
        sample: &SampledAnimation<'_, '_>,
        material: &mut StandardMaterial,
    ) -> bool {
        let mut changed = false;
        if self.color.is_some()
            || self.rgba.is_some()
            || self.opacity.is_some()
            || self.weight.is_some()
            || self.mesh.is_some()
        {
            let rgb = self.color.as_ref().map(|v| sample.animated(v));
            let rgba = self.rgba.as_ref().map(|v| sample.animated(v));
            let current = self
                .untinted
                .unwrap_or_else(|| material.base_color.to_linear());
            let rgb = rgb.or(rgba.map(|v| [v[0], v[1], v[2]])).unwrap_or([
                current.red,
                current.green,
                current.blue,
            ]);
            let alpha = self
                .opacity
                .as_ref()
                .map(|v| sample.animated(v))
                .unwrap_or_else(|| {
                    if self.rgba.is_none() && self.weight.is_none() {
                        current.alpha
                    } else {
                        1.
                    }
                })
                * rgba.map_or(1.0, |v| v[3])
                * self
                    .weight
                    .as_ref()
                    .map(|v| sample.animated(v))
                    .unwrap_or(1.0);
            let tint = self
                .mesh
                .as_ref()
                .map_or([1.; 4], |mesh| mesh.sample(sample));
            let next = Color::linear_rgba(
                rgb[0] * tint[0],
                rgb[1] * tint[1],
                rgb[2] * tint[2],
                alpha * tint[3],
            );
            if material.base_color != next {
                material.base_color = next;
                changed = true;
            }
            if self.mesh.is_some() {
                if let Some(authored) = self.untinted_alpha_mode {
                    let mode = mesh_alpha_mode(authored, tint[3]);
                    if material.alpha_mode != mode {
                        material.alpha_mode = mode;
                        changed = true;
                    }
                }
            }
        }
        if let Some(texture) = &self.texture {
            let selected = sample.animated(texture);
            let next = selected.and_then(|id| self.images.get(&id));
            if material.base_color_texture.as_ref() != next {
                material.base_color_texture = next.cloned();
                changed = true;
            }
        }
        if let Some(transform) = &self.transform {
            let translation = sample.animated(&transform.translation);
            let scale = sample.animated(&transform.scale);
            let rotation = Quat::from_array(sample.animated(&transform.rotation).0);
            let (_, _, z) = rotation.to_euler(EulerRot::XYZ);
            let next = Affine2::from_scale_angle_translation(
                Vec2::new(scale[0], scale[1]),
                z,
                Vec2::new(translation[0], translation[1]),
            );
            if material.uv_transform != next {
                material.uv_transform = next;
                changed = true;
            }
        }
        if let Some(gain) = &self.emissive_gain {
            let gain = sample.animated(gain);
            let next = LinearRgba::rgb(gain, gain, gain);
            if material.emissive != next {
                material.emissive = next;
                changed = true;
            }
            if material.emissive_texture != material.base_color_texture {
                material.emissive_texture = material.base_color_texture.clone();
                changed = true;
            }
        }
        if let Some(emission) = &self.emission {
            let value = sample.animated(emission);
            let next = LinearRgba::rgb(
                value[0] * self.emission_scale,
                value[1] * self.emission_scale,
                value[2] * self.emission_scale,
            );
            if material.emissive != next {
                material.emissive = next;
                changed = true;
            }
        }
        changed
    }
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
            state: UniversalMaterialExtension::default(),
            secondary_coordinate: None,
        }
    }
}

fn sampled_transform(sample: &SampledAnimation<'_, '_>, transform: &TextureTransform) -> Affine2 {
    let translation = sample.animated(&transform.translation);
    let scale = sample.animated(&transform.scale);
    let rotation = Quat::from_array(sample.animated(&transform.rotation).0);
    let (_, _, z) = rotation.to_euler(EulerRot::XYZ);
    Affine2::from_scale_angle_translation(
        Vec2::new(scale[0], scale[1]),
        z,
        Vec2::new(translation[0], translation[1]),
    )
}

pub(crate) fn compile_material_with_images(
    model: &Model<'_, Universal>,
    id: MaterialId,
    images: Arc<TextureImages>,
) -> Result<Vec<SurfacePass>, ModelError> {
    let resolved_images = images.as_ref();
    let material = model
        .material(id)
        .ok_or_else(|| ModelError(format!("missing semantic material {id:?}")))?;
    let mut passes = match &material.surface {
        SurfaceSet::Layered(surface) => surface
            .layers
            .iter()
            .map(|layer| compile_layer(model, surface, layer, resolved_images))
            .collect::<Result<Vec<_>, _>>()?,
        SurfaceSet::ModelShader(surface) => {
            vec![compile_model_shader(model, surface, resolved_images)?]
        }
        SurfaceSet::World(surface) => vec![compile_world(model, surface, resolved_images)?],
        SurfaceSet::Programmed(surface) => {
            vec![compile_programmed(model, surface, resolved_images)?]
        }
        SurfaceSet::D3(surface) => vec![compile_diablo3(surface)],
        SurfaceSet::D4(surface) => {
            let mut pass = SurfacePass::default();
            pass.diagnostics.push(match &surface.shader {
                Some(shader) => format!(
                    "Diablo IV shader map {} needs an application shader interpreter; texture semantic codes are unresolved",
                    shader.shader_map
                ),
                None => "Diablo IV external material shader is unresolved".into(),
            });
            vec![pass]
        }
    };
    for pass in &mut passes {
        pass.animation.images = images.color_images().clone();
        for diagnostic in &mut pass.diagnostics {
            *diagnostic = format!("material {id:?} ({}): {diagnostic}", material.name);
        }
    }
    Ok(passes)
}

fn image(
    model: &Model<'_, Universal>,
    id: Option<TextureId>,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    image_role(model, id, TextureRole::Color, images, diagnostics)
}

fn image_role(
    model: &Model<'_, Universal>,
    id: Option<TextureId>,
    role: TextureRole,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    let Some(id) = id else { return Ok(None) };
    let texture = model
        .texture(id)
        .ok_or_else(|| ModelError(format!("missing semantic texture {id:?}")))?;
    let resolved = images.get(id, texture.color_space, role).cloned();
    if resolved.is_none() {
        diagnostics.push(format!(
            "texture {id:?} resource {:?} with role {role:?} is unresolved",
            texture.resource
        ));
    }
    if let ColorSpace::Unknown(code) = texture.color_space {
        diagnostics.push(format!(
            "texture {id:?} transfer-function selector {code} is unresolved; using role {role:?}"
        ));
    }
    Ok(resolved)
}

fn blend<E: Debug, C, A, U: Debug>(blend: &BlendOperations<E, C, A, U>, pass: &mut SurfacePass) {
    pass.state.blend = match blend {
        BlendOperations::Opaque | BlendOperations::AlphaKey(_) => UniversalBlend::Opaque,
        BlendOperations::Alpha => UniversalBlend::Alpha,
        BlendOperations::Additive => UniversalBlend::Additive,
        BlendOperations::AdditiveAlpha(_) => UniversalBlend::AdditiveAlpha,
        BlendOperations::Modulate => UniversalBlend::Modulate,
        BlendOperations::Modulate2x => UniversalBlend::Modulate2x,
        _ => UniversalBlend::Alpha,
    };
    pass.material.alpha_mode = match blend {
        BlendOperations::Opaque => AlphaMode::Opaque,
        BlendOperations::Alpha => AlphaMode::Blend,
        BlendOperations::AlphaKey(_) => AlphaMode::Mask(128.0 / 255.0),
        // Keep straight-alpha shader output. Bevy's Add/Multiply modes modify
        // fragment RGB/alpha before our independent blend equation runs.
        BlendOperations::AdditiveAlpha(_)
        | BlendOperations::Additive
        | BlendOperations::Modulate
        | BlendOperations::Modulate2x => AlphaMode::Blend,
        BlendOperations::Extension(value) => {
            pass.diagnostics.push(format!(
                "blend operation {value:?} requires a custom pipeline; using alpha blending"
            ));
            AlphaMode::Blend
        }
        BlendOperations::Unknown(value) => {
            pass.diagnostics.push(format!(
                "unknown blend selector {value:?}; using alpha blending"
            ));
            AlphaMode::Blend
        }
    };
}

fn rasterization(pass: &mut SurfacePass, two_sided: bool, lighting: Lighting) {
    pass.material.double_sided = two_sided;
    pass.material.cull_mode = if two_sided { None } else { Some(Face::Back) };
    pass.material.unlit = lighting.unlit;
    pass.material.fog_enabled = !lighting.unfogged;
}

fn depth(pass: &mut SurfacePass, test: Operation, write: Operation) {
    pass.state.depth_test = !test.is_bypassed();
    pass.state.depth_write = Some(!write.is_bypassed());
}

fn transform(pass: &mut SurfacePass, transform: &TextureTransform) {
    let translation = transform.translation.default;
    let scale = transform.scale.default;
    let rotation = Quat::from_array(transform.rotation.default.0);
    let (x, y, z) = rotation.to_euler(EulerRot::XYZ);
    pass.material.uv_transform = Affine2::from_scale_angle_translation(
        Vec2::new(scale[0], scale[1]),
        z,
        Vec2::new(translation[0], translation[1]),
    );
    if x.abs() > 1e-5 || y.abs() > 1e-5 {
        pass.diagnostics
            .push("texture rotation outside the UV plane requires generated coordinates".into());
    }
}

fn compile_layer(
    model: &Model<'_, Universal>,
    surface: &layered::LayeredSurface<'_>,
    layer: &layered::Layer,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    blend(&layer.state.blend, &mut pass);
    let raster = &layer.state.rasterization;
    rasterization(
        &mut pass,
        surface.two_sided || raster.faces,
        raster.lighting,
    );
    depth(&mut pass, raster.depth.test, raster.depth.write);
    pass.coordinate = layer.binding.coordinates.source;
    pass.material.base_color = Color::linear_rgba(1.0, 1.0, 1.0, layer.opacity.default);
    pass.material.base_color_texture = image(
        model,
        layer.binding.image.default,
        images,
        &mut pass.diagnostics,
    )?;
    pass.animation.opacity = Some(layer.opacity.clone());
    pass.animation.texture = Some(layer.binding.image.clone());
    pass.animation.emissive_gain = layer.emissive_gain.clone();
    pass.material.depth_bias = surface.priority_plane as f32;
    if let Some(id) = layer.binding.coordinates.transform {
        let uv = model
            .texture_transform(id)
            .ok_or_else(|| ModelError(format!("missing texture transform {id:?}")))?;
        transform(&mut pass, uv);
        pass.animation.transform = Some(uv.clone());
    }
    if raster.controls.sphere_environment {
        pass.diagnostics
            .push("sphere-environment coordinates require a generated-coordinate shader".into());
    }
    if !surface.shader.is_empty()
        || layer.shader.is_some()
        || !layer.textures.is_empty()
        || layer.fresnel.is_some()
    {
        pass.diagnostics.push(format!(
            "extended layered shader {:?} needs its authored texture-slot and Fresnel interpreter",
            surface.shader
        ));
    }
    if surface.constant_color || surface.sort_near_z || surface.sort_far_z {
        pass.diagnostics.push(
            "constant-color or primitive-level sorting controls are not evaluated by this PBR pass"
                .into(),
        );
    }
    if let Some(gain) = &layer.emissive_gain {
        pass.material.emissive = LinearRgba::rgb(gain.default, gain.default, gain.default);
        pass.material.emissive_texture = pass.material.base_color_texture.clone();
    }
    Ok(pass)
}

fn compile_model_shader(
    model: &Model<'_, Universal>,
    surface: &model_shader::ShaderPass<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    if matches!(
        surface.state.blend,
        BlendOperations::Extension(ModelBlend::BlendAdd)
    ) {
        pass.state.blend = UniversalBlend::Premultiplied;
        pass.material.alpha_mode = AlphaMode::Premultiplied;
    } else {
        blend(&surface.state.blend, &mut pass);
    }
    let raster = &surface.state.rasterization;
    rasterization(&mut pass, raster.faces, raster.lighting);
    depth(&mut pass, raster.depth.test, raster.depth.write);
    let color = surface.color.default;
    pass.animation.color = Some(surface.color.clone());
    pass.animation.opacity = Some(surface.opacity.clone());
    pass.material.base_color =
        Color::linear_rgba(color[0], color[1], color[2], surface.opacity.default);
    pass.material.depth_bias = surface.state.ordering as f32;
    if let Some(input) = surface.textures.first() {
        pass.material.base_color_texture =
            image(model, input.image.texture, images, &mut pass.diagnostics)?;
        if input.coordinates.source < 0 {
            pass.diagnostics.push(format!(
                "M2 generated texture coordinates {} need a shader interpreter",
                input.coordinates.source
            ));
        } else {
            pass.coordinate = input.coordinates.source as u32;
        }
        transform(&mut pass, &input.coordinates.transform);
        pass.animation.transform = Some(input.coordinates.transform.clone());
        pass.animation.weight = Some(input.contribution.clone());
        pass.material
            .base_color
            .set_alpha(surface.opacity.default * input.contribution.default);
    }
    if let Some(mode) = legacy_combiner(surface.shader, surface.textures.len()).filter(|_| {
        surface
            .textures
            .iter()
            .all(|input| input.coordinates.source >= 0)
    }) {
        pass.state.combiner.mode = mode;
        if let Some(second) = surface.textures.get(1) {
            pass.state.secondary =
                image(model, second.image.texture, images, &mut pass.diagnostics)?;
            pass.secondary_coordinate = u32::try_from(second.coordinates.source).ok();
            let mut scratch = SurfacePass::default();
            transform(&mut scratch, &second.coordinates.transform);
            pass.state.combiner.uv_transform = Mat3::from(scratch.material.uv_transform);
            pass.diagnostics.extend(scratch.diagnostics);
            pass.animation.secondary_transform = Some(second.coordinates.transform.clone());
            if second.contribution.property.is_some() || second.contribution.default != 1.0 {
                pass.diagnostics.push(
                    "independent M2 secondary texture contribution weight is not evaluated".into(),
                );
            }
        }
    } else if surface.shader != 0 || surface.textures.len() > 1 {
        pass.diagnostics.push(format!("M2 shader {} with {} texture inputs is approximated by its first input; authored combiners are not evaluated", surface.shader, surface.textures.len()));
    }
    Ok(pass)
}

/// Decodes established legacy selector bits, keeping external shader tables opaque.
/// Bit semantics were researched independently from the noclip M2 skin renderer;
/// the implementation here operates directly on our semantic texture inputs.
fn legacy_combiner(shader: u16, inputs: usize) -> Option<UVec4> {
    let lower = shader & 7;
    if shader & 0x8088 != 0 || !(inputs == 1 || (inputs == 2 && !matches!(lower, 3 | 7))) {
        return None;
    }
    Some(UVec4::new(
        1,
        u32::from(shader & 0x70 != 0),
        u32::from(inputs == 2 && lower != 0 && lower != 6),
        u32::from(inputs == 2 && matches!(lower, 4 | 6)),
    ))
}

fn compile_world(
    model: &Model<'_, Universal>,
    surface: &world::Material<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    blend(&surface.state.blend, &mut pass);
    let raster = &surface.state.rasterization;
    rasterization(&mut pass, raster.faces, raster.lighting);
    pass.material.base_color_texture = image(
        model,
        surface.textures.first().and_then(|slot| slot.image),
        images,
        &mut pass.diagnostics,
    )?;
    if raster.controls.self_illuminated {
        let color = surface.self_illumination;
        pass.material.emissive = LinearRgba::rgb(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
        );
    }
    if surface
        .shader
        .is_some_and(|shader| shader != 1 && shader != 6)
        || surface.textures.len() > 1
    {
        pass.diagnostics.push(format!("WMO shader {:?} needs authored texture combiners and baked-light interpretation; showing its first diffuse input", surface.shader));
    }
    if surface.uv_scroll != [[0.0; 2]; 2] {
        pass.diagnostics
            .push("WMO UV scroll requires runtime surface animation".into());
    }
    Ok(pass)
}

fn compile_programmed(
    model: &Model<'_, Universal>,
    surface: &programmed::Material<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    if let programmed::Material::Standard(surface) = surface {
        blend(&surface.state.blend, &mut pass);
        if let Some(diffuse) = &surface.diffuse.input {
            pass.material.base_color_texture =
                image(model, diffuse.binding.image, images, &mut pass.diagnostics)?;
            let color = diffuse.binding.contribution.color.initial;
            pass.material.base_color = Color::linear_rgba(
                color[0],
                color[1],
                color[2],
                color[3] * diffuse.binding.contribution.map_alpha.initial,
            );
            pass.animation.rgba = Some(Animated {
                default: color,
                property: diffuse.binding.contribution.color.binding.clone(),
            });
            pass.animation.opacity = Some(Animated {
                default: diffuse.binding.contribution.map_alpha.initial,
                property: diffuse.binding.contribution.map_alpha.binding.clone(),
            });
        }
        if let Some(normal) = &surface.normals.base {
            pass.material.normal_map_texture = image_role(
                model,
                normal.binding.image,
                TextureRole::Normal,
                images,
                &mut pass.diagnostics,
            )?;
        }
        if let Some(emissive) = &surface.emissive.input.primary.input {
            pass.material.emissive_texture =
                image(model, emissive.binding.image, images, &mut pass.diagnostics)?;
            let color = emissive.binding.contribution.color.initial;
            let gain = surface.emissive.parameters;
            pass.material.emissive =
                LinearRgba::rgb(color[0] * gain, color[1] * gain, color[2] * gain);
            pass.animation.emission = Some(Animated {
                default: color,
                property: emissive.binding.contribution.color.binding.clone(),
            });
            pass.animation.emission_scale = gain;
        }
        pass.diagnostics.push("M3 standard surface uses primary diffuse, normal and emission roles; shader controls, layer composition, opacity programs and specular response need an interpreter".into());
    } else {
        pass.diagnostics
            .push("M3 material family requires a dedicated shader interpreter".into());
    }
    Ok(pass)
}

fn compile_diablo3(surface: &diablo3::Material<'_>) -> SurfacePass {
    let mut pass = SurfacePass::default();
    let color = &surface.uber.colors;
    pass.material.base_color = Color::linear_rgba(
        color.diffuse[0],
        color.diffuse[1],
        color.diffuse[2],
        color.diffuse[3],
    );
    pass.material.emissive =
        LinearRgba::rgb(color.emissive[0], color.emissive[1], color.emissive[2]);
    pass.diagnostics.push(format!("Diablo III shader map {} requires an application shader interpreter; authored constant diffuse/emission are displayed, texture-role codes and render passes remain unresolved", surface.uber.shader_map.id));
    pass
}

#[cfg(test)]
#[path = "material_tests.rs"]
mod tests;
