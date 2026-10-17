//! Direct logical volumenoisematerial decoding from native M3 fields.
use super::texture::get_layer;
use super::{anim, native, nr, read_error, revision, string, surface, Envelope, Error, Reader};
pub(super) fn map_volumenoisematerial<'a, E: Envelope>(
    r: nr::VolumeNoiseMaterial,
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::VolumeNoiseMaterialStep, Error> {
    reader.name = Some(string(source, r.name)?);
    Ok(surface::VolumeNoiseMaterialStep {
        falloff_type: surface::values::VolumeFalloff::from_raw((r.falloff_type).raw()),
        draw_transparency: surface::values::VolumeCameraMode::from_raw((r.draw_transparency).raw()),
        density: anim(r.density, reader.builder.values_mut()),
        near_plane: anim(r.near_plane, reader.builder.values_mut()),
        falloff: anim(r.falloff, reader.builder.values_mut()),
        color_map: get_layer(source, reader, r.color_map)?,
        noise_map1: get_layer(source, reader, r.noise_map1)?,
        noise_map2: get_layer(source, reader, r.noise_map2)?,
        scroll_rate: anim(r.scroll_rate, reader.builder.values_mut()),
        position: anim(r.position, reader.builder.values_mut()),
        scale: anim(r.scale, reader.builder.values_mut()),
        rotation: anim(r.rotation, reader.builder.values_mut()),
        alpha_threshold: r.alpha_threshold,
        flags: surface::values::VolumeNoiseFlags((r.flags).bits()),
    })
}

pub(super) fn get_volumenoisematerial<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::VolumeNoiseMaterial>,
    index: usize,
) -> Result<surface::VolumeNoiseMaterialStep, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_volumenoisematerial(r, source, reader)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::VolumeNoiseMaterial),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}
