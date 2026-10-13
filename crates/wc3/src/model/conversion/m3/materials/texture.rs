//! Texture mappings for native M3 materials.
use super::{
    anim, append, append_string, color, native, native_anim, native_color, nr, read_error,
    revision, string, text, wem, wr, Envelope, Error, Slot,
};
pub(in crate::model::conversion::m3) fn layer<E: Envelope, D: nr::TextureLayerDialect>(
    r: nr::TextureLayer<D>,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3TextureLayer, Error>
where
    D::NoiseAmplitude: Slot<f32>,
    D::NoiseFrequency: Slot<f32>,
    D::TriplanarOffset: Slot<nr::AnimationValue<[f32; 3]>>,
    D::TriplanarScale: Slot<nr::AnimationValue<[f32; 3]>>,
    D::FresnelTranslation: Slot<[f32; 3]>,
    D::FresnelMask: Slot<[f32; 3]>,
    D::FresnelRotation: Slot<[f32; 2]>,
    D::UvDensity: Slot<u32>,
{
    Ok(wr::M3TextureLayer {
        id: r.id,
        texture_path: text(doc, &string(source, r.texture_path).map_err(read_error)?)?,
        color: color(r.color),
        flags: wr::M3TextureLayerFlag((r.flags).bits()),
        uv_mapping: wr::M3UVMappingMode::from_raw((r.uv_mapping).raw()),
        color_type: wr::M3ColorChannelSelect::from_raw((r.color_type).raw()),
        rgb_multiply: anim(r.rgb_multiply),
        rgb_add: anim(r.rgb_add),
        poc_texture: r.poc_texture,
        noise_amplitude: r.noise_amplitude.value(),
        noise_frequency: r.noise_frequency.value(),
        texture_source: r.texture_source,
        avi_frame_rate: r.avi_frame_rate,
        avi_start: r.avi_start,
        avi_stop: r.avi_stop,
        avi_loop: r.avi_loop,
        avi_sync: r.avi_sync,
        avi_play: anim(r.avi_play),
        avi_restart: anim(r.avi_restart),
        flipbook_rows: r.flipbook_rows,
        flipbook_columns: r.flipbook_columns,
        current_frame: anim(r.current_frame),
        uv_offset: anim(r.uv_offset),
        uv_angle: anim(r.uv_angle),
        uv_tiling: anim(r.uv_tiling),
        w_offset: anim(r.w_offset),
        w_tiling: anim(r.w_tiling),
        map_alpha: anim(r.map_alpha),
        triplanar_offset: anim(r.triplanar_offset.value()),
        triplanar_scale: anim(r.triplanar_scale.value()),
        uv_source_related: r.uv_source_related,
        fresnel_mode: wr::M3FresnelMode::from_raw((r.fresnel_mode).raw()),
        fresnel_exponent: r.fresnel_exponent,
        fresnel_min: r.fresnel_min,
        fresnel_max: r.fresnel_max,
        fresnel_translation: r.fresnel_translation.value(),
        fresnel_mask: r.fresnel_mask.value(),
        fresnel_rotation: r.fresnel_rotation.value(),
        uv_density: r.uv_density.value(),
    })
}

pub(in crate::model::conversion::m3) fn get_layer<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::TextureLayer>,
) -> Result<wem::InlineOption<wr::M3TextureLayer>, Error> {
    if reference.entries == 0 {
        return Ok(wem::InlineOption::default());
    }
    macro_rules! read {
        ($v:ty) => {{
            let r = source
                .resolve::<nr::TextureLayer<$v>>(reference.cast())
                .map_err(read_error)?
                .get(0)
                .ok_or_else(|| Error::new("m3.material.layer", "empty layer"))?
                .map_err(read_error)?;
            layer(r, source, doc)
        }};
    }
    let value = match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::V0),
        1 => read!(nr::V1),
        2 => read!(nr::V2),
        3 => read!(nr::V3),
        4 => read!(nr::V4),
        5 => read!(nr::V5),
        6 => read!(nr::V6),
        7 => read!(nr::V7),
        8 => read!(nr::V8),
        9 => read!(nr::V9),
        10 => read!(nr::V10),
        11 => read!(nr::V11),
        12 => read!(nr::V12),
        13 => read!(nr::V13),
        14 => read!(nr::V14),
        15 => read!(nr::V15),
        16 => read!(nr::V16),
        17 => read!(nr::V17),
        18 => read!(nr::V18),
        19 => read!(nr::V19),
        20 => read!(nr::V20),
        21 => read!(nr::V21),
        22 => read!(nr::V22),
        23 => read!(nr::V23),
        24 => read!(nr::V24),
        25 => read!(nr::V25),
        26 => read!(nr::V26),
        _ => {
            return Err(Error::new(
                "m3.material.layer",
                "unsupported layer revision",
            ));
        }
    }?;
    Ok(wem::InlineOption::from(Some(value)))
}

pub(in crate::model::conversion::m3) fn native_layer<E: Envelope>(
    r: wr::M3TextureLayer,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::TextureLayer<nr::V26>, Error> {
    Ok(nr::TextureLayer {
        id: r.id,
        texture_path: append_string(doc, source.string(r.texture_path)?.unwrap_or_default())?,
        color: native_color(r.color),
        flags: nr::LayerFlags(r.flags.bits()),
        uv_mapping: nr::UvMappingMode::from_raw(r.uv_mapping.raw()),
        color_type: nr::ColorChannel::from_raw(r.color_type.raw()),
        rgb_multiply: native_anim(r.rgb_multiply),
        rgb_add: native_anim(r.rgb_add),
        poc_texture: r.poc_texture,
        noise_amplitude: r.noise_amplitude,
        noise_frequency: r.noise_frequency,
        texture_source: r.texture_source,
        avi_frame_rate: r.avi_frame_rate,
        avi_start: r.avi_start,
        avi_stop: r.avi_stop,
        avi_loop: r.avi_loop,
        avi_sync: r.avi_sync,
        avi_play: native_anim(r.avi_play),
        avi_restart: native_anim(r.avi_restart),
        flipbook_rows: r.flipbook_rows,
        flipbook_columns: r.flipbook_columns,
        current_frame: native_anim(r.current_frame),
        uv_offset: native_anim(r.uv_offset),
        uv_angle: native_anim(r.uv_angle),
        uv_tiling: native_anim(r.uv_tiling),
        w_offset: native_anim(r.w_offset),
        w_tiling: native_anim(r.w_tiling),
        map_alpha: native_anim(r.map_alpha),
        triplanar_offset: native_anim(r.triplanar_offset),
        triplanar_scale: native_anim(r.triplanar_scale),
        uv_source_related: r.uv_source_related,
        fresnel_mode: nr::FresnelMode::from_raw(r.fresnel_mode.raw()),
        fresnel_exponent: r.fresnel_exponent,
        fresnel_min: r.fresnel_min,
        fresnel_max: r.fresnel_max,
        fresnel_translation: r.fresnel_translation,
        fresnel_mask: r.fresnel_mask,
        fresnel_rotation: r.fresnel_rotation,
        uv_density: (),
    })
}

pub(in crate::model::conversion::m3) fn native_layer_ref<E: Envelope>(
    r: wem::InlineOption<wr::M3TextureLayer>,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<native::Ref<nr::targets::TextureLayer>, Error> {
    if let Some(layer) = r.value {
        let layer = native_layer(layer, source, doc)?;
        Ok(append(doc, &[layer])?.cast())
    } else {
        Ok(native::Ref::default())
    }
}
