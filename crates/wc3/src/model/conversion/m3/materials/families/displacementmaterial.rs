//! displacementmaterial body mapping.
use super::{
    anim, append_string, get_layer, native, native_anim, native_layer_ref, nr, read_error,
    revision, string, text, wem, wr, Envelope, Error, Slot,
};
pub(in crate::model::conversion::m3) fn map_displacementmaterial<
    E: Envelope,
    D: nr::DisplacementMaterialDialect,
>(
    r: nr::DisplacementMaterial<D>,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3Displacement, Error>
where
    D::Name: Slot<native::Ref<nr::targets::Character>>,
{
    Ok(wr::M3Displacement {
        name: text(doc, &string(source, r.name.value()).map_err(read_error)?)?,
        unknown: r.unknown,
        strength: anim(r.strength),
        normal_map: get_layer(source, doc, r.normal_map)?,
        strength_map: get_layer(source, doc, r.strength_map)?,
        flags: r.flags,
        priority: r.priority,
    })
}

pub(in crate::model::conversion::m3) fn get_displacementmaterial<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::DisplacementMaterial>,
    index: usize,
) -> Result<wr::M3Displacement, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_displacementmaterial(r, source, doc)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::DisplacementMaterial<nr::V0>),
        1 => read!(nr::DisplacementMaterial<nr::V1>),
        2 => read!(nr::DisplacementMaterial<nr::V2>),
        3 => read!(nr::DisplacementMaterial<nr::V3>),
        4 => read!(nr::DisplacementMaterial<nr::V4>),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}

pub(in crate::model::conversion::m3) fn export_displacementmaterial<E: Envelope>(
    r: wr::M3Displacement,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::DisplacementMaterial<nr::V4>, Error> {
    Ok(nr::DisplacementMaterial {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        unknown: r.unknown,
        strength: native_anim(r.strength),
        normal_map: native_layer_ref(r.normal_map, source, doc)?,
        strength_map: native_layer_ref(r.strength_map, source, doc)?,
        flags: r.flags,
        priority: r.priority,
    })
}
