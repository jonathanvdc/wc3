//! creepmaterial body mapping.
use super::{
    append_string, get_layer, native, native_layer_ref, nr, read_error, revision, string, text,
    wem, wr, Envelope, Error, Slot,
};
pub(in crate::model::conversion::m3) fn map_creepmaterial<
    E: Envelope,
    D: nr::CreepMaterialDialect,
>(
    r: nr::CreepMaterial<D>,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3Creep, Error>
where
    D::CreepLow: Slot<u32>,
{
    Ok(wr::M3Creep {
        name: text(doc, &string(source, r.name).map_err(read_error)?)?,
        mask_map: get_layer(source, doc, r.mask_map)?,
        creep_low: r.creep_low.value(),
    })
}

pub(in crate::model::conversion::m3) fn get_creepmaterial<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::CreepMaterial>,
    index: usize,
) -> Result<wr::M3Creep, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_creepmaterial(r, source, doc)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::CreepMaterial<nr::V0>),
        1 => read!(nr::CreepMaterial<nr::V1>),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}

pub(in crate::model::conversion::m3) fn export_creepmaterial<E: Envelope>(
    r: wr::M3Creep,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::CreepMaterial<nr::V1>, Error> {
    Ok(nr::CreepMaterial {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        mask_map: native_layer_ref(r.mask_map, source, doc)?,
        creep_low: r.creep_low,
    })
}
