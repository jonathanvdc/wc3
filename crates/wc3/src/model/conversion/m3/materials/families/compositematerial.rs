//! compositematerial body mapping.
use super::{
    anim, append, append_string, native, native_anim, nr, read_error, revision, string, text, wem,
    wr, Envelope, Error, Slot,
};
pub(in crate::model::conversion::m3) fn map_compositematerial<
    E: Envelope,
    D: nr::CompositeMaterialDialect,
>(
    r: nr::CompositeMaterial<D>,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3Composite, Error>
where
    D::Name: Slot<native::Ref<nr::targets::Character>>,
{
    Ok(wr::M3Composite {
        name: text(doc, &string(source, r.name.value()).map_err(read_error)?)?,
        priority: r.priority,
        sections: wem::InlineList(
            source
                .resolve::<nr::CompositeSection>(r.sections.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| {
                    r.map(|r| wr::M3CompositeSection {
                        material_index: r.material_index,
                        map_multiplier: anim(r.map_multiplier),
                    })
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(read_error)?,
        ),
    })
}

pub(in crate::model::conversion::m3) fn get_compositematerial<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::CompositeMaterial>,
    index: usize,
) -> Result<wr::M3Composite, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_compositematerial(r, source, doc)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::CompositeMaterial<nr::V0>),
        1 => read!(nr::CompositeMaterial<nr::V1>),
        2 => read!(nr::CompositeMaterial<nr::V2>),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}

pub(in crate::model::conversion::m3) fn export_compositematerial<E: Envelope>(
    r: wr::M3Composite,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::CompositeMaterial<nr::V2>, Error> {
    Ok(nr::CompositeMaterial {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        priority: r.priority,
        sections: append(
            doc,
            &r.sections
                .0
                .into_iter()
                .map(|r| nr::CompositeSection {
                    material_index: r.material_index,
                    map_multiplier: native_anim(r.map_multiplier),
                })
                .collect::<Vec<_>>(),
        )?
        .cast(),
    })
}
