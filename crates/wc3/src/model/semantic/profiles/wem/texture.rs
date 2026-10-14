//! Texture roles and alternative resource declarations belong to actual image objects.
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{Error, Link, Resources, ValidateLinks};
use bitfield::bitfield;
use std::borrow::Cow;

/// Declared image role independent of individual material input usage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextureSlot {
    #[default]
    /// Diffuse.
    Diffuse,
    /// Normal.
    Normal,
    /// Specular.
    Specular,
    /// Gloss.
    Gloss,
    /// Emissive.
    Emissive,
    /// Environment.
    Environment,
    /// AmbientOcclusion.
    AmbientOcclusion,
    /// Height.
    Height,
    /// Lightmap.
    Lightmap,
    /// Detail.
    Detail,
    /// Count.
    Count,
    /// Unrecognized authored texture role.
    Unknown(u8),
}

bitfield! {
    /// Image options outside the common sampler's U and V addressing authority.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct SupplementalTextureFlags(u32);
    /// Retained supplemental option bits.
    pub bits,_:31,0;
}
impl SupplementalTextureFlags {
    /// Retains unnamed options while excluding the common sampler's wrap selections.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits & !3)
    }
}
/// Additional WEM image declarations alongside the common resource identity.
#[derive(Clone, Debug, Default)]
pub struct TextureControls<'a> {
    /// Separately authored path spelling, including a path paired with a numeric resource key.
    pub path: Option<Cow<'a, [u8]>>,
    /// Independently declared replacement-image category; absence means no category.
    pub replaceable: Option<u32>,
    /// Declared image role.
    pub slot: TextureSlot,
    /// Options not represented by common sampler addressing.
    pub flags: SupplementalTextureFlags,
}
impl ValidateLinks for TextureControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self.flags.bits() & 3 != 0
            || self.replaceable == Some(0)
            || self.path.as_ref().is_some_and(|path| path.is_empty())
        {
            visitor(Link::InvalidValue);
        }
    }
}

pub(super) fn resources(resources: &Resources<'_, Wem>) -> Result<(), Error> {
    use crate::model::semantic::{AddressMode, Resource};
    for (_, texture) in resources.textures() {
        if !matches!(
            texture.resource,
            Resource::Unspecified
                | Resource::Path(_)
                | Resource::FileDataId(_)
                | Resource::Sno { .. }
        ) {
            return Err(Error::new(
                "wem.texture.resource",
                "texture keys support an absent identity, filename, file-data key, or SNO identity",
            ));
        }
        if !matches!(texture.sampler.u, AddressMode::Repeat | AddressMode::Clamp)
            || !matches!(texture.sampler.v, AddressMode::Repeat | AddressMode::Clamp)
        {
            return Err(Error::new(
                "wem.texture.sampler",
                "image flags support repeat or clamp addressing",
            ));
        }
    }
    Ok(())
}
