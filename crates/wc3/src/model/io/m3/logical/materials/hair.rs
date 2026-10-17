//! Direct anisotropic hair shading values.
use super::texture::get_layer;
use super::{color, native, nr, read_error, string, surface, Envelope, Error, Reader};
use crate::model::materials::contributions::SurfaceContribution;
pub(super) fn get_hair<'a, E: Envelope>(
    source: &'a native::Container<'_, E>,
    reader: &mut Reader<'a, '_>,
    reference: native::Ref<nr::targets::HairMaterial>,
    index: usize,
) -> Result<surface::HairMaterialStep, Error> {
    let value = source
        .resolve::<nr::HairMaterial>(reference.cast())
        .map_err(read_error)?
        .get(index)
        .ok_or_else(|| Error::new("m3.material.hair", "missing hair material"))?
        .map_err(read_error)?;
    reader.name = Some(string(source, value.name)?);
    let layer_base = get_layer(source, reader, value.layer_base)?;
    let layer_spec_shift = get_layer(source, reader, value.layer_spec_shift)?;
    let layer_spec_noise = get_layer(source, reader, value.layer_spec_noise)?;
    let layer_ambient_occlusion = get_layer(source, reader, value.layer_a_o)?;
    Ok(surface::HairMaterialStep {
        diffuse: SurfaceContribution {
            input: layer_base,
            parameters: color(value.color_diffuse, reader.builder.values_mut()),
        },
        layer_spec_shift,
        layer_spec_noise,
        layer_ambient_occlusion,
        shift_primary: value.shift_primary,
        shift_secondary: value.shift_secondary,
        color_specular: color(value.color_spec, reader.builder.values_mut()),
        specular_exponent_primary: value.spec_exponent0,
        specular_exponent_secondary: value.spec_exponent1,
    })
}
