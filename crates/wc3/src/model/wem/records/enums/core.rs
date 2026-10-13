//! Named core choices retain unrecognized wire values.
use crate::model::wem;

/// Named ProfileId choices; unrecognized codes remain representable.
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
pub enum ProfileId {
    #[default]
    /// Generic.
    #[wem(value = 0)]
    Generic,
    /// Wc3Classic.
    #[wem(value = 1)]
    Wc3Classic,
    /// Wc3Reforged.
    #[wem(value = 2)]
    Wc3Reforged,
    /// Wow.
    #[wem(value = 3)]
    Wow,
    /// Sc2.
    #[wem(value = 4)]
    Sc2,
    /// Heroes.
    #[wem(value = 5)]
    Heroes,
    /// Diablo3.
    #[wem(value = 6)]
    Diablo3,
    /// Wc3Definitive.
    #[wem(value = 7)]
    Wc3Definitive,
    /// Count.
    #[wem(value = 8)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Game choices; unrecognized codes remain representable.
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
pub enum Game {
    #[default]
    /// Warcraft.
    #[wem(value = 0)]
    Warcraft,
    /// StarCraft.
    #[wem(value = 1)]
    StarCraft,
    /// Wow.
    #[wem(value = 2)]
    Wow,
    /// Diablo.
    #[wem(value = 3)]
    Diablo,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named CoordSpace choices; unrecognized codes remain representable.
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
pub enum CoordSpace {
    #[default]
    /// Blizzard.
    #[wem(value = 0)]
    Blizzard,
    /// Sc2.
    #[wem(value = 1)]
    Sc2,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Handedness choices; unrecognized codes remain representable.
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
pub enum Handedness {
    #[default]
    /// Right.
    #[wem(value = 0)]
    Right,
    /// Left.
    #[wem(value = 1)]
    Left,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named WindingOrder choices; unrecognized codes remain representable.
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
pub enum WindingOrder {
    #[default]
    /// CounterClockwise.
    #[wem(value = 0)]
    CounterClockwise,
    /// Clockwise.
    #[wem(value = 1)]
    Clockwise,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named NativeKind choices; unrecognized codes remain representable.
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
pub enum NativeKind {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Mdx.
    #[wem(value = 1)]
    Mdx,
    /// M2.
    #[wem(value = 2)]
    M2,
    /// M3.
    #[wem(value = 3)]
    M3,
    /// D3.
    #[wem(value = 4)]
    D3,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named AssetHint choices; unrecognized codes remain representable.
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
pub enum AssetHint {
    #[default]
    /// The format's unspecified choice.
    #[wem(value = 0)]
    Unspecified,
    /// Appearance.
    #[wem(value = 1)]
    Appearance,
    /// Actor.
    #[wem(value = 2)]
    Actor,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named AxisPreset choices; unrecognized codes remain representable.
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
pub enum AxisPreset {
    #[default]
    /// YUp.
    #[wem(value = 0)]
    YUp,
    /// ZUpMax.
    #[wem(value = 1)]
    ZUpMax,
    /// Native.
    #[wem(value = 2)]
    Native,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named LengthUnit choices; unrecognized codes remain representable.
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
pub enum LengthUnit {
    #[default]
    /// Millimetre.
    #[wem(value = 0)]
    Millimetre,
    /// Centimetre.
    #[wem(value = 1)]
    Centimetre,
    /// Metre.
    #[wem(value = 2)]
    Metre,
    /// Inch.
    #[wem(value = 3)]
    Inch,
    /// Foot.
    #[wem(value = 4)]
    Foot,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}
