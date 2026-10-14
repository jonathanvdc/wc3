//! Direct anisotropic hair shading values.
use super::texture::get_layer;
use super::{color, native, nr, read_error, string, surface, Envelope, Error, Reader};
pub(super) fn get_hair<'a, E: Envelope>(
    source: &'a native::Document<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::HairMaterial>,
    index: usize,
) -> Result<surface::Hair, Error> {
    let value = source
        .resolve::<nr::HairMaterial>(reference.cast())
        .map_err(read_error)?
        .get(index)
        .ok_or_else(|| Error::new("m3.material.hair", "missing hair material"))?
        .map_err(read_error)?;
    reader.name = Some(string(source, value.name)?);
    Ok(surface::Hair {
        layer_base: get_layer(source, reader, value.layer_base)?,
        layer_spec_shift: get_layer(source, reader, value.layer_spec_shift)?,
        layer_spec_noise: get_layer(source, reader, value.layer_spec_noise)?,
        layer_ambient_occlusion: get_layer(source, reader, value.layer_a_o)?,
        shift_primary: value.shift_primary,
        shift_secondary: value.shift_secondary,
        color_diffuse: color(value.color_diffuse),
        color_specular: color(value.color_spec),
        specular_exponent_primary: value.spec_exponent0,
        specular_exponent_secondary: value.spec_exponent1,
    })
}
