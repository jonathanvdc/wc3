//! Resource admissibility for the built-in model profile.
use crate::model::error::{Location, ProfileError, ProfileKind, ProfileViolation};
use crate::model::profiles::M2;
use crate::model::{Error, Model};
pub(super) fn resources(resources: &Model<'_, M2>) -> Result<(), Error> {
    use crate::model::{AddressMode, ColorSpace, Sampler};
    for texture in resources.sampled_images() {
        if texture.color_space != ColorSpace::Unspecified {
            return Err(Error::at(
                Location::for_owner(texture.resource).field("color_space"),
                ProfileError::new(
                    ProfileKind::M2,
                    ProfileViolation::ColorSpace {
                        actual: texture.color_space,
                    },
                ),
            ));
        }
        if !matches!(texture.sampler.u, AddressMode::Repeat | AddressMode::Clamp)
            || !matches!(texture.sampler.v, AddressMode::Repeat | AddressMode::Clamp)
        {
            return Err(Error::at(
                Location::for_owner(texture.resource).field("sampler"),
                ProfileError::new(
                    ProfileKind::M2,
                    ProfileViolation::AddressMode {
                        u: texture.sampler.u,
                        v: texture.sampler.v,
                    },
                ),
            ));
        }
    }
    for (id, feature) in resources.features() {
        if let super::ActorKind::Particle(particle) = &feature.payload {
            if let Some((index, image)) = particle
                .details
                .rendering
                .textures
                .iter()
                .enumerate()
                .find(|(_, image)| image.sampler != Sampler::default())
            {
                return Err(Error::at(
                    Location::for_owner(id)
                        .field("details")
                        .field("rendering")
                        .field("textures")
                        .index(index)
                        .field("sampler"),
                    ProfileError::new(
                        ProfileKind::M2,
                        ProfileViolation::ParticleSampler {
                            actual: image.sampler,
                        },
                    ),
                ));
            }
        }
    }
    Ok(())
}
