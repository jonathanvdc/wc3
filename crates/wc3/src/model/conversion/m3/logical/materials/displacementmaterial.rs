//! Direct logical displacementmaterial decoding from native M3 fields.
use super::texture::get_layer;
use super::{
    anim, native, nr, read_error, revision, string, surface, Envelope, Error, Reader, Slot,
};
use crate::model::semantic::materials::contributions::SurfaceContribution;
pub(super) fn map_displacementmaterial<'a, E: Envelope, D: nr::DisplacementMaterialDialect>(
    r: nr::DisplacementMaterial<D>,
    source: &'a native::Document<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::Displacement, Error>
where
    D::Name: Slot<native::Ref<nr::targets::Character>>,
{
    reader.name = Some(string(source, r.name.value())?);
    let normal_map = get_layer(source, reader, r.normal_map)?;
    let strength_map = get_layer(source, reader, r.strength_map)?;
    Ok(surface::Displacement {
        displacement: SurfaceContribution {
            input: normal_map,
            parameters: anim(r.strength),
        },
        unknown: r.unknown,
        strength_map: strength_map,
        flags: surface::values::DisplacementFlags(
            (surface::values::LayerFlags((r.flags).bits())).bits(),
        ),
        priority: r.priority,
    })
}

pub(super) fn get_displacementmaterial<'a, E: Envelope>(
    source: &'a native::Document<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::DisplacementMaterial>,
    index: usize,
) -> Result<surface::Displacement, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_displacementmaterial(r, source, reader)
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
