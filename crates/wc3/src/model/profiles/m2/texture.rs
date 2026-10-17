//! Resource admissibility for the built-in model profile.
use crate::model::profiles::M2;
use crate::model::{Error, Model};
pub(super) fn resources(resources: &Model<'_, M2>) -> Result<(), Error> {
    use crate::model::{AddressMode, ColorSpace, M2ImageResource, Sampler};
    for texture in resources.sampled_images() {
        let controls = texture.m2_controls()?;
        if let Some(path) = controls.path {
            if !matches!(resources.image_resource(path), Some(M2ImageResource::Path(bytes)) if !bytes.is_empty() && !bytes.contains(&0))
            {
                return Err(Error::new(
                    "m2.texture.path",
                    "alternate filenames require nonempty path dependencies without NUL",
                ));
            }
        }
        if !matches!(
            resources
                .image_resource(texture.resource)
                .expect("validated image"),
            M2ImageResource::Path(_)
                | M2ImageResource::FileDataId(1..=u32::MAX)
                | M2ImageResource::Replaceable(1..=u32::MAX)
        ) {
            return Err(Error::new(
                "m2.texture.resource",
                "textures require a filename, nonzero file-data key, or nonzero replacement category",
            ));
        }
        if texture.m2_controls()?.path.is_some()
            && matches!(
                resources
                    .image_resource(texture.resource)
                    .expect("validated image"),
                M2ImageResource::Path(_)
            )
        {
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
