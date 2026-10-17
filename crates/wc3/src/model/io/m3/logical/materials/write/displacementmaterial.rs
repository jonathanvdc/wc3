//! Mechanical displacement surface encoding.
use super::{native_anim, nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::DisplacementMaterialStep,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::DisplacementMaterial<nr::V4>, Error> {
    Ok(nr::DisplacementMaterial {
        name: writer.string(name)?,
        unknown: r.unknown,
        strength: native_anim(&r.displacement.parameters, writer.animation_ids),
        normal_map: writer.layer(r.displacement.input.as_ref())?,
        strength_map: writer.layer(r.strength_map.as_ref())?,
        flags: nr::DisplacementFlags((r.flags).bits()),
        priority: r.priority,
    })
}
