//! Native material selectors and flags preserve their encoded widths.
use crate::model::wem;
use bitfield::bitfield;

/// Native TextureSlot choices, retaining unrecognized codes.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value=u8)]
pub enum TextureSlot {
    #[default]
    /// Diffuse.
    #[wem(value = 0)]
    Diffuse,
    /// Normal.
    #[wem(value = 1)]
    Normal,
    /// Specular.
    #[wem(value = 2)]
    Specular,
    /// Gloss.
    #[wem(value = 3)]
    Gloss,
    /// Emissive.
    #[wem(value = 4)]
    Emissive,
    /// Environment.
    #[wem(value = 5)]
    Environment,
    /// AmbientOcclusion.
    #[wem(value = 6)]
    AmbientOcclusion,
    /// Height.
    #[wem(value = 7)]
    Height,
    /// Lightmap.
    #[wem(value = 8)]
    Lightmap,
    /// Detail.
    #[wem(value = 9)]
    Detail,
    /// Count.
    #[wem(value = 10)]
    Count,
    /// Unrecognized wire code.
    #[wem(unknown)]
    Unknown(u8),
}

bitfield! {
    /// Native TextureFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct TextureFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes WrapU.
    pub wrap_u, set_wrap_u: 0;
    /// Returns or changes WrapV.
    pub wrap_v, set_wrap_v: 1;
}
impl TextureFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u32> for TextureFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<TextureFlags> for u32 {
    fn from(flags: TextureFlags) -> Self {
        flags.bits()
    }
}
