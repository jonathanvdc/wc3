//! stbmaterial body mapping.
use super::{
    append_string, get_layer, native, native_layer_ref, nr, read_error, revision, string, text,
    wem, wr, Envelope, Error,
};
pub(in crate::model::conversion::m3) fn map_stbmaterial<E: Envelope>(
    r: nr::STBMaterial,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3Stb, Error> {
    Ok(wr::M3Stb {
        name: text(doc, &string(source, r.name).map_err(read_error)?)?,
        diffuse_map: get_layer(source, doc, r.diffuse_map)?,
        normal_map: get_layer(source, doc, r.normal_map)?,
        specular_map: get_layer(source, doc, r.specular_map)?,
    })
}

pub(in crate::model::conversion::m3) fn get_stbmaterial<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::STBMaterial>,
    index: usize,
) -> Result<wr::M3Stb, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_stbmaterial(r, source, doc)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::STBMaterial),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}

pub(in crate::model::conversion::m3) fn export_stbmaterial<E: Envelope>(
    r: wr::M3Stb,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::STBMaterial, Error> {
    Ok(nr::STBMaterial {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        diffuse_map: native_layer_ref(r.diffuse_map, source, doc)?,
        normal_map: native_layer_ref(r.normal_map, source, doc)?,
        specular_map: native_layer_ref(r.specular_map, source, doc)?,
    })
}
