//! Native diablo selectors and flags preserve their encoded widths.
use crate::model::wem;
use bitfield::bitfield;

bitfield! {
    /// Native D3MaterialFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct D3MaterialFlags(i32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
}
impl D3MaterialFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: i32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<i32> for D3MaterialFlags {
    fn from(bits: i32) -> Self {
        Self(bits)
    }
}
impl From<D3MaterialFlags> for i32 {
    fn from(flags: D3MaterialFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native D3UvFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct D3UvFlags(i32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
}
impl D3UvFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: i32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<i32> for D3UvFlags {
    fn from(bits: i32) -> Self {
        Self(bits)
    }
}
impl From<D3UvFlags> for i32 {
    fn from(flags: D3UvFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native D3PassFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct D3PassFlags(i32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
}
impl D3PassFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: i32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<i32> for D3PassFlags {
    fn from(bits: i32) -> Self {
        Self(bits)
    }
}
impl From<D3PassFlags> for i32 {
    fn from(flags: D3PassFlags) -> Self {
        flags.bits()
    }
}
