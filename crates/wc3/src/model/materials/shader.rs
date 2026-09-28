//! Named shader pipelines; other raw shader IDs remain valid binary data.

/// The four shader IDs with names in the Warcraft III registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ShaderType {
    SdLegacy = 0,
    HdDefaultUnit = 1,
    SdFixedFunction = 2,
    HdCrystal = 24,
}

impl ShaderType {
    /// Returns a named pipeline, leaving unnamed raw IDs unclassified.
    pub const fn from_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Self::SdLegacy),
            1 => Some(Self::HdDefaultUnit),
            2 => Some(Self::SdFixedFunction),
            24 => Some(Self::HdCrystal),
            _ => None,
        }
    }

    pub const fn id(self) -> u32 {
        self as u32
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::SdLegacy => "Shader_SD_Legacy",
            Self::HdDefaultUnit => "Shader_HD_DefaultUnit",
            Self::SdFixedFunction => "Shader_SD_FixedFunction",
            Self::HdCrystal => "Shader_HD_Crystal",
        }
    }

    /// Matches registry names case-insensitively. Unknown names are not
    /// silently downgraded to SD as they are by the game client's text reader.
    pub fn from_name(name: &str) -> Option<Self> {
        [
            Self::SdLegacy,
            Self::HdDefaultUnit,
            Self::SdFixedFunction,
            Self::HdCrystal,
        ]
        .into_iter()
        .find(|shader| name.eq_ignore_ascii_case(shader.name()))
    }
}
