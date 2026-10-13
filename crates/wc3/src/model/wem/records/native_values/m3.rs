//! Native m3 selectors and flags preserve their encoded widths.
use crate::model::wem;
use bitfield::bitfield;

/// Interpolation operation stored in a native M3 animation header.
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
#[wem(value=u16)]
pub enum NativeInterpolation16 {
    #[default]
    /// Constant.
    #[wem(value = 0)]
    Constant,
    /// Linear.
    #[wem(value = 1)]
    Linear,
    /// Unrecognized wire code.
    #[wem(unknown)]
    Unknown(u16),
}

bitfield! {
    /// Native M3AnimationFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M3AnimationFlags(u16);
    /// Returns the exact stored bits.
    pub bits, _: 15, 0;
}
impl M3AnimationFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u16> for M3AnimationFlags {
    fn from(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<M3AnimationFlags> for u16 {
    fn from(flags: M3AnimationFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native M3AlphaFresnelFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M3AlphaFresnelFlags(u8);
    /// Returns the exact stored bits.
    pub bits, _: 7, 0;
}
impl M3AlphaFresnelFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u8) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u8> for M3AlphaFresnelFlags {
    fn from(bits: u8) -> Self {
        Self(bits)
    }
}
impl From<M3AlphaFresnelFlags> for u8 {
    fn from(flags: M3AlphaFresnelFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native M3DisplacementFlags control bits; unresolved bits remain intact.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M3DisplacementFlags(u32);
    /// Returns the exact stored flag word.
    pub bits, _: 31, 0;
}
impl M3DisplacementFlags {
    /// Preserves every bit of the flag word.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for M3DisplacementFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<M3DisplacementFlags> for u32 {
    fn from(flags: M3DisplacementFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native M3VolumeFlags control bits; unresolved bits remain intact.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M3VolumeFlags(u32);
    /// Returns the exact stored flag word.
    pub bits, _: 31, 0;
}
impl M3VolumeFlags {
    /// Preserves every bit of the flag word.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for M3VolumeFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<M3VolumeFlags> for u32 {
    fn from(flags: M3VolumeFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native RttChannels mask, including unnamed channels.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct RttChannels(u32);
    /// Returns the complete stored mask.
    pub bits, _: 31, 0;
    /// Uses render-to-texture channel 0.
    pub channel_0, set_channel_0: 0;
    /// Uses render-to-texture channel 1.
    pub channel_1, set_channel_1: 1;
    /// Uses render-to-texture channel 2.
    pub channel_2, set_channel_2: 2;
    /// Uses render-to-texture channel 3.
    pub channel_3, set_channel_3: 3;
    /// Uses render-to-texture channel 4.
    pub channel_4, set_channel_4: 4;
    /// Uses render-to-texture channel 5.
    pub channel_5, set_channel_5: 5;
    /// Uses render-to-texture channel 6.
    pub channel_6, set_channel_6: 6;
}
impl RttChannels {
    /// Preserves the entire stored mask.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for RttChannels {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<RttChannels> for u32 {
    fn from(flags: RttChannels) -> Self {
        flags.bits()
    }
}
