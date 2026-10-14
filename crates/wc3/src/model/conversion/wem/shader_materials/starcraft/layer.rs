//! StarCraft shader inputs resolve text independently of source index sections.
use crate::model::conversion::wem::shader_materials::starcraft::{read_anim, write_anim};
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::materials::starcraft::TextureLayer;
use crate::model::semantic::TextureId;
use crate::model::wem::records::M3TextureLayerFlag;
use crate::model::wem::records::{
    M3ColorBGRA, M3ColorChannelSelect, M3FresnelMode, M3TextureLayer, M3UVMappingMode,
};
use crate::model::wem::Ref;
use std::borrow::Cow;

use crate::model::conversion::wem::text;

use crate::model::semantic::profiles::m3::values::{
    ColorChannel, FresnelMode, LayerFlags, UvMappingMode,
};

use crate::model::wem::{records, RawDocument};
pub(super) fn read_texture_layer<'view>(
    document: &'view RawDocument<'_>,
    value: &M3TextureLayer,

    texture: &mut impl FnMut(Cow<'view, [u8]>) -> Result<TextureId, Error>,
) -> Result<TextureLayer, Error> {
    Ok(TextureLayer {
        id: value.id,
        texture: {
            let path = text::read_bytes(document, value.texture_path)?;
            if path.is_empty() {
                None
            } else {
                Some(texture(path)?)
            }
        },
        color: read_anim(&value.color, |value| [value.r, value.g, value.b, value.a]),
        flags: LayerFlags::from(value.flags.bits()),
        uv_mapping: UvMappingMode::from_raw(value.uv_mapping.raw()),
        color_type: ColorChannel::from_raw(value.color_type.raw()),
        rgb_multiply: read_anim(&value.rgb_multiply, |value| *value),
        rgb_add: read_anim(&value.rgb_add, |value| *value),
        poc_texture: value.poc_texture,
        noise_amplitude: value.noise_amplitude,
        noise_frequency: value.noise_frequency,
        texture_source: value.texture_source,
        avi_frame_rate: value.avi_frame_rate,
        avi_start: value.avi_start,
        avi_stop: value.avi_stop,
        avi_loop: value.avi_loop,
        avi_sync: value.avi_sync,
        avi_play: read_anim(&value.avi_play, |value| *value),
        avi_restart: read_anim(&value.avi_restart, |value| *value),
        flipbook_rows: value.flipbook_rows,
        flipbook_columns: value.flipbook_columns,
        current_frame: read_anim(&value.current_frame, |value| *value),
        uv_offset: read_anim(&value.uv_offset, |value| *value),
        uv_angle: read_anim(&value.uv_angle, |value| *value),
        uv_tiling: read_anim(&value.uv_tiling, |value| *value),
        w_offset: read_anim(&value.w_offset, |value| *value),
        w_tiling: read_anim(&value.w_tiling, |value| *value),
        map_alpha: read_anim(&value.map_alpha, |value| *value),
        triplanar_offset: read_anim(&value.triplanar_offset, |value| *value),
        triplanar_scale: read_anim(&value.triplanar_scale, |value| *value),
        uv_source_related: value.uv_source_related,
        fresnel_mode: FresnelMode::from_raw(value.fresnel_mode.raw()),
        fresnel_exponent: value.fresnel_exponent,
        fresnel_min: value.fresnel_min,
        fresnel_max: value.fresnel_max,
        fresnel_translation: value.fresnel_translation,
        fresnel_mask: value.fresnel_mask,
        fresnel_rotation: value.fresnel_rotation,
        uv_density: value.uv_density,
    })
}
pub(super) fn write_texture_layer(
    _document: &mut RawDocument<'_>,
    value: &TextureLayer,

    texture: &impl Fn(TextureId) -> Result<Ref<records::targets::Character>, Error>,
) -> Result<M3TextureLayer, Error> {
    Ok(M3TextureLayer {
        id: value.id,
        texture_path: value.texture.map(texture).transpose()?.unwrap_or_default(),
        color: write_anim(&value.color, |value| M3ColorBGRA {
            r: value[0],
            g: value[1],
            b: value[2],
            a: value[3],
        }),
        flags: M3TextureLayerFlag::from_bits_retain(value.flags.bits()),
        uv_mapping: M3UVMappingMode::from_raw(value.uv_mapping.raw()),
        color_type: M3ColorChannelSelect::from_raw(value.color_type.raw()),
        rgb_multiply: write_anim(&value.rgb_multiply, |value| *value),
        rgb_add: write_anim(&value.rgb_add, |value| *value),
        poc_texture: value.poc_texture,
        noise_amplitude: value.noise_amplitude,
        noise_frequency: value.noise_frequency,
        texture_source: value.texture_source,
        avi_frame_rate: value.avi_frame_rate,
        avi_start: value.avi_start,
        avi_stop: value.avi_stop,
        avi_loop: value.avi_loop,
        avi_sync: value.avi_sync,
        avi_play: write_anim(&value.avi_play, |value| *value),
        avi_restart: write_anim(&value.avi_restart, |value| *value),
        flipbook_rows: value.flipbook_rows,
        flipbook_columns: value.flipbook_columns,
        current_frame: write_anim(&value.current_frame, |value| *value),
        uv_offset: write_anim(&value.uv_offset, |value| *value),
        uv_angle: write_anim(&value.uv_angle, |value| *value),
        uv_tiling: write_anim(&value.uv_tiling, |value| *value),
        w_offset: write_anim(&value.w_offset, |value| *value),
        w_tiling: write_anim(&value.w_tiling, |value| *value),
        map_alpha: write_anim(&value.map_alpha, |value| *value),
        triplanar_offset: write_anim(&value.triplanar_offset, |value| *value),
        triplanar_scale: write_anim(&value.triplanar_scale, |value| *value),
        uv_source_related: value.uv_source_related,
        fresnel_mode: M3FresnelMode::from_raw(value.fresnel_mode.raw()),
        fresnel_exponent: value.fresnel_exponent,
        fresnel_min: value.fresnel_min,
        fresnel_max: value.fresnel_max,
        fresnel_translation: value.fresnel_translation,
        fresnel_mask: value.fresnel_mask,
        fresnel_rotation: value.fresnel_rotation,
        uv_density: value.uv_density,
    })
}
