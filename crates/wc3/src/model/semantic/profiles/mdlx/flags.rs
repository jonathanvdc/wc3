//! Independent option words retain unresolved bits without assigning new meanings.
use bitfield::bitfield;
bitfield! {
    /// Unresolved texture flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct TextureFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl TextureFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for TextureFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved sequence flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct SequenceFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl SequenceFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for SequenceFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved node flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct NodeFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl NodeFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for NodeFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved material flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct MaterialFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl MaterialFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for MaterialFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved layer flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct LayerFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl LayerFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for LayerFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved geometry animation flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct GeometryAnimationFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl GeometryAnimationFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for GeometryAnimationFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved particle flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct ParticleFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl ParticleFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for ParticleFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved particle2 flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct Particle2Flags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl Particle2Flags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for Particle2Flags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
bitfield! {
    /// Unresolved popcorn flags retained alongside named authoring controls.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct PopcornFlags(u32);
    /// Complete retained option word.
    pub bits, _:31,0;
}
impl PopcornFlags {
    /// Constructs an option word without discarding any unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for PopcornFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
