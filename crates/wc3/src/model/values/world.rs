//! Independent option words retain unresolved bits without assigning new meanings.
use bitfield::bitfield;
bitfield! {
    /// Unresolved group flags2 retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct GroupFlags2(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl GroupFlags2 {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for GroupFlags2 {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved doodad flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct DoodadFlags(u8);
    /// Complete retained option word.
    pub bits, _:7,0;
}
impl DoodadFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u8) -> Self {
        Self(bits)
    }
}
impl From<u8> for DoodadFlags {
    fn from(bits: u8) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved light flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct LightFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl LightFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for LightFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved fog flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct FogFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl FogFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for FogFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved ambient flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct AmbientFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl AmbientFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for AmbientFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
