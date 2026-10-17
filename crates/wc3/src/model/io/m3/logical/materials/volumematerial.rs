//! Direct logical volumematerial decoding from native M3 fields.
use super::texture::get_layer;
use super::{anim, native, nr, read_error, revision, string, surface, Envelope, Error, Reader};
pub(super) fn map_volumematerial<'a, E: Envelope>(
    r: nr::VolumeMaterial,
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
) -> Result<surface::VolumeMaterialStep, Error> {
    reader.name = Some(string(source, r.name)?);
    Ok(surface::VolumeMaterialStep {
        blend_mode: surface::values::BlendMode::from_raw((r.blend_mode).raw()),
        falloff_type: surface::values::VolumeFalloff::from_raw((r.falloff_type).raw()),
        density: anim(r.density, reader.builder.values_mut()),
        color_map: get_layer(source, reader, r.color_map)?,
        noise_map1: get_layer(source, reader, r.noise_map1)?,
        noise_map2: get_layer(source, reader, r.noise_map2)?,
        alpha_threshold: r.alpha_threshold,
        flags: surface::values::VolumeFlags((surface::values::LayerFlags((r.flags).bits())).bits()),
    })
}

pub(super) fn get_volumematerial<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::VolumeMaterial>,
    index: usize,
) -> Result<surface::VolumeMaterialStep, Error> {
    macro_rules! read {
        ($t:ty) => {{
            let r = source
                .resolve::<$t>(reference.cast())
                .map_err(read_error)?
                .get(index)
                .ok_or_else(|| Error::new("m3.material", "material index out of range"))?
                .map_err(read_error)?;
            map_volumematerial(r, source, reader)
        }};
    }
    match revision(source, reference).map_err(read_error)? {
        0 => read!(nr::VolumeMaterial),
        _ => Err(Error::new("m3.material", "unsupported material revision")),
    }
}
