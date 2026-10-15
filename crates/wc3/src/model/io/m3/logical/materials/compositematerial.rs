//! Direct logical compositematerial decoding from native M3 fields.

use super::{
    anim, identity, native, nr, read_error, revision, string, surface, Envelope, Error, Reader,
    Slot,
};
pub(super) fn map_compositematerial<'a, E: Envelope, D: nr::CompositeMaterialDialect>(
    r: nr::CompositeMaterial<D>,
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::Composite, Error>
where
    D::Name: Slot<native::Ref<nr::targets::Character>>,
{
    reader.name = Some(string(source, r.name.value())?);
    Ok(surface::Composite {
        priority: r.priority,
        sections: identity(
            source
                .resolve::<nr::CompositeSection>(r.sections.cast())
                .map_err(read_error)?
                .iter()
                .map(|r| {
                    let r = r.map_err(read_error)?;
                    Ok(surface::CompositeSection {
                        material: reader.material(r.material_index)?,
                        map_multiplier: anim(r.map_multiplier),
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?,
        ),
    })
}

pub(super) fn get_compositematerial<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::CompositeMaterial>,
    index: usize,
) -> Result<surface::Composite, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_compositematerial(r, source, reader)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::CompositeMaterial<nr::V0>),
        1 => read!(nr::CompositeMaterial<nr::V1>),
        2 => read!(nr::CompositeMaterial<nr::V2>),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}
