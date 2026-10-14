//! World of Warcraft shader parameters retain their external selection domains.
use crate::model::semantic::profiles::wem::{
    WowBatchFlags, WowBlend, WowColorSelection, WowMaterial, WowRenderFlags, WowTextureSelection,
    WowTextureSource, WowTextureUnit, WowTransformSelection, WowWeightSelection,
};
use crate::model::wem::{records, InlineList};
pub(in crate::model::conversion::wem) fn read_wow(value: &records::M2Material) -> WowMaterial {
    WowMaterial {
        flags: WowRenderFlags::from_bits_retain(value.render_flags.bits()),
        blend: WowBlend::from_raw(value.blending_mode.raw()),
        shader: value.shader_id,
        priority_plane: value.priority_plane,
        layer: value.material_layer,
        color: WowColorSelection(value.color_index),
        batch_flags: WowBatchFlags::from_bits_retain(value.batch_flags.bits()),
        textures: value
            .units
            .0
            .iter()
            .map(|unit| WowTextureUnit {
                texture: WowTextureSelection(unit.texture),
                coordinate_set: unit.uv_set,
                transform: WowTransformSelection(unit.transform),
                weight: WowWeightSelection(unit.weight),
                source: WowTextureSource::from_raw(unit.texture_type.raw()),
                replaceable: unit.replaceable_id,
            })
            .collect(),
    }
}
pub(in crate::model::conversion::wem) fn write_wow(
    value: &WowMaterial,
    source_version: u32,
) -> records::M2Material {
    records::M2Material {
        source_version,
        render_flags: records::M2RenderFlags16::from_bits_retain(value.flags.bits()),
        blending_mode: records::M2BlendingMode16::from_raw(value.blend.raw()),
        shader_id: value.shader,
        priority_plane: value.priority_plane,
        material_layer: value.layer,
        color_index: value.color.0,
        batch_flags: records::M2BatchFlags8::from_bits_retain(value.batch_flags.bits()),
        units: InlineList(
            value
                .textures
                .iter()
                .map(|unit| records::M2TextureUnit {
                    texture: unit.texture.0,
                    uv_set: unit.coordinate_set,
                    transform: unit.transform.0,
                    weight: unit.weight.0,
                    texture_type: records::M2TextureSource8::from_raw(unit.source.raw()),
                    replaceable_id: unit.replaceable,
                })
                .collect::<Vec<_>>(),
        ),
    }
}
