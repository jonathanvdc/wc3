//! Documented WMO scalar codes with lossless unknown values.
use crate::model::{common::Ownership, formats::wmo};

/// Native blendmode code.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Value,
    wmo::Validate,
)]
#[wmo(value = u32)]
pub enum BlendMode {
    /// Native opaque behavior.
    #[default]
    #[wmo(value = 0)]
    Opaque,
    /// Native alphakey behavior.
    #[wmo(value = 1)]
    AlphaKey,
    /// Native alpha behavior.
    #[wmo(value = 2)]
    Alpha,
    /// Native add behavior.
    #[wmo(value = 3)]
    Add,
    /// Native modulate behavior.
    #[wmo(value = 4)]
    Modulate,
    /// Native modulate2x behavior.
    #[wmo(value = 5)]
    Modulate2x,
    /// Native modulateadd behavior.
    #[wmo(value = 6)]
    ModulateAdd,
    /// Native inversesourcealphaadd behavior.
    #[wmo(value = 7)]
    InverseSourceAlphaAdd,
    /// A native code with no established interpretation.
    #[wmo(unknown)]
    Unknown(u32),
}

/// Native lightkind code.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Value,
    wmo::Validate,
)]
#[wmo(value = u8)]
pub enum LightKind {
    /// Native omni behavior.
    #[default]
    #[wmo(value = 0)]
    Omni,
    /// Native spot behavior.
    #[wmo(value = 1)]
    Spot,
    /// Native directional behavior.
    #[wmo(value = 2)]
    Directional,
    /// Native ambient behavior.
    #[wmo(value = 3)]
    Ambient,
    /// A native code with no established interpretation.
    #[wmo(unknown)]
    Unknown(u8),
}

/// Native newlightkind code.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Value,
    wmo::Validate,
)]
#[wmo(value = i32)]
pub enum NewLightKind {
    /// Native point behavior.
    #[default]
    #[wmo(value = 0)]
    Point,
    /// Native spot behavior.
    #[wmo(value = 1)]
    Spot,
    /// A native code with no established interpretation.
    #[wmo(unknown)]
    Unknown(i32),
}

/// Native variation function for animated light intensity.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Value,
    wmo::Validate,
)]
#[wmo(value=i32)]
pub enum FlickerMode {
    /// Constant authored intensity.
    #[default]
    #[wmo(value = 0)]
    Off,
    /// Sinusoidal variation.
    #[wmo(value = 1)]
    Sine,
    /// Smooth noise variation.
    #[wmo(value = 2)]
    NoiseCurve,
    /// Stepped noise variation.
    #[wmo(value = 3)]
    NoiseStep,
    /// A code without an established interpretation.
    #[wmo(unknown)]
    Unknown(i32),
}

/// Coordinate component selected by the low two BSP-node flag bits.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ownership,
    wmo::Read,
    wmo::Write,
    wmo::RecordLayout,
    wmo::Value,
    wmo::Validate,
)]
#[wmo(value=u16)]
pub enum BspAxis {
    /// Split perpendicular to the X coordinate.
    #[default]
    #[wmo(value = 0)]
    X,
    /// Split perpendicular to the Y coordinate.
    #[wmo(value = 1)]
    Y,
    /// Split perpendicular to the Z coordinate.
    #[wmo(value = 2)]
    Z,
    /// An unrecognized encoded axis choice.
    #[wmo(unknown)]
    Unknown(u16),
}
