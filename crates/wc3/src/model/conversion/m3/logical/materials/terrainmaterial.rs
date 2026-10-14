//! Direct logical terrainmaterial decoding from native M3 fields.
use super::texture::get_layer;
use super::{native, nr, read_error, revision, string, surface, Envelope, Error, Reader, Slot};
pub(super) fn map_terrainmaterial<'a, E: Envelope, D: nr::TerrainMaterialDialect>(
    r: nr::TerrainMaterial<D>,
    source: &'a native::Document<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::Terrain, Error>
where
    D::Unknown: Slot<u32>,
{
    reader.name = Some(string(source, r.name)?);
    Ok(surface::Terrain {
        terrain_map: get_layer(source, reader, r.terrain_map)?,
        unknown: r.unknown.value(),
    })
}

pub(super) fn get_terrainmaterial<'a, E: Envelope>(
    source: &'a native::Document<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::TerrainMaterial>,
    index: usize,
) -> Result<surface::Terrain, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_terrainmaterial(r, source, reader)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::TerrainMaterial<nr::V0>),
        1 => read!(nr::TerrainMaterial<nr::V1>),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}
