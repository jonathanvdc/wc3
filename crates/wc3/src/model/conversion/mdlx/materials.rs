//! Typed native Warcraft III material mappings.
use super::*;
pub(super) fn filter(value: native::LayerFilterMode) -> wr::BlendMode {
    match value {
        native::LayerFilterMode::None => wr::BlendMode::Opaque,
        native::LayerFilterMode::Transparent => wr::BlendMode::AlphaKey,
        native::LayerFilterMode::Blend => wr::BlendMode::AlphaBlend,
        native::LayerFilterMode::Additive => wr::BlendMode::Additive,
        native::LayerFilterMode::AddAlpha => wr::BlendMode::AdditiveAlpha,
        native::LayerFilterMode::Modulate => wr::BlendMode::Modulate,
        native::LayerFilterMode::Modulate2x => wr::BlendMode::Modulate2x,
        native::LayerFilterMode::Unknown(value) => match wr::BlendMode::from_raw(value) {
            wr::BlendMode::Unknown(value) => wr::BlendMode::Unknown(value),
            _ => wr::BlendMode::Opaque,
        },
    }
}
pub(super) fn common_material_flags<V: ModelVersion>(
    material: &native::Material<'_, V>,
) -> wr::MaterialFlags {
    let mut flags = wr::MaterialFlags::default();
    flags.set_sort_near_z(material.render_mode.sort_primitives_near_z());
    flags.set_sort_far_z(material.render_mode.sort_primitives_far_z());
    if let Some(layer) = material.layers.first() {
        flags.set_unlit(layer.shading_flags.unshaded());
        flags.set_unfogged(layer.shading_flags.unfogged());
    }
    flags
}

pub(super) fn native_layers<V: ModelVersion>(
    material: &native::Material<'_, V>,
) -> Vec<wr::MdxLayer> {
    material
        .layers
        .iter()
        .map(|layer| wr::MdxLayer {
            filter_mode: wr::MdxFilterMode::from_raw(match layer.filter_mode {
                native::LayerFilterMode::Unknown(value) => value,
                value => filter(value).raw(),
            }),
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
            shader_type: wr::MdxShaderType::from_raw(
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
                                slot: wr::MdxSlotType::from_raw(slot.texture_type),
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
    material.render_mode = native::MaterialRenderFlags(record.flags.bits());
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
        layer.filter_mode = match data.filter_mode {
            wr::MdxFilterMode::None => native::LayerFilterMode::None,
            wr::MdxFilterMode::Transparent => native::LayerFilterMode::Transparent,
            wr::MdxFilterMode::Blend => native::LayerFilterMode::Blend,
            wr::MdxFilterMode::Additive => native::LayerFilterMode::Additive,
            wr::MdxFilterMode::AddAlpha => native::LayerFilterMode::AddAlpha,
            wr::MdxFilterMode::Modulate => native::LayerFilterMode::Modulate,
            wr::MdxFilterMode::Modulate2x => native::LayerFilterMode::Modulate2x,
            value => native::LayerFilterMode::Unknown(value.raw()),
        };
        layer.shading_flags = native::LayerShadingFlags(data.shading_flags.bits());
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
            layer.try_set_shader_type(native::materials::ShaderType::new(data.shader_type.raw()))
        {
            if data.shader_type.raw() != 0 {
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
                    texture_type: texture.slot.raw(),
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
