//! Layer shader pipelines, including unnamed IDs retained from binary files.
use crate::model::mdx;

/// A layer shader ID, preserving all u32 values, including unnamed pipelines.
///
/// Known names are available through [`Self::name`]. Unknown IDs have no engine
/// shader-name spelling; HiveWorkshop MDL stores them numerically. All IDs
/// round-trip exactly through MDX.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, mdx::Read, mdx::Write)]
pub struct ShaderType(u32);

impl ShaderType {
    pub const SD_LEGACY: Self = Self(0);
    pub const HD_DEFAULT_UNIT: Self = Self(1);
    pub const SD_FIXED_FUNCTION: Self = Self(2);
    pub const HD_CRYSTAL: Self = Self(24);

    /// Wraps a raw ID without restricting it to known pipelines.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the exact stored binary ID.
    pub const fn id(self) -> u32 {
        self.0
    }

    /// Returns the registry name, or None for an unnamed ID.
    pub const fn name(self) -> Option<&'static str> {
        match self.0 {
            0 => Some("Shader_SD_Legacy"),
            1 => Some("Shader_HD_DefaultUnit"),
            2 => Some("Shader_SD_FixedFunction"),
            24 => Some("Shader_HD_Crystal"),
            _ => None,
        }
    }

    /// Matches registry names case-insensitively without silently downgrading
    /// unknown names to SD as the game client's text reader does.
    pub fn from_name(name: &str) -> Option<Self> {
        [
            Self::SD_LEGACY,
            Self::HD_DEFAULT_UNIT,
            Self::SD_FIXED_FUNCTION,
            Self::HD_CRYSTAL,
        ]
        .into_iter()
        .find(|shader| name.eq_ignore_ascii_case(shader.name().expect("named pipeline")))
    }
}
