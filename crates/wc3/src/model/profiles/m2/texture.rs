//! Resource admissibility for the built-in model profile.
use crate::model::profiles::M2;
use crate::model::{Error, Model};
pub(super) fn resources(resources: &Model<'_, M2>) -> Result<(), Error> {
    use crate::model::{AddressMode, ColorSpace, Sampler};
    for texture in resources.sampled_images() {
        if texture.color_space != ColorSpace::Unspecified {
            return Err(Error::new(
                "m2.texture.color_space",
                "texture definitions do not declare a color transfer function",
            ));
        }
        if !matches!(texture.sampler.u, AddressMode::Repeat | AddressMode::Clamp)
            || !matches!(texture.sampler.v, AddressMode::Repeat | AddressMode::Clamp)
        {
            return Err(Error::new(
                "m2.texture.sampler",
                "texture definitions support repeat or clamp addressing",
            ));
        }
    }
    for (_, feature) in resources.features() {
        if let super::ActorKind::Particle(particle) = &feature.payload {
            if particle
                .details
                .rendering
                .textures
                .iter()
                .any(|image| image.sampler != Sampler::default())
            {
                return Err(Error::new(
                    "m2.particle.sampler",
                    "native age-driven particles repeat both texture axes",
                ));
            }
        }
    }
    Ok(())
}
