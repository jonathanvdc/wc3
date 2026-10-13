//! volumenoisematerial body mapping.
use super::{
    anim, append_string, get_layer, native, native_anim, native_layer_ref, nr, read_error,
    revision, string, text, wem, wr, Envelope, Error,
};
pub(in crate::model::conversion::m3) fn map_volumenoisematerial<E: Envelope>(
    r: nr::VolumeNoiseMaterial,
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
) -> Result<wr::M3VolumeNoise, Error> {
    Ok(wr::M3VolumeNoise {
        name: text(doc, &string(source, r.name).map_err(read_error)?)?,
        falloff_type: wr::M3VolumeFalloffType::from_raw((r.falloff_type).raw()),
        draw_transparency: wr::M3VolumeNoiseCameraMode::from_raw((r.draw_transparency).raw()),
        density: anim(r.density),
        near_plane: anim(r.near_plane),
        falloff: anim(r.falloff),
        color_map: get_layer(source, doc, r.color_map)?,
        noise_map1: get_layer(source, doc, r.noise_map1)?,
        noise_map2: get_layer(source, doc, r.noise_map2)?,
        scroll_rate: anim(r.scroll_rate),
        position: anim(r.position),
        scale: anim(r.scale),
        rotation: anim(r.rotation),
        alpha_threshold: r.alpha_threshold,
        flags: wr::M3VolumeNoiseMaterialFlag((r.flags).bits()),
    })
}

pub(in crate::model::conversion::m3) fn get_volumenoisematerial<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    reference: native::Ref<nr::targets::VolumeNoiseMaterial>,
    index: usize,
) -> Result<wr::M3VolumeNoise, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_volumenoisematerial(r, source, doc)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::VolumeNoiseMaterial),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}

pub(in crate::model::conversion::m3) fn export_volumenoisematerial<E: Envelope>(
    r: wr::M3VolumeNoise,
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
) -> Result<nr::VolumeNoiseMaterial, Error> {
    Ok(nr::VolumeNoiseMaterial {
        name: append_string(doc, source.string(r.name)?.unwrap_or_default())?,
        falloff_type: nr::VolumeFalloff::from_raw(r.falloff_type.raw()),
        draw_transparency: nr::VolumeCameraMode::from_raw(r.draw_transparency.raw()),
        density: native_anim(r.density),
        near_plane: native_anim(r.near_plane),
        falloff: native_anim(r.falloff),
        color_map: native_layer_ref(r.color_map, source, doc)?,
        noise_map1: native_layer_ref(r.noise_map1, source, doc)?,
        noise_map2: native_layer_ref(r.noise_map2, source, doc)?,
        scroll_rate: native_anim(r.scroll_rate),
        position: native_anim(r.position),
        scale: native_anim(r.scale),
        rotation: native_anim(r.rotation),
        alpha_threshold: r.alpha_threshold,
        flags: nr::VolumeNoiseFlags(r.flags.bits()),
    })
}
