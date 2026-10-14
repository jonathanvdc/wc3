//! Warcraft shader contributions use shared images and external transform selections.
use super::super::text;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{
    WarcraftFilter, WarcraftInput, WarcraftLayer, WarcraftMaterial, WarcraftMaterialFlags,
    WarcraftShader, WarcraftShadingFlags, WarcraftTextureSlot, WarcraftTextures,
    WarcraftTransformSelection,
};
use crate::model::semantic::TextureId;
use crate::model::wem::{records, InlineList, RawDocument};
use std::collections::BTreeMap;
fn texture(index: u32, textures: &[TextureId]) -> Result<Option<TextureId>, Error> {
    if index == u32::MAX {
        Ok(None)
    } else {
        textures
            .get(index as usize)
            .copied()
            .map(Some)
            .ok_or_else(|| {
                Error::new(
                    "wem.material.warcraft.texture",
                    "image is absent from the shared declaration arena",
                )
            })
    }
}
fn index(value: Option<TextureId>, textures: &BTreeMap<TextureId, u32>) -> Result<u32, Error> {
    match value {
        None => Ok(u32::MAX),
        Some(id) => textures.get(&id).copied().ok_or_else(|| {
            Error::new(
                "wem.material.warcraft.texture",
                "image belongs to another declaration arena",
            )
        }),
    }
}
pub(in crate::model::conversion::wem) fn read_warcraft<'a>(
    document: &'a RawDocument<'_>,
    value: &records::MdxMaterial,
    textures: &[TextureId],
) -> Result<WarcraftMaterial<'a>, Error> {
    let mut layers = Vec::new();
    for layer in &value.layers.0 {
        let textures = if layer.sub_textures.0.is_empty() {
            WarcraftTextures::Single(texture(layer.texture_id, textures)?)
        } else {
            WarcraftTextures::Slots(
                layer
                    .sub_textures
                    .0
                    .iter()
                    .map(|input| {
                        Ok(WarcraftInput {
                            texture: texture(input.texture_id, textures)?,
                            slot: WarcraftTextureSlot::from_raw(input.slot.raw()),
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?,
            )
        };
        let mut result = WarcraftLayer::new(textures);
        result.blend = WarcraftFilter::from_raw(layer.filter_mode.raw());
        result.shading = WarcraftShadingFlags::from_bits_retain(layer.shading_flags.bits());
        result.transform = WarcraftTransformSelection(layer.texture_animation_id);
        result.coordinate_set = layer.coord_id;
        result.opacity = layer.alpha;
        result.emissive = layer.emissive_gain;
        result.fresnel_color = layer.fresnel_color;
        result.fresnel_opacity = layer.fresnel_opacity;
        result.fresnel_team_color = layer.fresnel_team_color;
        result.shader = WarcraftShader::from_raw(layer.shader_type.raw());
        result.high_definition = layer.is_hd.get();
        layers.push(result);
    }
    Ok(WarcraftMaterial {
        priority_plane: value.priority_plane,
        flags: WarcraftMaterialFlags::from_bits_retain(value.flags.bits()),
        shader: text::read(document, value.shader)?,
        layers,
    })
}
pub(in crate::model::conversion::wem) fn write_warcraft(
    document: &mut RawDocument<'_>,
    value: &WarcraftMaterial<'_>,
    source_version: u32,
    textures: &BTreeMap<TextureId, u32>,
) -> Result<records::MdxMaterial, Error> {
    value.validate()?;
    let mut layers = Vec::new();
    for layer in &value.layers {
        let (texture_id, sub_textures) = match &layer.textures {
            WarcraftTextures::Single(texture) => (index(*texture, textures)?, Vec::new()),
            WarcraftTextures::Slots(inputs) => (
                0,
                inputs
                    .iter()
                    .map(|input| {
                        Ok(records::MdxSubTexture {
                            texture_id: index(input.texture, textures)?,
                            slot: records::MdxSlotType::from_raw(input.slot.raw()),
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?,
            ),
        };
        layers.push(records::MdxLayer {
            filter_mode: records::MdxFilterMode::from_raw(layer.blend.raw()),
            shading_flags: records::MdxShadingFlag::from_bits_retain(layer.shading.bits()),
            texture_id,
            texture_animation_id: layer.transform.0,
            coord_id: layer.coordinate_set,
            alpha: layer.opacity,
            emissive_gain: layer.emissive,
            fresnel_color: layer.fresnel_color,
            fresnel_opacity: layer.fresnel_opacity,
            fresnel_team_color: layer.fresnel_team_color,
            shader_type: records::MdxShaderType::from_raw(layer.shader.raw()),
            is_hd: layer.high_definition.into(),
            sub_textures: InlineList(sub_textures),
        });
    }
    Ok(records::MdxMaterial {
        source_version,
        priority_plane: value.priority_plane,
        flags: records::MdxMaterialFlag::from_bits_retain(value.flags.bits()),
        shader: text::write_bytes(document, value.shader.as_bytes())?,
        layers: InlineList(layers),
    })
}
