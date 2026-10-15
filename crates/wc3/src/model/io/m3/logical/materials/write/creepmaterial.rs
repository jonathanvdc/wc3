//! Mechanical creep surface encoding.
use super::{nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::Creep,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::CreepMaterial<nr::V1>, Error> {
    Ok(nr::CreepMaterial {
        name: writer.string(name)?,
        mask_map: writer.layer(r.mask_map.as_ref())?,
        creep_low: r.creep_low,
    })
}
