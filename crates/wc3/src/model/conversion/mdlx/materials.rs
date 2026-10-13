//! Typed native Warcraft III material mappings.
use super::*;
pub(super) fn filter(value: native::LayerFilterMode) -> u32 {
    match value {
        native::LayerFilterMode::None => 0,
        native::LayerFilterMode::Transparent => 1,
        native::LayerFilterMode::Blend => 2,
        native::LayerFilterMode::Additive => 3,
        native::LayerFilterMode::AddAlpha => 4,
        native::LayerFilterMode::Modulate => 5,
        native::LayerFilterMode::Modulate2x => 6,
        native::LayerFilterMode::Unknown(value) => value,
    }
}

pub(super) fn native_layers<V: ModelVersion>(
    material: &native::Material<'_, V>,
) -> Vec<wr::MdxLayer> {
    material
        .layers
        .iter()
        .map(|layer| wr::MdxLayer {
            filter_mode: wr::MdxFilterMode(filter(layer.filter_mode)),
            shading_flags: wr::MdxShadingFlag(layer.shading_flags.bits()),
            texture_id: base(&layer.texture_id),
            texture_animation_id: layer.texture_animation_id,
            coord_id: layer.coordinate_id,
            alpha: base(&layer.alpha),
            emissive_gain: layer.try_emissive_gain().map(base).unwrap_or(0.),
            fresnel_color: layer
                .try_fresnel()
                .map(|f| base(&f.color))
                .unwrap_or([1.; 3]),
            fresnel_opacity: layer.try_fresnel().map(|f| base(&f.opacity)).unwrap_or(0.),
            fresnel_team_color: layer
                .try_fresnel()
                .map(|f| base(&f.team_color))
                .unwrap_or(0.),
            shader_type: wr::MdxShaderType(
                layer
                    .try_shader_type()
                    .map(|shader| shader.id())
                    .unwrap_or(0),
            ),
            is_hd: wem::Bool::from(
                material
                    .try_shader()
                    .map(|shader| shader.starts_with("Shader_HD"))
                    .unwrap_or(false),
            ),
            sub_textures: wem::InlineList(
                layer
                    .try_texture_slots()
                    .map(|slots| {
                        slots
                            .iter()
                            .map(|slot| wr::MdxSubTexture {
                                texture_id: base(&slot.texture_id),
                                slot: wr::MdxSlotType(slot.texture_type),
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
            ),
        })
        .collect()
}

pub(super) fn overlay_material<V: ModelVersion>(
    material: &mut native::Material<'static, V>,
    record: &wr::MdxMaterial,
    raw: &RawDocument<'_>,
    context: &mut Context,
) -> Result<(), Error> {
    material.priority_plane = record.priority_plane;
    material.render_mode = native::MaterialRenderFlags(record.flags.0);
    let shader = raw.string(record.shader)?.unwrap_or_default();
    if let Err(error) = material.try_set_shader(shader) {
        if !shader.is_empty() {
            context.unsupported("material.shader", error.to_string())?;
        }
    }
    material
        .layers
        .resize_with(record.layers.0.len(), native::Layer::new);
    for (layer, data) in material.layers.iter_mut().zip(&record.layers.0) {
        layer.filter_mode = match data.filter_mode.0 {
            0 => native::LayerFilterMode::None,
            1 => native::LayerFilterMode::Transparent,
            2 => native::LayerFilterMode::Blend,
            3 => native::LayerFilterMode::Additive,
            4 => native::LayerFilterMode::AddAlpha,
            5 => native::LayerFilterMode::Modulate,
            6 => native::LayerFilterMode::Modulate2x,
            value => native::LayerFilterMode::Unknown(value),
        };
        layer.shading_flags = native::LayerShadingFlags(data.shading_flags.0);
        layer.texture_id.set_value(data.texture_id);
        layer.texture_animation_id = data.texture_animation_id;
        layer.coordinate_id = data.coord_id;
        layer.alpha.set_value(data.alpha);
        let mut emissive = layer.try_emissive_gain().cloned().unwrap_or_default();
        emissive.set_value(data.emissive_gain);
        if let Err(error) = layer.try_set_emissive_gain(emissive) {
            if data.emissive_gain != 0. {
                context.unsupported("material.emissive", error.to_string())?;
            }
        }
        let mut fresnel = layer.try_fresnel().cloned().unwrap_or_default();
        fresnel.color.set_value(data.fresnel_color);
        fresnel.opacity.set_value(data.fresnel_opacity);
        fresnel.team_color.set_value(data.fresnel_team_color);
        if let Err(error) = layer.try_set_fresnel(fresnel) {
            if data.fresnel_color != [1.; 3]
                || data.fresnel_opacity != 0.
                || data.fresnel_team_color != 0.
            {
                context.unsupported("material.fresnel", error.to_string())?;
            }
        }
        if let Err(error) =
            layer.try_set_shader_type(native::materials::ShaderType::new(data.shader_type.0))
        {
            if data.shader_type.0 != 0 {
                context.unsupported("material.shader_type", error.to_string())?;
            }
        }
        let old = layer.try_texture_slots().unwrap_or_default();
        let slots: Vec<_> = data
            .sub_textures
            .0
            .iter()
            .enumerate()
            .map(|(index, texture)| {
                let mut id = old
                    .get(index)
                    .map(|slot| slot.texture_id.clone())
                    .unwrap_or_default();
                id.set_value(texture.texture_id);
                native::materials::LayerTextureSlot {
                    texture_id: id,
                    texture_type: texture.slot.0,
                }
            })
            .collect();
        if let Err(error) = layer.try_set_texture_slots(&slots) {
            if !slots.is_empty() {
                context.unsupported("material.texture_slots", error.to_string())?;
            }
        }
    }
    Ok(())
}
