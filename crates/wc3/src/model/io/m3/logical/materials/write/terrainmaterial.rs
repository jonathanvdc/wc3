//! Mechanical terrain surface encoding.
use super::{nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::Terrain,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::TerrainMaterial<nr::V1>, Error> {
    Ok(nr::TerrainMaterial {
        name: writer.string(name)?,
        terrain_map: writer.layer(r.terrain_map.as_ref())?,
        unknown: r.unknown,
    })
}
