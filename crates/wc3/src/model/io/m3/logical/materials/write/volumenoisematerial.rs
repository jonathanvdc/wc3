//! Mechanical volumenoise surface encoding.
use super::{native_anim, nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::VolumeNoise,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::VolumeNoiseMaterial, Error> {
    Ok(nr::VolumeNoiseMaterial {
        name: writer.string(name)?,
        falloff_type: nr::VolumeFalloff::from_raw(r.falloff_type.raw()),
        draw_transparency: nr::VolumeCameraMode::from_raw(r.draw_transparency.raw()),
        density: native_anim(&r.density, writer.animation_ids),
        near_plane: native_anim(&r.near_plane, writer.animation_ids),
        falloff: native_anim(&r.falloff, writer.animation_ids),
        color_map: writer.layer(r.color_map.as_ref())?,
        noise_map1: writer.layer(r.noise_map1.as_ref())?,
        noise_map2: writer.layer(r.noise_map2.as_ref())?,
        scroll_rate: native_anim(&r.scroll_rate, writer.animation_ids),
        position: native_anim(&r.position, writer.animation_ids),
        scale: native_anim(&r.scale, writer.animation_ids),
        rotation: native_anim(&r.rotation, writer.animation_ids),
        alpha_threshold: r.alpha_threshold,
        flags: nr::VolumeNoiseFlags(r.flags.bits()),
    })
}
