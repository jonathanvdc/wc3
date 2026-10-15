//! Complete layered surfaces written directly to the selected Warcraft III dialect.
use super::encode_animation::Encoder;
use crate::model::conversion::Error;
use crate::model::mdlx::{self as native, ModelVersion};
use crate::model::semantic::profiles::mdlx as profile;
use crate::model::semantic::TextureId;
use std::collections::BTreeMap;

pub(super) fn write<'a, V: ModelVersion>(
    source: &profile::Material<'_>,
    textures: &BTreeMap<TextureId, u32>,
    encoder: &mut Encoder<'_, '_, '_>,
    path: &str,
) -> Result<native::Material<'a, V>, Error> {
    let map = |id: Option<TextureId>| {
        id.map(|id| {
            textures
                .get(&id)
                .copied()
                .ok_or_else(|| Error::new(path, "texture identity does not exist"))
        })
        .transpose()
        .map(|id| id.unwrap_or(u32::MAX))
    };
    let mut material = native::Material::<V>::new();
    material.priority_plane = source.priority_plane;
    material.render_mode = native::MaterialRenderFlags(source.unclassified.bits());
    material
        .render_mode
        .set_constant_color(source.constant_color);
    material.render_mode.set_two_sided(source.two_sided);
    material
        .render_mode
        .set_sort_primitives_near_z(source.sort_near_z);
    material
        .render_mode
        .set_sort_primitives_far_z(source.sort_far_z);
    material
        .render_mode
        .set_full_resolution(source.full_resolution);
    if let Err(error) = material.try_set_shader(&source.shader) {
        if !source.shader.is_empty() {
            encoder
                .context
                .unsupported(format!("{path}.shader"), error.to_string())?;
        }
    }
    for (index, source) in source.layers.iter().enumerate() {
        let path = format!("{path}.layers[{index}]");
        let mut layer = native::Layer::<V>::new();
        layer.filter_mode = blend(source.state.blend);
        layer.shading_flags =
            native::LayerShadingFlags(source.state.rasterization.controls.unclassified.bits());
        layer
            .shading_flags
            .set_unshaded(source.state.rasterization.lighting.unlit);
        layer
            .shading_flags
            .set_sphere_env_map(source.state.rasterization.controls.sphere_environment);
        layer
            .shading_flags
            .set_wrap_width(source.state.rasterization.controls.wrap_u);
        layer
            .shading_flags
            .set_wrap_height(source.state.rasterization.controls.wrap_v);
        layer
            .shading_flags
            .set_two_sided(source.state.rasterization.faces);
        layer
            .shading_flags
            .set_unfogged(source.state.rasterization.lighting.unfogged);
        layer
            .shading_flags
            .set_no_depth_test(source.state.rasterization.depth.test.is_bypassed());
        layer
            .shading_flags
            .set_no_depth_set(source.state.rasterization.depth.write.is_bypassed());
        layer.texture_id = encoder.value(&source.binding.image, map, &format!("{path}.texture"))?;
        layer.texture_animation_id = source.binding.coordinates.transform.map_or(u32::MAX, |id| {
            encoder
                .model
                .texture_transforms()
                .position(|(key, _)| key == id)
                .expect("validated transform") as u32
        });
        layer.coordinate_id = source.binding.coordinates.source;
        layer.alpha = encoder.value(&source.opacity, Ok, &format!("{path}.opacity"))?;
        if let Some(source) = &source.emissive_gain {
            let value = encoder.value(source, Ok, &format!("{path}.emissive_gain"))?;
            if let Err(error) = layer.try_set_emissive_gain(value) {
                encoder
                    .context
                    .unsupported(format!("{path}.emissive_gain"), error.to_string())?;
            }
        }
        if let Some(source) = &source.fresnel {
            let value = native::materials::LayerFresnel {
                color: encoder.value(&source.color, Ok, &format!("{path}.fresnel.color"))?,
                opacity: encoder.value(&source.opacity, Ok, &format!("{path}.fresnel.opacity"))?,
                team_color: encoder.value(
                    &source.team_color,
                    Ok,
                    &format!("{path}.fresnel.team_color"),
                )?,
            };
            if let Err(error) = layer.try_set_fresnel(value) {
                encoder
                    .context
                    .unsupported(format!("{path}.fresnel"), error.to_string())?;
            }
        }
        if let Some(source) = source.shader {
            if let Err(error) =
                layer.try_set_shader_type(native::materials::ShaderType::new(source))
            {
                encoder
                    .context
                    .unsupported(format!("{path}.shader"), error.to_string())?;
            }
        }
        let slots = source
            .textures
            .iter()
            .enumerate()
            .map(|(index, source)| {
                Ok(native::materials::LayerTextureSlot {
                    texture_type: source.slot,
                    texture_id: encoder.value(
                        &source.binding.image,
                        map,
                        &format!("{path}.textures[{index}]"),
                    )?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        if !slots.is_empty() {
            if let Err(error) = layer.try_set_texture_slots(&slots) {
                encoder
                    .context
                    .unsupported(format!("{path}.textures"), error.to_string())?;
            }
        }
        material.layers.push(layer);
    }
    Ok(material)
}
fn blend(source: profile::Blend) -> native::LayerFilterMode {
    match source {
        profile::Blend::Opaque => native::LayerFilterMode::None,
        profile::Blend::AlphaKey(()) => native::LayerFilterMode::Transparent,
        profile::Blend::Alpha => native::LayerFilterMode::Blend,
        profile::Blend::Additive => native::LayerFilterMode::Additive,
        profile::Blend::AdditiveAlpha(()) => native::LayerFilterMode::AddAlpha,
        profile::Blend::Modulate => native::LayerFilterMode::Modulate,
        profile::Blend::Modulate2x => native::LayerFilterMode::Modulate2x,
        profile::Blend::Unknown(value) => native::LayerFilterMode::Unknown(value),
        profile::Blend::Extension(value) => match value {},
    }
}
