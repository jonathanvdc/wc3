//! Texture sampling and independent producer options have separate authorities.
use crate::model::semantic::{Link, ValidateLinks};
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
