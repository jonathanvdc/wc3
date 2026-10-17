//! Mechanical volume surface encoding.
use super::{native_anim, nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::VolumeMaterialStep,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::VolumeMaterial, Error> {
    Ok(nr::VolumeMaterial {
        name: writer.string(name)?,
        blend_mode: nr::BlendMode::from_raw((r.blend_mode).raw()),
        falloff_type: nr::VolumeFalloff::from_raw(r.falloff_type.raw()),
        density: native_anim(&r.density, writer.animation_ids),
        color_map: writer.layer(r.color_map.as_ref())?,
        noise_map1: writer.layer(r.noise_map1.as_ref())?,
        noise_map2: writer.layer(r.noise_map2.as_ref())?,
        alpha_threshold: r.alpha_threshold,
        flags: nr::VolumeFlags((r.flags).bits()),
    })
}
