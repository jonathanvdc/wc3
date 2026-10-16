//! M3 standard material roles interpreted within Bevy's PBR pipeline.
use super::compile::{blend, image_role_with_sampler};
use super::SurfacePass;
use crate::textures::{programmed_sampler, TextureImages, TextureRole};
use crate::ModelError;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use wc3::model::materials::programmed;
use wc3::model::profiles::Universal;
use wc3::model::values::programmed::{ColorChannel, SpecularMode, UvMappingMode};
use wc3::model::{Animated, Model};

pub(super) fn compile_programmed(
    model: &Model<'_, Universal>,
    surface: &programmed::Material<'_>,
    images: &TextureImages,
) -> Result<SurfacePass, ModelError> {
    let mut pass = SurfacePass::default();
    if let programmed::Material::Standard(surface) = surface {
        blend(&surface.state.blend, &mut pass);
        if let Some(diffuse) = &surface.diffuse.input {
            pass.material.base_color_texture = layer_image(
                model,
                diffuse,
                TextureRole::Color,
                images,
                &mut pass.diagnostics,
            )?;
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
            pass.material.normal_map_texture = layer_image(
                model,
                normal,
                TextureRole::Normal,
                images,
                &mut pass.diagnostics,
            )?;
            if !simple_sampling(normal) {
                pass.diagnostics.push("M3 base normal map uses AG decoding with UV0; its coordinate, sampling or contribution program is not evaluated".into());
            }
        }
        if let Some(emissive) = &surface.emissive.input.primary.input {
            pass.material.emissive_texture = layer_image(
                model,
                emissive,
                TextureRole::Color,
                images,
                &mut pass.diagnostics,
            )?;
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
        if pass.material.normal_map_texture.is_some() {
            // Native M3 normal maps store X in alpha and Y in green.
            // Mode two changes only normal decoding, leaving HD role routing off.
            pass.state.hd.maps.x = 2;
        }
        if let Some(ao) = &surface.ambient_occlusion_layer {
            if simple_red_data_layer(ao) {
                pass.material.occlusion_texture =
                    layer_image(model, ao, TextureRole::Data, images, &mut pass.diagnostics)?;
            } else {
                pass.diagnostics.push("M3 ambient occlusion layer uses unsupported coordinates, channel selection or contribution controls".into());
            }
        }
        constant_specular(surface, &mut pass);
        if surface.specular.input.gloss.is_some() {
            pass.diagnostics.push("M3 gloss map encoding is unresolved; the authored map is retained without assigning a guessed roughness channel".into());
        }
        pass.material.double_sided = surface.state.rasterization.flags.two_sided();
        pass.material.cull_mode = if pass.material.double_sided {
            None
        } else {
            Some(Face::Back)
        };
        pass.material.unlit = surface.state.rasterization.flags.unshaded();
        pass.material.depth_bias = surface.state.ordering as f32;
        pass.diagnostics.push("M3 standard surface evaluates primary diffuse/emission, alpha-green packed normals and simple red-channel ambient occlusion; layer composition, opacity programs and advanced shader controls remain unresolved".into());
    } else {
        pass.diagnostics
            .push("M3 material family requires a dedicated shader interpreter".into());
    }
    Ok(pass)
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

fn simple_red_data_layer(layer: &programmed::TextureLayer) -> bool {
    simple_sampling(layer)
        && matches!(
            layer.binding.contribution.color_type,
            ColorChannel::Rgb | ColorChannel::Rgba | ColorChannel::Red
        )
}

fn simple_sampling(layer: &programmed::TextureLayer) -> bool {
    let binding = &layer.binding;
    let transform = &binding.coordinates.transform;
    let flags = binding.sampling.flags;
    binding.coordinates.source == UvMappingMode::ExplicitUV0
        && transform.uv_offset.initial == [0.; 2]
        && transform.uv_angle.initial == [0.; 3]
        && transform.uv_tiling.initial == [1.; 2]
        && transform.uv_offset.binding.is_none()
        && transform.uv_angle.binding.is_none()
        && transform.uv_tiling.binding.is_none()
        && transform.uv_source_related == u32::MAX
        && binding.contribution.map_alpha.initial == 1.
        && binding.contribution.map_alpha.binding.is_none()
        && !flags.color()
        && !flags.color_invert()
        && !flags.color_add()
        && !flags.color_mult()
        && !flags.video()
        && !flags.particle_uv_flipbook()
        && binding.sampling.noise_amplitude == 0.
        && binding.sampling.texture_source == 0
        && binding.sampling.poc_texture == 0
        && flags.bits() & !0x2c == 0
        && layer.fresnel_mode.raw() == 0
}

fn constant_specular(surface: &programmed::Standard, pass: &mut SurfacePass) {
    let Some(layer) = &surface.specular.input.color else {
        return;
    };
    let contribution = &layer.binding.contribution;
    if layer.binding.image.is_some()
        || !layer.binding.sampling.flags.color()
        || contribution.color.binding.is_some()
        || contribution.map_alpha.initial != 1.
        || contribution.map_alpha.binding.is_some()
        || layer.binding.sampling.flags.color_invert()
        || layer.binding.sampling.flags.color_add()
        || layer.binding.sampling.flags.color_mult()
        || layer.binding.sampling.flags.bits() & !0x42c != 0
        || layer.fresnel_mode.raw() != 0
    {
        pass.diagnostics
            .push("M3 textured or programmed specular contribution requires an interpreter".into());
        return;
    }
    let mut color = contribution.color.initial;
    match contribution.color_type {
        ColorChannel::Rgb => color[3] = 1.,
        ColorChannel::Rgba => {}
        ColorChannel::Alpha => color = [color[3]; 4],
        ColorChannel::Red => color = [color[0]; 4],
        ColorChannel::Green => color = [color[1]; 4],
        ColorChannel::Blue => color = [color[2]; 4],
        ColorChannel::Unknown(_) => {
            pass.diagnostics
                .push("M3 specular channel selection is unresolved".into());
            return;
        }
    }
    if layer.binding.sampling.flags.color_clamp() {
        color = color.map(|value| value.clamp(0., 1.));
    }
    let rgb = match surface.specular.parameters.mode {
        SpecularMode::Rgb => [color[0], color[1], color[2]],
        SpecularMode::AlphaOnly => [color[3]; 3],
        SpecularMode::Unknown(_) => {
            pass.diagnostics
                .push("M3 specular computation mode is unresolved".into());
            return;
        }
    };
    let strength = rgb.into_iter().fold(0., f32::max);
    let gain = surface.specular.parameters.hdr_multiplier.max(0.);
    pass.material.reflectance = (strength * gain).clamp(0., 1.);
    if strength > 0. {
        pass.material.specular_tint =
            Color::linear_rgb(rgb[0] / strength, rgb[1] / strength, rgb[2] / strength);
    }
    // Blinn-Phong exponent n gives physical roughness sqrt(2/(n+2));
    // Bevy stores its square root as perceptual roughness. This preserves
    // approximate lobe width, not the native distribution or energy.
    pass.material.perceptual_roughness =
        (2. / (surface.specular.parameters.exponent.max(0.) + 2.)).powf(0.25);
    pass.diagnostics.push("M3 constant Blinn-Phong specular uses a bounded Bevy PBR approximation for intensity, tint and highlight width; native distribution and HDR energy are not reproduced".into());
}

#[cfg(test)]
#[path = "programmed_tests.rs"]
mod tests;
