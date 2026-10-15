//! Direct logical reflectionmaterial decoding from native M3 fields.
use super::texture::get_layer;
use super::{
    anim, native, nr, read_error, revision, string, surface, Envelope, Error, Reader, Slot,
};
use crate::model::semantic::materials::contributions::SurfaceContribution;
pub(super) fn map_reflectionmaterial<'a, E: Envelope, D: nr::ReflectionMaterialDialect>(
    r: nr::ReflectionMaterial<D>,
    source: &'a native::Document<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::Reflection, Error>
where
    D::ReflectionOffset: Slot<nr::AnimationValue<f32>>,
    D::BlurAngle: Slot<nr::AnimationValue<f32>>,
    D::BlurDistanceMax: Slot<nr::AnimationValue<f32>>,
    D::BlurMap: Slot<native::Ref<nr::targets::TextureLayer>>,
    D::Unknown2: Slot<u32>,
{
    reader.name = Some(string(source, r.name)?);
    let reflection_map = get_layer(source, reader, r.reflection_map)?;
    let displacement_map = get_layer(source, reader, r.displacement_map)?;
    let blur_map = get_layer(source, reader, r.blur_map.value())?;
    Ok(surface::Reflection {
        displacement: SurfaceContribution {
            input: displacement_map,
            parameters: anim(r.displacement_strength),
        },
        reflection: SurfaceContribution {
            input: reflection_map,
            parameters: anim(r.reflection_strength),
        },
        unknown: r.unknown,
        reflection_offset: anim(r.reflection_offset.value()),
        blur_angle: anim(r.blur_angle.value()),
        blur_distance_max: anim(r.blur_distance_max.value()),
        blur_map: blur_map,
        flags: surface::values::ReflectionFlags((r.flags).bits()),
        unknown2: r.unknown2.value(),
    })
}

pub(super) fn get_reflectionmaterial<'a, E: Envelope>(
    source: &'a native::Document<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::ReflectionMaterial>,
    index: usize,
) -> Result<surface::Reflection, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_reflectionmaterial(r, source, reader)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::ReflectionMaterial<nr::V0>),
        1 => read!(nr::ReflectionMaterial<nr::V1>),
        2 => read!(nr::ReflectionMaterial<nr::V2>),
        3 => read!(nr::ReflectionMaterial<nr::V3>),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}
