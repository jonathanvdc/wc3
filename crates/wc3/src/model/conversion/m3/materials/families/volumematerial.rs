//! volumematerial body mapping.
use super::{
    anim, append_string, get_layer, native, native_anim, native_layer_ref, nr, read_error,
    revision, string, text, wem, wr, Envelope, Error,
};
pub(in crate::model::conversion::m3) fn map_volumematerial<E: Envelope>(
    r: nr::VolumeMaterial,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3Volume, Error> {
    Ok(wr::M3Volume {
        name: text(doc, &string(source, r.name).map_err(read_error)?)?,
        blend_mode: r.blend_mode,
        falloff_type: wr::M3VolumeFalloffType(r.falloff_type),
        density: anim(r.density),
        color_map: get_layer(source, doc, r.color_map)?,
        noise_map1: get_layer(source, doc, r.noise_map1)?,
        noise_map2: get_layer(source, doc, r.noise_map2)?,
        alpha_threshold: r.alpha_threshold,
        flags: r.flags,
    })
}

pub(in crate::model::conversion::m3) fn get_volumematerial<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::VolumeMaterial>,
    index: usize,
) -> Result<wr::M3Volume, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_volumematerial(r, source, doc)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::VolumeMaterial),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}

pub(in crate::model::conversion::m3) fn export_volumematerial<E: Envelope>(
    r: wr::M3Volume,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::VolumeMaterial, Error> {
    Ok(nr::VolumeMaterial {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        blend_mode: r.blend_mode,
        falloff_type: r.falloff_type.0,
        density: native_anim(r.density),
        color_map: native_layer_ref(r.color_map, source, doc)?,
        noise_map1: native_layer_ref(r.noise_map1, source, doc)?,
        noise_map2: native_layer_ref(r.noise_map2, source, doc)?,
        alpha_threshold: r.alpha_threshold,
        flags: r.flags,
    })
}
