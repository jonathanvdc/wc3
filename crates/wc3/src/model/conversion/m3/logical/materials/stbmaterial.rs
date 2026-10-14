//! Direct logical stbmaterial decoding from native M3 fields.
use super::texture::get_layer;
use super::{native, nr, read_error, revision, string, surface, Envelope, Error, Reader};
pub(super) fn map_stbmaterial<'a, E: Envelope>(
    r: nr::STBMaterial,
    source: &'a native::Document<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::SplatTerrainBake, Error> {
    reader.name = Some(string(source, r.name)?);
    Ok(surface::SplatTerrainBake {
        diffuse_map: get_layer(source, reader, r.diffuse_map)?,
        normal_map: get_layer(source, reader, r.normal_map)?,
        specular_map: get_layer(source, reader, r.specular_map)?,
    })
}

pub(super) fn get_stbmaterial<'a, E: Envelope>(
    source: &'a native::Document<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::STBMaterial>,
    index: usize,
) -> Result<surface::SplatTerrainBake, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_stbmaterial(r, source, reader)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::STBMaterial),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}
