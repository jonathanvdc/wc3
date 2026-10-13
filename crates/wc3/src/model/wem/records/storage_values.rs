//! Envelope and reference bit words whose individual meanings remain unestablished.
use crate::model::{common::Ownership, wem};
use bitfield::bitfield;
bitfield! {
    /// Native reference metadata; unknown bits survive reference edits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct ReferenceFlags(u32);
    /// Returns the complete stored word.
    pub bits, _: 31, 0;
}
impl From<u32> for ReferenceFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ReferenceFlags> for u32 {
    fn from(value: ReferenceFlags) -> Self {
        value.bits()
    }
}
bitfield! {
    /// Envelope profile availability mask; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct ProfileMask(u32);
    /// Returns the complete stored word.
    pub bits, _: 31, 0;
}
impl From<u32> for ProfileMask {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ProfileMask> for u32 {
    fn from(value: ProfileMask) -> Self {
        value.bits()
    }
}
bitfield! {
    /// Native Diablo material variant bits, including unassigned producer controls.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct D3VariantFlags(i32);
    /// Returns the complete signed wire word.
    pub bits, _: 31, 0;
}
impl From<i32> for D3VariantFlags {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl From<D3VariantFlags> for i32 {
    fn from(value: D3VariantFlags) -> Self {
        value.bits()
    }
}
bitfield! {
    /// WoW collision-shape callback flags, including unassigned controls.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct WowShapeGameFlags(u16);
    /// Returns the complete native word.
    pub bits, _: 15, 0;
}

impl From<u16> for WowShapeGameFlags {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<WowShapeGameFlags> for u16 {
    fn from(value: WowShapeGameFlags) -> Self {
        value.bits()
    }
}

impl ReferenceFlags {
    /// Keeps every native bit, including controls without known meanings.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no controls are enabled.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl ProfileMask {
    /// Keeps every native bit, including controls without known meanings.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no controls are enabled.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl D3VariantFlags {
    /// Keeps every native bit, including controls without known meanings.
    pub const fn from_bits_retain(bits: i32) -> Self {
        Self(bits)
    }
    /// Whether no controls are enabled.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

impl WowShapeGameFlags {
    /// Keeps every native bit, including controls without known meanings.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
    /// Whether no controls are enabled.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
