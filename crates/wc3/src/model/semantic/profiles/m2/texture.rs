//! Resource admissibility for the built-in model profile.
use crate::model::semantic::profiles::M2;
use crate::model::semantic::{Error, Resources};
pub(super) fn resources(resources: &Resources<'_, M2>) -> Result<(), Error> {
    use crate::model::semantic::{AddressMode, ColorSpace, Resource};
    for (_, texture) in resources.textures() {
        if !matches!(
            texture.resource,
            Resource::Path(_)
                | Resource::FileDataId(1..=u32::MAX)
                | Resource::Replaceable(1..=u32::MAX)
        ) {
            return Err(Error::new(
                "m2.texture.resource",
                "textures require a filename, nonzero file-data key, or nonzero replacement category",
            ));
        }
        if texture.controls.path.is_some() && matches!(texture.resource, Resource::Path(_)) {
            return Err(Error::new(
                "m2.texture.path",
                "the primary filename and alternate filename cannot both own the path declaration",
            ));
        }
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
    Ok(())
}
