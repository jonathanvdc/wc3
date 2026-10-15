//! Mechanical reflection surface encoding.
use super::{native_anim, nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::Reflection,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::ReflectionMaterial<nr::V3>, Error> {
    Ok(nr::ReflectionMaterial {
        name: writer.string(name)?,
        unknown: r.unknown,
        reflection_strength: native_anim(&r.reflection.parameters, writer.animation_ids),
        displacement_strength: native_anim(&r.displacement.parameters, writer.animation_ids),
        reflection_offset: native_anim(&r.reflection_offset, writer.animation_ids),
        blur_angle: native_anim(&r.blur_angle, writer.animation_ids),
        blur_distance_max: native_anim(&r.blur_distance_max, writer.animation_ids),
        reflection_map: writer.layer(r.reflection.input.as_ref())?,
        displacement_map: writer.layer(r.displacement.input.as_ref())?,
        blur_map: writer.layer(r.blur_map.as_ref())?,
        flags: nr::ReflectionFlags(r.flags.bits()),
        unknown2: r.unknown2,
    })
}
