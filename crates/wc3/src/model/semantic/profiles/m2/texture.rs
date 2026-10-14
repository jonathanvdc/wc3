//! Texture sampling and independent producer options have separate authorities.
use crate::model::semantic::profiles::M2;
use crate::model::semantic::{Error, Link, Resources, ValidateLinks};
use bitfield::bitfield;
use std::borrow::Cow;

bitfield! {
    /// Producer options outside the common sampler's U and V address selections.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct SupplementalTextureFlags(u32);
    /// Retained producer option bits.
    pub bits,_:31,0;
}
impl SupplementalTextureFlags {
    /// Retains supplemental options, excluding the common address-mode bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits & !3)
    }
}
/// Owner-local options of a World of Warcraft texture definition.
#[derive(Clone, Debug, Default)]
pub struct TextureControls<'a> {
    /// Separately declared filename paired with a numeric or replacement identity.
    pub path: Option<Cow<'a, [u8]>>,
    /// Producer options independent of address modes.
    pub flags: SupplementalTextureFlags,
}
impl ValidateLinks for TextureControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.flags.bits() & 3 != 0 || self.path.as_ref().is_some_and(|path| path.is_empty()) {
            visitor(Link::InvalidValue);
        }
    }
}

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
