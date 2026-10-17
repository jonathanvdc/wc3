//! Texture sampling and independent producer options have separate authorities.
use crate::model::ImageResourceId;
use crate::model::{Link, ValidateLinks};
use bitfield::bitfield;

bitfield! {
    /// Producer options outside the common sampler's U and V address selections.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq,PartialOrd,Ord,Hash)]
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextureControls {
    /// Separately declared filename paired with a numeric or replacement identity.
    pub path: Option<ImageResourceId>,
    /// Producer options independent of address modes.
    pub flags: SupplementalTextureFlags,
}
impl ValidateLinks for TextureControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.path.visit_links(visitor);
        if self.flags.bits() & 3 != 0 {
            visitor(Link::InvalidValue);
        }
    }
}
