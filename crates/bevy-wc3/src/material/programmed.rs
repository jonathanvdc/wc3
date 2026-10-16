//! Fixed M3 standard-material roles evaluated before Bevy PBR lighting.
use std::array::from_fn;
pub(super) mod layers;
mod samplers;
use super::compile::{blend, image_role_with_sampler};
use super::SurfacePass;
use crate::animation_targets::AnimationTargets;
use crate::textures::{programmed_sampler, TextureImages, TextureRole};
use crate::ModelError;
use bevy::material::OpaqueRendererMethod;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use layers::{layer_uniform, ProgrammedAnimation, ROLE_COUNT};
use samplers::SamplerPool;
use wc3::model::materials::programmed;
use wc3::model::materials::state::BlendOperations;
use wc3::model::profiles::Universal;
use wc3::model::values::programmed::{ColorChannel, LayerBlendOp, SpecularMode, UvMappingMode};
use wc3::model::Model;

/// Application-owned colours for M3 standard team-colour operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TeamColors {
    /// Linear RGB diffuse team colour.
    pub diffuse: Vec3,
    /// Linear RGB emission team colour.
    pub emissive: Vec3,
    /// Exponent applied to the diffuse team mask.
    pub diffuse_exponent: f32,
    /// Exponent applied by team-emission blend operations.
    pub emissive_exponent: f32,
}
impl Default for TeamColors {
    fn default() -> Self {
        Self {
            diffuse: Vec3::ONE,
            emissive: Vec3::ONE,
            diffuse_exponent: 1.,
            emissive_exponent: 1.,
        }
    }
}

#[cfg(test)]
pub(super) fn compile_programmed(
    model: &Model<'_, Universal>,
    surface: &programmed::Material<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    compile_programmed_with_targets(model, surface, images, &AnimationTargets::new(model))
}

