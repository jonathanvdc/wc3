//! Typed material programs checked and written to a Warcraft III dialect.
use super::encode_animation::Encoder;
use super::encode_helpers::texture_transform;
use crate::model::conversion::{Error, IssueKind};
use crate::model::formats::mdlx::{self as native, FormatVersion};
use crate::model::materials::mdlx::{AlphaAdditiveEncoding, Shader};
use crate::model::profiles::mdlx as profile;
use crate::model::Texture;
use std::collections::BTreeMap;
use std::mem::replace;

pub(super) fn write<'a, V: FormatVersion>(
    source: &profile::Material<'_>,
    textures: &BTreeMap<Texture, u32>,
    encoder: &mut Encoder<'_, '_, '_>,
    transforms: &mut Vec<native::TextureAnimation<'a>>,
    path: &str,
) -> Result<native::Material<'a, V>, Error> {
    let map = |id: Option<Texture>| {
        id.map(|id| {
            textures
                .get(&id)
                .copied()
                .ok_or_else(|| Error::new(path, "texture identity does not exist"))
        })
        .transpose()
        .map(|id| id.unwrap_or(u32::MAX))
    };
    let declaration = &source.declaration;
    let mut material = native::Material::<V>::new();
    material.priority_plane = declaration.priority_plane;
    material.render_mode = native::MaterialRenderFlags(0);
    material
        .render_mode
        .set_constant_color(declaration.constant_color);
    material.render_mode.set_two_sided(declaration.two_sided);
    material
        .render_mode
        .set_sort_primitives_near_z(declaration.sort_near_z);
    material
        .render_mode
        .set_sort_primitives_far_z(declaration.sort_far_z);
    material
        .render_mode
        .set_full_resolution(declaration.full_resolution);
    // Older layouts select a single shader by name instead of per-layer IDs.
    let named_default_unit = matches!(V::NUMBER, 900 | 1000)
        && (declaration.shader_name.is_empty()
            || declaration.shader_name == "Shader_HD_DefaultUnit")
        && !source.steps.is_empty()
        && source
            .steps
            .iter()
            .all(|step| matches!(step.shader, Shader::ReforgedDefault(_)));
    if named_default_unit && source.steps.len() != 1 {
        return Err(Error::new(
            path,
            "legacy DefaultUnit supports one surface, not multiple draw passes",
        ));
    }
    let shader_name = if named_default_unit {
        "Shader_HD_DefaultUnit"
    } else {
        declaration.shader_name.as_ref()
    };
    if let Err(error) = material.try_set_shader(shader_name) {
        if !shader_name.is_empty() {
            let known_selection = !source.steps.is_empty()
                && match shader_name {
                    "Shader_HD_DefaultUnit" => source.steps.iter().all(|step| matches!(step.shader, Shader::ReforgedDefault(_))),
                    "Shader_SD_FixedFunction" => source.steps.iter().all(|step| matches!(step.shader, Shader::Classic(_))),
                    _ => false,
                };
            if known_selection {
                encoder.context.issue(
                    format!("{path}.shader"),
                    IssueKind::Normalized,
                    "encoded the known material shader selection through the destination's per-layer selectors",
                );
            } else {
                encoder.context.unsupported(format!("{path}.shader"), error.to_string())?;
            }
        }
    }
    for (index, source) in source.steps.iter().enumerate() {
        let path = format!("{path}.layers[{index}]");
        let inputs = source.shader.inputs();
        let selector = match &source.shader {
            Shader::Classic(_)
                if !shader_name.is_empty() && shader_name != "Shader_SD_FixedFunction" =>
            {
                Some(0)
            }
            Shader::Classic(_) => None,
            Shader::ReforgedDefault(_) => Some(1),
            Shader::Unresolved(inputs) => inputs.selector,
        };
        let mut layer = native::Layer::<V>::new();
        layer.filter_mode = if let Some(mode) = blend(source.state.blend) {
            mode
        } else {
            encoder.context.unsupported(
                format!("{path}.layers[{index}].blend"),
                "unweighted source-color addition has no exact Warcraft III layer selector",
            )?;
            native::LayerFilterMode::None
        };
        layer.shading_flags = native::LayerShadingFlags(0);
        layer
            .shading_flags
            .set_unshaded(source.state.rasterization.lighting.unlit);
        layer
            .shading_flags
            .set_sphere_env_map(source.state.rasterization.controls.sphere_environment);
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
        let controls = source.state.rasterization.controls;
        layer.shading_flags.set_wrap_width(controls.wrap_u);
        layer.shading_flags.set_wrap_height(controls.wrap_v);
        layer.shading_flags.set_unlit(controls.unlit);
        layer
            .shading_flags
            .set_back_faces_for_shadows(controls.back_faces_for_shadows);
        layer
            .shading_flags
            .set_ambient_occlusion(controls.ambient_occlusion);
        layer.texture_id = encoder.value(&inputs.binding.image, map, &format!("{path}.texture"))?;
        let transform = texture_transform(&inputs.binding.coordinates.transform, encoder)?;
        layer.texture_animation_id = if transform.translation.is_none()
            && transform.rotation.is_none()
            && transform.scaling.is_none()
        {
            u32::MAX
        } else {
            let index = transforms
                .iter()
                .position(|value| value == &transform)
                .unwrap_or_else(|| {
                    transforms.push(transform);
                    transforms.len() - 1
                });
            index as u32
        };

        layer.coordinate_id = inputs.binding.coordinates.source;
        layer.alpha = encoder.value(&inputs.opacity, Ok, &format!("{path}.opacity"))?;
        if let Some(source) = &inputs.emissive_gain {
            let value = encoder.value(source, Ok, &format!("{path}.emissive_gain"))?;
            if let Err(error) = layer.try_set_emissive_gain(value) {
                encoder
                    .context
                    .unsupported(format!("{path}.emissive_gain"), error.to_string())?;
            }
        }
        if let Some(source) = source.shader.fresnel() {
            let value = native::materials::LayerFresnel {
                color: encoder.value(
                    &source.color,
                    |v| Ok(v.to_array()),
                    &format!("{path}.fresnel.color"),
                )?,
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
        if let Some(source) = selector.filter(|_| !named_default_unit) {
            if let Err(error) =
                layer.try_set_shader_type(native::materials::ShaderType::new(source))
            {
                encoder
                    .context
                    .unsupported(format!("{path}.shader"), error.to_string())?;
            }
        }
        let mut slots = source
            .shader
            .textures()
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
        if matches!(source.shader, Shader::Classic(_)) && V::NUMBER >= 1100 {
            slots.push(native::materials::LayerTextureSlot {
                texture_type: 0,
                texture_id: replace(&mut layer.texture_id, native::Animatable::Static(0)),
            });
        }
        if named_default_unit {
            let mut roles = BTreeMap::new();
            for slot in slots {
                if slot.texture_type > 5
                    || roles.insert(slot.texture_type, slot.texture_id).is_some()
                {
                    return Err(Error::new(
                        &path,
                        "legacy DefaultUnit requires unique texture roles zero through five",
                    ));
                }
            }
            if let Some(diffuse) = roles.remove(&0) {
                layer.texture_id = diffuse;
            }
            let last = roles.keys().next_back().copied().unwrap_or(0);
            material.layers.push(layer);
            for role in 1..=last {
                let mut auxiliary = native::Layer::<V>::new();
                auxiliary.texture_id = roles
                    .remove(&role)
                    .unwrap_or(native::Animatable::Static(u32::MAX));
                material.layers.push(auxiliary);
            }
            continue;
        }
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
fn blend(source: profile::Blend) -> Option<native::LayerFilterMode> {
    Some(match source {
        profile::Blend::Opaque => native::LayerFilterMode::None,
        profile::Blend::AlphaKey(()) => native::LayerFilterMode::Transparent,
        profile::Blend::Alpha => native::LayerFilterMode::Blend,
        profile::Blend::Additive => return None,
        profile::Blend::AdditiveAlpha(AlphaAdditiveEncoding::Additive) => {
            native::LayerFilterMode::Additive
        }
        profile::Blend::AdditiveAlpha(AlphaAdditiveEncoding::AddAlpha) => {
            native::LayerFilterMode::AddAlpha
        }
        profile::Blend::Modulate => native::LayerFilterMode::Modulate,
        profile::Blend::Modulate2x => native::LayerFilterMode::Modulate2x,
        profile::Blend::Unknown(value) => native::LayerFilterMode::Unknown(value),
        profile::Blend::Extension(value) => match value {},
    })
}
