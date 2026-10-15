//! Flare element and optical response encoding.
use super::{
    append, native_anim, native_color, nr, quantize_color, surface, Envelope, Error, Writer,
};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::LensFlare<'_>,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::LensFlare<nr::V3>, Error> {
    let sub = r
        .sub_flares
        .iter()
        .map(|r| {
            Ok(nr::SubFlare::<nr::V2> {
                index: r.index,
                position: r.position,
                size_x_y: r.size_xy,
                scale_x_y: r.scale_xy,
                fade_in: r.fade_in,
                fade_out: r.fade_out,
                color_alpha: quantize_color(r.color_alpha, writer.context)?,
                face_center: r.face_center,
                offset: r.offset,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    let color = native_color(&r.color, writer)?;
    Ok(nr::LensFlare {
        name: writer.string(name)?,
        flare_map: writer.layer(r.flare_map.as_ref())?,
        mask_map: writer.layer(r.mask_map.as_ref())?,
        sub_flares: append(writer.document, &sub)?.cast(),
        columns: r.columns,
        rows: r.rows,
        distance_fade: r.distance_fade,
        lib_name: writer.string(&r.lib_name)?,
        intensity: native_anim(&r.intensity, writer.animation_ids),
        color: nr::AnimationValue {
            interpolation: color.interpolation,
            flags: color.flags,
            animation_id: color.animation_id,
            initial_value: nr::FixedBytes(color.initial_value),
            null_value: nr::FixedBytes(color.null_value),
            reserved: color.reserved,
        },
        hdr: native_anim(&r.hdr, writer.animation_ids),
        size: native_anim(&r.size, writer.animation_ids),
    })
}
