//! Native scene wire codes and flags.
use super::*;

/// Native LightKind code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u16)]
pub enum LightKind {
    #[default]
    /// Illuminates along a fixed direction.
    #[m2(value = 0)]
    Directional,
    /// Radiates from a position in the model.
    #[m2(value = 1)]
    Point,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u16),
}

/// Native CameraKind code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u32)]
pub enum CameraKind {
    #[default]
    /// Frames the character portrait.
    #[m2(value = 0)]
    Portrait,
    /// Frames the character information display.
    #[m2(value = 1)]
    CharacterInfo,
    /// Defines a flyby camera.
    #[m2(value = 4294967295)]
    Flyby,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u32),
}
wire_ule!(LightKind, u16, raw, from_raw);
wire_ule!(CameraKind, u32, raw, from_raw);
