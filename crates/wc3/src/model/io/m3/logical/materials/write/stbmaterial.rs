//! Mechanical splatterrainbake surface encoding.
use super::{nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::SplatTerrainBakeMaterialStep,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::STBMaterial, Error> {
    Ok(nr::STBMaterial {
        name: writer.string(name)?,
        diffuse_map: writer.layer(r.diffuse_map.as_ref())?,
        normal_map: writer.layer(r.normal_map.as_ref())?,
        specular_map: writer.layer(r.specular_map.as_ref())?,
    })
}
