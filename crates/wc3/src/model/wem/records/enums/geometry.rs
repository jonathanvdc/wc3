//! Named geometry choices and flags retain unrecognized wire values.
use crate::model::wem;
use bitfield::bitfield;

/// Named IndexWidth choices; unrecognized codes remain representable.
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
#[wem(value = u8)]
pub enum IndexWidth {
    #[default]
    /// U16.
    #[wem(value = 0)]
    U16,
    /// U32.
    #[wem(value = 1)]
    U32,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named VertexWarpProperty choices; unrecognized codes remain representable.
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
#[wem(value = u32)]
pub enum VertexWarpProperty {
    #[default]
    /// Radius.
    #[wem(value = 0)]
    Radius,
    /// Height.
    #[wem(value = 1)]
    Height,
    /// Strength.
    #[wem(value = 2)]
    Strength,
    /// Angular.
    #[wem(value = 3)]
    Angular,
    /// Axial.
    #[wem(value = 4)]
    Axial,
    /// Radial.
    #[wem(value = 5)]
    Radial,
    /// Count.
    #[wem(value = 6)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named AttrType choices; unrecognized codes remain representable.
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
#[wem(value = u8)]
pub enum AttrType {
    #[default]
    /// F32.
    #[wem(value = 0)]
    F32,
    /// F32x2.
    #[wem(value = 1)]
    F32x2,
    /// F32x3.
    #[wem(value = 2)]
    F32x3,
    /// F32x4.
    #[wem(value = 3)]
    F32x4,
    /// U8x4.
    #[wem(value = 4)]
    U8x4,
    /// U16.
    #[wem(value = 5)]
    U16,
    /// U32.
    #[wem(value = 6)]
    U32,
    /// I32.
    #[wem(value = 7)]
    I32,
    /// Quat.
    #[wem(value = 8)]
    Quat,
    /// Bool.
    #[wem(value = 9)]
    Bool,
    /// U32x4.
    #[wem(value = 10)]
    U32x4,
    /// Count.
    #[wem(value = 11)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

bitfield! {
    /// SectionFlags bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct SectionFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the Hidden bit.
    pub hidden, set_hidden: 0;
    /// Returns or changes the ClothSimulated bit.
    pub cloth_simulated, set_cloth_simulated: 1;
    /// Returns or changes the ClothInfluenced bit.
    pub cloth_influenced, set_cloth_influenced: 2;
    /// Returns or changes the Billboard bit.
    pub billboard, set_billboard: 3;
    /// Returns or changes the ProjectedShadow bit.
    pub projected_shadow, set_projected_shadow: 4;
}

impl SectionFlags {
    /// Retains every stored bit, including flags without known names.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    /// Bit mask for None.
    pub const NONE: Self = Self(0);
    /// Bit mask for Hidden.
    pub const HIDDEN: Self = Self(1);
    /// Bit mask for ClothSimulated.
    pub const CLOTH_SIMULATED: Self = Self(2);
    /// Bit mask for ClothInfluenced.
    pub const CLOTH_INFLUENCED: Self = Self(4);
    /// Bit mask for Billboard.
    pub const BILLBOARD: Self = Self(8);
    /// Bit mask for ProjectedShadow.
    pub const PROJECTED_SHADOW: Self = Self(16);
}
impl From<u32> for SectionFlags {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<SectionFlags> for u32 {
    fn from(value: SectionFlags) -> Self {
        value.bits()
    }
}

/// Named NormalPacking choices; unrecognized codes remain representable.
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
#[wem(value = u8)]
pub enum NormalPacking {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// XInAlpha.
    #[wem(value = 1)]
    XInAlpha,
    /// TwoChannel.
    #[wem(value = 2)]
    TwoChannel,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named AttributeClass choices; unrecognized codes remain representable.
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
#[wem(value = u32)]
pub enum AttributeClass {
    #[default]
    /// Position.
    #[wem(value = 0)]
    Position,
    /// Normal.
    #[wem(value = 1)]
    Normal,
    /// Tangent.
    #[wem(value = 2)]
    Tangent,
    /// UV.
    #[wem(value = 3)]
    UV,
    /// Color.
    #[wem(value = 4)]
    Color,
    /// BlendIndices.
    #[wem(value = 5)]
    BlendIndices,
    /// BlendWeights.
    #[wem(value = 6)]
    BlendWeights,
    /// Binormal.
    #[wem(value = 7)]
    Binormal,
    /// Count.
    #[wem(value = 8)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named AttributeEncoding choices; unrecognized codes remain representable.
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
#[wem(value = u32)]
pub enum AttributeEncoding {
    #[default]
    /// Float32.
    #[wem(value = 0)]
    Float32,
    /// Float16.
    #[wem(value = 1)]
    Float16,
    /// SNorm8.
    #[wem(value = 2)]
    SNorm8,
    /// SNorm16.
    #[wem(value = 3)]
    SNorm16,
    /// UNorm8.
    #[wem(value = 4)]
    UNorm8,
    /// UNorm16.
    #[wem(value = 5)]
    UNorm16,
    /// UInt8.
    #[wem(value = 6)]
    UInt8,
    /// UInt16.
    #[wem(value = 7)]
    UInt16,
    /// UInt32.
    #[wem(value = 8)]
    UInt32,
    /// Int8.
    #[wem(value = 9)]
    Int8,
    /// Int16.
    #[wem(value = 10)]
    Int16,
    /// Int32.
    #[wem(value = 11)]
    Int32,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}
