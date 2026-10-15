//! Native scene wire codes and flags.

/// Native LightKind code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LightKind {
    #[default]
    /// Illuminates along a fixed direction.
    Directional,
    /// Radiates from a position in the model.
    Point,
    /// Unrecognized native code.
    Unknown(u16),
}

/// Native CameraKind code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CameraKind {
    #[default]
    /// Frames the character portrait.
    Portrait,
    /// Frames the character information display.
    CharacterInfo,
    /// Defines a flyby camera.
    Flyby,
    /// Unrecognized native code.
    Unknown(u32),
}

impl LightKind {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Directional,
            1 => Self::Point,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Directional => 0,
            Self::Point => 1,
            Self::Unknown(value) => value,
        }
    }
}

impl CameraKind {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Portrait,
            1 => Self::CharacterInfo,
            4294967295 => Self::Flyby,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Portrait => 0,
            Self::CharacterInfo => 1,
            Self::Flyby => 4294967295,
            Self::Unknown(value) => value,
        }
    }
}
