//! Direct logical creepmaterial decoding from native M3 fields.
use super::texture::get_layer;
use super::{native, nr, read_error, revision, string, surface, Envelope, Error, Reader, Slot};
pub(super) fn map_creepmaterial<'a, E: Envelope, D: nr::CreepMaterialDialect>(
    r: nr::CreepMaterial<D>,
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::CreepMaterialStep, Error>
where
    D::CreepLow: Slot<u32>,
{
    reader.name = Some(string(source, r.name)?);
    Ok(surface::CreepMaterialStep {
        mask_map: get_layer(source, reader, r.mask_map)?,
        creep_low: r.creep_low.value(),
    })
}

pub(super) fn get_creepmaterial<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::CreepMaterial>,
    index: usize,
) -> Result<surface::CreepMaterialStep, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_creepmaterial(r, source, reader)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::CreepMaterial<nr::V0>),
        1 => read!(nr::CreepMaterial<nr::V1>),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}