pub(super) fn compile_programmed_with_targets(
    model: &Model<'_, Universal>,
    surface: &programmed::Material<'_>,
    images: &TextureImages,
    targets: &AnimationTargets,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    let programmed::Material::Standard(surface) = surface else {
        pass.diagnostics.push(format!(
            "M3 {} material requires a dedicated shader interpreter",
            family_name(surface)
        ));
        return Ok(pass);
    };
    blend(&surface.state.blend, &mut pass);
    let threshold = surface.state.rasterization.alpha_test_threshold;
    if threshold != 0 {
        let cutoff = threshold.min(255) as f32 / 255.;
        pass.state.combiner.clip.x = cutoff;
        if matches!(surface.state.blend, BlendOperations::Opaque) {
            pass.material.alpha_mode = AlphaMode::Mask(cutoff);
        }
        if threshold > 255 {
            pass.diagnostics.push("M3 alpha-test threshold exceeds its native byte range; rendering uses the maximum cutoff".into());
        }
    }
    for (name, operation) in [
        ("decal/environment", surface.layer_blend),
        (
            "primary emission",
            surface.emissive.input.primary.parameters,
        ),
        (
            "secondary emission",
            surface.emissive.input.secondary.parameters,
        ),
    ] {
        if let LayerBlendOp::Unknown(code) = operation {
            pass.diagnostics.push(format!("M3 {name} uses unresolved layer blend operation {code}; this operation does not change its accumulator"));
        }
    }
    if let SpecularMode::Unknown(code) = surface.specular.parameters.mode {
        pass.diagnostics.push(format!(
            "M3 specular computation mode {code} is unresolved; its specular role is not evaluated"
        ));
    }
    let controls = &surface.state.rasterization;
    for (enabled, name) in [
        (
            controls.additional_flags.depth_blend_falloff(),
            "scene-depth blend falloff",
        ),
        (controls.flags.simulate_roughness(), "simulated roughness"),
        (controls.flags.double_lambert(), "double-Lambert lighting"),
        (controls.flags.terrain_hdr(), "terrain HDR lighting"),
        (
            controls.flags.transparent_depth_effects(),
            "transparent depth effects",
        ),
        (controls.flags.decal_lighting(), "decal lighting"),
        (controls.flags.clamp_output(), "native output clamping"),
    ] {
        if enabled {
            pass.diagnostics.push(format!(
                "M3 standard {name} controls are not interpreted by the Bevy PBR adaptation"
            ));
        }
    }
    if controls.rtt_channels.bits() != 0 {
        pass.diagnostics
            .push("M3 render-target channel masks are not applied to Bevy render targets".into());
    }
    let flags = surface.state.rasterization.flags;
    let additional = surface.state.rasterization.additional_flags;
    pass.material.double_sided = flags.two_sided();
    pass.material.cull_mode = if pass.material.double_sided {
        None
    } else {
        Some(Face::Back)
    };
    pass.material.unlit = flags.unshaded();
    // Native post-lighting emission operations require the forward lighting result.
    pass.material.opaque_render_method = OpaqueRendererMethod::Forward;
    pass.material.fog_enabled = flags.bits() & (1 << 13) == 0;
    pass.material.depth_bias = surface.state.ordering as f32;
    pass.state.cast_shadows = !flags.no_shadows_cast();
    pass.state.receive_shadows = !flags.no_shadows_receive();
    pass.state.programmed.settings = UVec4::new(
        1,
        u32::from(flags.vertex_color() || additional.vertex_color()),
        u32::from(flags.vertex_alpha() || additional.vertex_alpha()),
        surface.specular.parameters.mode.raw(),
    );
    pass.state.programmed.blends = UVec4::new(
        surface.layer_blend.raw(),
        surface.emissive.input.primary.parameters.raw(),
        surface.emissive.input.secondary.parameters.raw(),
        flags.bits(),
    );
    pass.state.programmed.response = Vec4::new(
        surface.specular.parameters.exponent,
        surface.specular.parameters.hdr_multiplier,
        surface.emissive.parameters,
        if flags.bits() & 4 != 0 { 1. } else { 0. },
    );
    let sources: [Option<&programmed::TextureLayer>; ROLE_COUNT] = [
        surface.diffuse.as_ref(),
        surface.decal_layer.as_ref(),
        surface.specular.input.color.as_ref(),
        surface.specular.input.gloss.as_ref(),
        surface.emissive.input.primary.input.as_ref(),
        surface.emissive.input.secondary.input.as_ref(),
        surface.opacity.primary.as_ref(),
        surface.opacity.secondary.as_ref(),
        surface.normals.base.as_ref(),
        surface.normals.primary.input.as_ref(),
        surface.normals.primary.mask.as_ref(),
        surface.normals.secondary.input.as_ref(),
        surface.normals.secondary.mask.as_ref(),
        surface.light_map_layer.as_ref(),
        surface.ambient_occlusion_layer.as_ref(),
    ];
    let mut animations = ProgrammedAnimation {
        layers: from_fn(|_| None),
        weights: surface
            .normals
            .weights
            .iter()
            .take(8)
            .cloned()
            .map(|mut weight| {
                targets.retain_scalar_binding(weight.initial, &mut weight.binding);
                weight
            })
            .collect(),
    };
    let mut samplers = SamplerPool::default();
    for (slot, layer) in sources.into_iter().enumerate() {
        let Some(layer) = layer else {
            continue;
        };
        if slot == 2 && matches!(surface.specular.parameters.mode, SpecularMode::Unknown(_)) {
            continue;
        }
        if !layer.binding.sampling.flags.color() && layer.binding.image.is_none() {
            continue;
        }
        let route = if layer.binding.sampling.flags.color() {
            Some(0)
        } else {
            uv_route(layer.binding.coordinates.source)
        };
        let Some(route) = route else {
            pass.diagnostics.push(format!("M3 standard role {slot} uses unsupported coordinate generation {:?}; this role is not evaluated",layer.binding.coordinates.source));
            continue;
        };
        if matches!(
            layer.binding.contribution.color_type,
            ColorChannel::Unknown(_)
        ) {
            pass.diagnostics.push(format!("M3 standard role {slot} has an unresolved channel selector; this role is not evaluated"));
            continue;
        }
        diagnose_layer(layer, slot, targets, &mut pass.diagnostics);
        if layer.binding.sampling.flags.color() && matches!(slot, 8 | 9 | 11) {
            pass.diagnostics.push(format!("M3 standard normal role {slot} decodes authored constant AG values; native constant-normal parity remains unverified"));
        }
        let uniform = layer_uniform(layer, route, None);
        pass.state.programmed.layers[slot] = uniform;
        animations.layers[slot] = layers::animated_layer(layer, targets);
        if !layer.binding.sampling.flags.color() {
            let role = if matches!(slot, 0 | 1 | 2 | 4 | 5 | 13) {
                TextureRole::Color
            } else if matches!(slot, 8 | 9 | 11) {
                TextureRole::Normal
            } else {
                TextureRole::Data
            };
            let image = layer_image(model, layer, role, images, &mut pass.diagnostics)?;
            pass.state.programmed.layers[slot].flags.y = u32::from(image.is_some());
            if image.is_none() && slot != 0 {
                pass.state.programmed.layers[slot].flags.x = 0;
                animations.layers[slot] = None;
                continue;
            }
            if let Some(image) = image.as_ref() {
                let source_sampler = layer
                    .binding
                    .image
                    .and_then(|id| model.texture(id))
                    .map(|texture| texture.sampler)
                    .unwrap_or_default();
                let sampler = programmed_sampler(layer, source_sampler);
                let Some(selector) = samplers.intern(sampler, image) else {
                    pass.state.programmed.layers[slot].flags.x = 0;
                    animations.layers[slot] = None;
                    pass.diagnostics.push(format!("M3 role {slot} exceeds the shared sampler-state budget and is not evaluated"));
                    continue;
                };
                pass.state.programmed.layers[slot].route.y = selector;
                match route {
                    1 => pass.secondary_coordinate = Some(1),
                    2 => pass.third_coordinate = Some(2),
                    3 => {
                        pass.fourth_coordinate = Some(3);
                        pass.state.fourth_uv = true;
                    }
                    _ => {}
                }
            }

            match slot {
                0 => pass.material.base_color_texture = image,
                1 => pass.material.metallic_roughness_texture = image,
                2 => pass.material.occlusion_texture = image,
                3 => pass.material.emissive_texture = image,
                4 => pass.state.secondary = image,
                5 => pass.state.orm = image,
                6 => pass.state.team_color = image,
                7 => pass.state.environment = image,
                8 => pass.material.normal_map_texture = image,
                9 => pass.state.tertiary = image,
                10 => pass.state.fourth = image,
                11 => pass.state.normal_secondary = image,
                12 => pass.state.normal_secondary_mask = image,
                13 => pass.state.light_map = image,
                14 => pass.state.ambient_occlusion = image,
                _ => unreachable!(),
            }
        }
    }
    pass.state.programmed_samplers = samplers.images();
    for (index, weight) in surface.normals.weights.iter().take(8).enumerate() {
        pass.state.programmed.normal_weights[index / 4][index % 4] = weight.initial;
    }
    if surface.normals.weights.len() > 8 {
        pass.diagnostics.push("M3 normal blending has more than eight weights; the standard shader evaluates its first eight native mask channels".into());
    }
    if surface.height_layer.is_some()
        || surface.environment_layer.is_some()
        || surface.environment_mask_layer.is_some()
    {
        pass.diagnostics
            .push("M3 parallax height and environment reflection roles remain unsupported".into());
    }
    if surface.specular.input.color.is_some() {
        pass.diagnostics.push("M3 native specular intensity, tint and gloss exponent use a bounded Bevy PBR approximation; the native highlight distribution and energy are not reproduced".into());
    }
    if surface.light_map_layer.is_some() {
        pass.diagnostics.push("M3 lightmap adds its doubled diffuse contribution through Bevy emission; Bevy scene lighting remains active instead of being replaced by native baked illumination".into());
    }
    let required = if flags.bits() & (1 << 27) != 0 { 8 } else { 4 };
    if flags.bits() & 4 != 0 && surface.normals.weights.len() < required {
        pass.state.programmed.blends.w &= !4;
        pass.diagnostics.push(format!("M3 enabled normal blending needs {required} mask weights; its incomplete or version-ambiguous controls are not evaluated"));
    }
    if let (Some(first), Some(second)) = (
        &surface.normals.primary.input,
        &surface.normals.secondary.input,
    ) {
        if first.binding.coordinates != second.binding.coordinates {
            pass.diagnostics.push("M3 second additional normal uses its own authored UV controls; native sharing with the first additional normal remains unverified".into());
        }
    }
    pass.animation.programmed = animations.is_animated().then_some(animations);
    Ok(pass)
}

