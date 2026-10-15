//! Authoring controls owned by an individual world-surface draw.
use super::BatchFlags;
use crate::model::semantic::{Bounds, Link, TextureId, ValidateLinks};
/// Illumination class of a world-surface draw.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lighting {
    /// Blend interior and exterior illumination through the transition region.
    Transition,
    /// Use interior illumination.
    Interior,
    /// Use exterior illumination.
    #[default]
    Exterior,
    /// Authored illumination class whose behavior is unresolved.
    Unknown(u32),
}
/// Purpose of an independently authored world-surface pass.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Pass {
    /// Color and illumination pass.
    #[default]
    Surface,
    /// Shadow pass with unresolved supplemental authored controls.
    Shadow {
        /// Five prefix controls with no established meanings.
        controls: [u16; 5],
        /// Two additional controls with no established meanings.
        supplemental: [u16; 2],
    },
}
/// Complete world-surface draw controls independent of batch record revisions.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DrawControls {
    /// Purpose of this draw.
    pub pass: Pass,
    /// Illumination class for this draw.
    pub lighting: Lighting,
    /// Authored culling bounds when explicitly provided.
    pub bounds: Option<Bounds>,
    /// Optional baked-light texture resource.
    pub lightmap: Option<TextureId>,
    /// Historical baked-light selector whose table interpretation remains unresolved.
    pub lightmap_selector: Option<u8>,
    /// Batch behavior bits whose meanings remain unresolved.
    pub unclassified: BatchFlags,
    /// Individual unresolved supplemental controls surrounding the draw selection.
    pub supplemental: [u16; 5],
    /// Historical alignment control whose meaning remains unresolved.
    pub alignment: u8,
}
impl ValidateLinks for DrawControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.lightmap.visit_links(visitor);
        if self
            .bounds
            .as_ref()
            .is_some_and(|bounds| bounds.validate().is_err())
        {
            visitor(Link::InvalidValue);
        }
    }
}
