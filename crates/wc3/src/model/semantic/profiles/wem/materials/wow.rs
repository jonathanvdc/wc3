//! Native WoW shader selectors reference producer-owned source tables, not WEM resource ordinals.
use super::shader_values::{WowBatchFlags, WowBlend, WowRenderFlags, WowTextureSource};
use crate::model::semantic::{Link, ValidateLinks};
/// Identifies a texture in the native source material's texture-selection table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WowTextureSelection(pub u16);
/// Identifies a coordinate transform in the native source material's transform table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WowTransformSelection(pub u16);
/// Identifies opacity in the native source material's texture-weight table.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WowWeightSelection(pub u16);
/// Identifies color in the native source material's color table; negative values retain producer choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WowColorSelection(pub i16);
/// Authored shader input with unresolved external native table selections.
#[derive(Clone, Debug, Default)]
pub struct WowTextureUnit {
    /// Native texture-table selection.
    pub texture: WowTextureSelection,
    /// Native coordinate set selection.
    pub coordinate_set: u16,
    /// Native texture-transform selection.
    pub transform: WowTransformSelection,
    /// Native texture-weight selection.
    pub weight: WowWeightSelection,
    /// Runtime image source or replacement mechanism.
    pub source: WowTextureSource,
    /// Replacement-image category interpreted by the game.
    pub replaceable: u8,
}
/// Independently authored native WoW shader pass retained alongside the common shader.
#[derive(Clone, Debug, Default)]
pub struct WowMaterial {
    /// Draw behavior for the native pass.
    pub flags: WowRenderFlags,
    /// Native color compositing operation.
    pub blend: WowBlend,
    /// Shader permutation identity.
    pub shader: u16,
    /// Draw-order priority plane.
    pub priority_plane: i16,
    /// Layer order within the native geometry section.
    pub layer: u16,
    /// External native color selection.
    pub color: WowColorSelection,
    /// Native batch behavior.
    pub batch_flags: WowBatchFlags,
    /// Ordered native shader inputs.
    pub textures: Vec<WowTextureUnit>,
}
impl ValidateLinks for WowMaterial {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}

impl WowMaterial {
    /// Checks native selectors, which do not imply document-local references.
    pub fn validate(&self) -> Result<(), crate::model::semantic::Error> {
        Ok(())
    }
}