fn family_name(material: &programmed::Material<'_>) -> &'static str {
    match material {
        programmed::Material::Standard(_) => "Standard",
        programmed::Material::Displacement(_) => "Displacement",
        programmed::Material::Composite(_) => "Composite",
        programmed::Material::Terrain(_) => "Terrain",
        programmed::Material::Volume(_) => "Volume",
        programmed::Material::VolumeNoise(_) => "VolumeNoise",
        programmed::Material::Creep(_) => "Creep",
        programmed::Material::Hair(_) => "Hair",
        programmed::Material::SplatTerrainBake(_) => "SplatTerrainBake",
        programmed::Material::Reflection(_) => "Reflection",
        programmed::Material::LensFlare(_) => "LensFlare",
        programmed::Material::DataDriven(_) => "DataDriven",
    }
}
fn uv_route(mode: UvMappingMode) -> Option<u32> {
    match mode {
        UvMappingMode::ExplicitUV0 => Some(0),
        UvMappingMode::ExplicitUV1 => Some(1),
        UvMappingMode::ExplicitUV2 => Some(2),
        UvMappingMode::ExplicitUV3 => Some(3),
        _ => None,
    }
}
fn diagnose_layer(
    layer: &programmed::TextureLayer,
    slot: usize,
    targets: &AnimationTargets,
    diagnostics: &mut Vec<String>,
) {
    let transform = &layer.binding.coordinates.transform;
    if transform.uv_angle.initial[..2] != [0.; 2] || targets.contains(&transform.uv_angle.binding) {
        diagnostics.push(format!("M3 standard role {slot} evaluates Z UV rotation; authored XY rotation is not interpreted"));
    }
    if transform.uv_source_related != u32::MAX {
        diagnostics.push(format!("M3 standard role {slot} has unresolved shared UV-source controls; its own coordinate transform is used"));
    }
    let sampling = &layer.binding.sampling;
    if sampling.flags.video()
        || sampling.flags.particle_uv_flipbook()
        || sampling.noise_amplitude != 0.
        || sampling.texture_source != 0
        || sampling.poc_texture != 0
    {
        diagnostics.push(format!("M3 standard role {slot} evaluates a static image without authored video, flipbook, procedural noise or external-source controls"));
    }
    if layer.fresnel_mode.raw() != 0 {
        diagnostics.push(format!("M3 standard role {slot} has unresolved Fresnel preset controls; Fresnel attenuation is not evaluated"));
    }
}
fn layer_image(
    model: &Model<'_, Universal>,
    layer: &programmed::TextureLayer,
    role: TextureRole,
    images: &TextureImages,
    diagnostics: &mut Vec<String>,
) -> Result<Option<Handle<Image>>, ModelError> {
    let sampler = layer
        .binding
        .image
        .and_then(|id| model.texture(id))
        .map(|texture| texture.sampler)
        .unwrap_or_default();
    image_role_with_sampler(
        model,
        layer.binding.image,
        role,
        Some(programmed_sampler(layer, sampler)),
        images,
        diagnostics,
    )
}
#[cfg(test)]
#[path = "programmed_tests.rs"]
mod tests;
