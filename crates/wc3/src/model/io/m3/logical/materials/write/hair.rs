//! Anisotropic hair encoding from actual material controls.
use super::{native_color, nr, surface, Envelope, Error, Writer};
pub(super) fn encode<E: Envelope>(
    name: &str,
    r: &surface::HairMaterialStep,
    writer: &mut Writer<'_, '_, '_, '_, E>,
) -> Result<nr::HairMaterial, Error> {
    Ok(nr::HairMaterial {
        name: writer.string(name)?,
        layer_base: writer.layer(r.diffuse.input.as_ref())?,
        layer_spec_shift: writer.layer(r.layer_spec_shift.as_ref())?,
        layer_spec_noise: writer.layer(r.layer_spec_noise.as_ref())?,
        layer_a_o: writer.layer(r.layer_ambient_occlusion.as_ref())?,
        shift_primary: r.shift_primary,
        shift_secondary: r.shift_secondary,
        color_diffuse: native_color(&r.diffuse.parameters, writer)?,
        color_spec: native_color(&r.color_specular, writer)?,
        spec_exponent0: r.specular_exponent_primary,
        spec_exponent1: r.specular_exponent_secondary,
    })
}
