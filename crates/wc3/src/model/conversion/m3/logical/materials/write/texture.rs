//! Layer encoding preserves authored UV, video, color, and fresnel controls.
use super::{native_anim, native_color, nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    r: &surface::TextureLayer,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::TextureLayer<nr::V25>, Error> {
    let path = match r.binding.image {
        Some(id) => writer.texture(id)?,
        None => "",
    };
    Ok(nr::TextureLayer {
        id: r.id,
        texture_path: writer.string(path)?,
        color: native_color(&r.binding.contribution.color, writer)?,
        flags: nr::LayerFlags(r.binding.sampling.flags.bits()),
        uv_mapping: nr::UvMappingMode::from_raw(r.binding.coordinates.source.raw()),
        color_type: nr::ColorChannel::from_raw(r.binding.contribution.color_type.raw()),
        rgb_multiply: native_anim(&r.binding.contribution.rgb_multiply, writer.animation_ids),
        rgb_add: native_anim(&r.binding.contribution.rgb_add, writer.animation_ids),
        poc_texture: r.binding.sampling.poc_texture,
        noise_amplitude: r.binding.sampling.noise_amplitude,
        noise_frequency: r.binding.sampling.noise_frequency,
        texture_source: r.binding.sampling.texture_source,
        avi_frame_rate: r.binding.sampling.avi_frame_rate,
        avi_start: r.binding.sampling.avi_start,
        avi_stop: r.binding.sampling.avi_stop,
        avi_loop: r.binding.sampling.avi_loop,
        avi_sync: r.binding.sampling.avi_sync,
        avi_play: native_anim(&r.binding.sampling.avi_play, writer.animation_ids),
        avi_restart: native_anim(&r.binding.sampling.avi_restart, writer.animation_ids),
        flipbook_rows: r.binding.sampling.flipbook_rows,
        flipbook_columns: r.binding.sampling.flipbook_columns,
        current_frame: native_anim(&r.binding.sampling.current_frame, writer.animation_ids),
        uv_offset: native_anim(
            &r.binding.coordinates.transform.uv_offset,
            writer.animation_ids,
        ),
        uv_angle: native_anim(
            &r.binding.coordinates.transform.uv_angle,
            writer.animation_ids,
        ),
        uv_tiling: native_anim(
            &r.binding.coordinates.transform.uv_tiling,
            writer.animation_ids,
        ),
        w_offset: native_anim(
            &r.binding.coordinates.transform.w_offset,
            writer.animation_ids,
        ),
        w_tiling: native_anim(
            &r.binding.coordinates.transform.w_tiling,
            writer.animation_ids,
        ),
        map_alpha: native_anim(&r.binding.contribution.map_alpha, writer.animation_ids),
        triplanar_offset: native_anim(
            &r.binding.coordinates.transform.triplanar_offset,
            writer.animation_ids,
        ),
        triplanar_scale: native_anim(
            &r.binding.coordinates.transform.triplanar_scale,
            writer.animation_ids,
        ),
        uv_source_related: r.binding.coordinates.transform.uv_source_related,
        fresnel_mode: nr::FresnelMode::from_raw(r.fresnel_mode.raw()),
        fresnel_exponent: r.fresnel_exponent,
        fresnel_min: r.fresnel_min,
        fresnel_max: r.fresnel_max,
        fresnel_translation: r.fresnel_translation,
        fresnel_mask: r.fresnel_mask,
        fresnel_rotation: r.fresnel_rotation,
        uv_density: r.binding.coordinates.transform.uv_density,
    })
}
