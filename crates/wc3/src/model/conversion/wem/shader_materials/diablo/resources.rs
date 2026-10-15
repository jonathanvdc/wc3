//! Diablo shader inputs retain external dependencies and independent native state.
use crate::model::conversion::wem::shader_materials::diablo::{read_group, write_group};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::diablo::d3::{
    AssetRef, MaterialColors, MaterialFlags, ParameterType, TagValue, TexAnim, TextureEntry,
    TextureType, UberMaterial, UvFlags, UvTransformMode,
};
use crate::model::wem::records::{
    D3AssetRef, D3MaterialColors, D3MaterialFlags, D3TagValue, D3TexAnim, D3TextureEntry,
    D3UberMaterial, D3UvFlags,
};

use crate::model::wem::{InlineList, RawDocument};
pub(super) fn read_asset_ref(
    _document: &RawDocument<'_>,
    value: &D3AssetRef,
) -> Result<AssetRef, Error> {
    Ok(AssetRef {
        id: value.id,
        group: read_group(value.group),
    })
}
pub(super) fn write_asset_ref(
    _document: &mut RawDocument<'_>,
    value: &AssetRef,
) -> Result<D3AssetRef, Error> {
    Ok(D3AssetRef {
        id: value.id,
        group: write_group(value.group),
    })
}
pub(super) fn read_material_colors(
    _document: &RawDocument<'_>,
    value: &D3MaterialColors,
) -> Result<MaterialColors, Error> {
    Ok(MaterialColors {
        diffuse: value.diffuse,
        specular: value.specular,
        emissive: value.emissive,
        ambient: value.ambient,
        shininess: value.shininess,
        material_flags: MaterialFlags::from_bits_retain(value.material_flags.bits()),
    })
}
pub(super) fn write_material_colors(
    _document: &mut RawDocument<'_>,
    value: &MaterialColors,
) -> Result<D3MaterialColors, Error> {
    Ok(D3MaterialColors {
        diffuse: value.diffuse,
        specular: value.specular,
        emissive: value.emissive,
        ambient: value.ambient,
        shininess: value.shininess,
        material_flags: D3MaterialFlags::from(value.material_flags.bits()),
    })
}
pub(super) fn read_tex_anim(
    _document: &RawDocument<'_>,
    value: &D3TexAnim,
) -> Result<TexAnim, Error> {
    Ok(TexAnim {
        amount: value.amount,
        rate0: value.rate0,
        rate1: value.rate1,
    })
}
pub(super) fn write_tex_anim(
    _document: &mut RawDocument<'_>,
    value: &TexAnim,
) -> Result<D3TexAnim, Error> {
    Ok(D3TexAnim {
        amount: value.amount,
        rate0: value.rate0,
        rate1: value.rate1,
    })
}
pub(super) fn read_texture_entry(
    document: &RawDocument<'_>,
    value: &D3TextureEntry,
) -> Result<TextureEntry, Error> {
    Ok(TextureEntry {
        texture_type: TextureType(value.type_),
        unclassified_control: value.dw_unknown04,
        texture: read_asset_ref(document, &value.texture)?,
        uv_transform_mode: UvTransformMode(value.uv_transform_mode),
        uv_row0: value.uv_row0,
        uv_row1: value.uv_row1,
        uv_row2: value.uv_row2,
        uv_row3: value.uv_row3,
        anim_u: read_tex_anim(document, &value.anim_u)?,
        anim_v: read_tex_anim(document, &value.anim_v)?,
        anim_rotate: read_tex_anim(document, &value.anim_rotate)?,
        anim3: read_tex_anim(document, &value.anim3)?,
        anim4: read_tex_anim(document, &value.anim4)?,
        anim5: read_tex_anim(document, &value.anim5)?,
        uv_flags: UvFlags::from_bits_retain(value.uv_flags.bits()),
        unclassified_control2: value.dw_unknown9_c,
    })
}
pub(super) fn write_texture_entry(
    document: &mut RawDocument<'_>,
    value: &TextureEntry,
) -> Result<D3TextureEntry, Error> {
    Ok(D3TextureEntry {
        type_: value.texture_type.0,
        dw_unknown04: value.unclassified_control,
        texture: write_asset_ref(document, &value.texture)?,
        uv_transform_mode: value.uv_transform_mode.0,
        uv_row0: value.uv_row0,
        uv_row1: value.uv_row1,
        uv_row2: value.uv_row2,
        uv_row3: value.uv_row3,
        anim_u: write_tex_anim(document, &value.anim_u)?,
        anim_v: write_tex_anim(document, &value.anim_v)?,
        anim_rotate: write_tex_anim(document, &value.anim_rotate)?,
        anim3: write_tex_anim(document, &value.anim3)?,
        anim4: write_tex_anim(document, &value.anim4)?,
        anim5: write_tex_anim(document, &value.anim5)?,
        uv_flags: D3UvFlags::from(value.uv_flags.bits()),
        dw_unknown9_c: value.unclassified_control2,
    })
}
pub(super) fn read_uber_material(
    document: &RawDocument<'_>,
    value: &D3UberMaterial,
) -> Result<UberMaterial, Error> {
    Ok(UberMaterial {
        shader_map: read_asset_ref(document, &value.shader_map)?,
        colors: read_material_colors(document, &value.colors)?,
        textures: value
            .textures
            .0
            .iter()
            .map(|value| read_texture_entry(document, value))
            .collect::<Result<Vec<_>, Error>>()?,
    })
}
pub(super) fn write_uber_material(
    document: &mut RawDocument<'_>,
    value: &UberMaterial,
) -> Result<D3UberMaterial, Error> {
    Ok(D3UberMaterial {
        shader_map: write_asset_ref(document, &value.shader_map)?,
        colors: write_material_colors(document, &value.colors)?,
        textures: InlineList(
            value
                .textures
                .iter()
                .map(|value| write_texture_entry(document, value))
                .collect::<Result<Vec<_>, Error>>()?,
        ),
    })
}
pub(super) fn read_tag_value(
    _document: &RawDocument<'_>,
    value: &D3TagValue,
) -> Result<TagValue, Error> {
    Ok(TagValue {
        value_type: ParameterType(value.value_type),
        tag_id: value.tag_id,
        value: value.value,
    })
}
pub(super) fn write_tag_value(
    _document: &mut RawDocument<'_>,
    value: &TagValue,
) -> Result<D3TagValue, Error> {
    Ok(D3TagValue {
        value_type: value.value_type.0,
        tag_id: value.tag_id,
        value: value.value,
    })
}
