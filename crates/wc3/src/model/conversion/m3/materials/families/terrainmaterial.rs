//! terrainmaterial body mapping.
use super::{
    append_string, get_layer, native, native_layer_ref, nr, read_error, revision, string, text,
    wem, wr, Envelope, Error, Slot,
};
pub(in crate::model::conversion::m3) fn map_terrainmaterial<
    E: Envelope,
    D: nr::TerrainMaterialDialect,
>(
    r: nr::TerrainMaterial<D>,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3Terrain, Error>
where
    D::Unknown: Slot<u32>,
{
    Ok(wr::M3Terrain {
        name: text(doc, &string(source, r.name).map_err(read_error)?)?,
        terrain_map: get_layer(source, doc, r.terrain_map)?,
        unknown: r.unknown.value(),
    })
}

pub(in crate::model::conversion::m3) fn get_terrainmaterial<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::TerrainMaterial>,
    index: usize,
) -> Result<wr::M3Terrain, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_terrainmaterial(r, source, doc)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::TerrainMaterial<nr::V0>),
        1 => read!(nr::TerrainMaterial<nr::V1>),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}

pub(in crate::model::conversion::m3) fn export_terrainmaterial<E: Envelope>(
    r: wr::M3Terrain,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::TerrainMaterial<nr::V1>, Error> {
    Ok(nr::TerrainMaterial {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        terrain_map: native_layer_ref(r.terrain_map, source, doc)?,
        unknown: r.unknown,
    })
}
