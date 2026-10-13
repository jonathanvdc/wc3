//! reflectionmaterial body mapping.
use super::{
    anim, append_string, get_layer, native, native_anim, native_layer_ref, nr, read_error,
    revision, string, text, wem, wr, Envelope, Error, Slot,
};
pub(in crate::model::conversion::m3) fn map_reflectionmaterial<
    E: Envelope,
    D: nr::ReflectionMaterialDialect,
>(
    r: nr::ReflectionMaterial<D>,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3Reflection, Error>
where
    D::ReflectionOffset: Slot<nr::AnimationValue<f32>>,
    D::BlurAngle: Slot<nr::AnimationValue<f32>>,
    D::BlurDistanceMax: Slot<nr::AnimationValue<f32>>,
    D::BlurMap: Slot<native::Ref<nr::targets::TextureLayer>>,
    D::Unknown2: Slot<u32>,
{
    Ok(wr::M3Reflection {
        name: text(doc, &string(source, r.name).map_err(read_error)?)?,
        unknown: r.unknown,
        reflection_strength: anim(r.reflection_strength),
        displacement_strength: anim(r.displacement_strength),
        reflection_offset: anim(r.reflection_offset.value()),
        blur_angle: anim(r.blur_angle.value()),
        blur_distance_max: anim(r.blur_distance_max.value()),
        reflection_map: get_layer(source, doc, r.reflection_map)?,
        displacement_map: get_layer(source, doc, r.displacement_map)?,
        blur_map: get_layer(source, doc, r.blur_map.value())?,
        flags: wr::M3ReflectionMaterialFlag(r.flags),
        unknown2: r.unknown2.value(),
    })
}

pub(in crate::model::conversion::m3) fn get_reflectionmaterial<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::ReflectionMaterial>,
    index: usize,
) -> Result<wr::M3Reflection, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_reflectionmaterial(r, source, doc)
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

pub(in crate::model::conversion::m3) fn export_reflectionmaterial<E: Envelope>(
    r: wr::M3Reflection,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::ReflectionMaterial<nr::V3>, Error> {
    Ok(nr::ReflectionMaterial {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        unknown: r.unknown,
        reflection_strength: native_anim(r.reflection_strength),
        displacement_strength: native_anim(r.displacement_strength),
        reflection_offset: native_anim(r.reflection_offset),
        blur_angle: native_anim(r.blur_angle),
        blur_distance_max: native_anim(r.blur_distance_max),
        reflection_map: native_layer_ref(r.reflection_map, source, doc)?,
        displacement_map: native_layer_ref(r.displacement_map, source, doc)?,
        blur_map: native_layer_ref(r.blur_map, source, doc)?,
        flags: r.flags.0,
        unknown2: r.unknown2,
    })
}
