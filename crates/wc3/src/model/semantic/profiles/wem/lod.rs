//! Simplification requests describe future level-of-detail preparation.
use crate::model::semantic::{Link, ValidateLinks};
/// Authored simplification request independent of existing mesh LOD membership.
#[derive(Clone, Debug, Default)]
pub struct LodPreparation {
    /// Requests generation of three additional levels.
    pub generate: bool,
    /// Requested triangle-count ratios relative to level zero.
    pub triangle_ratios: [f32; 3],
    /// Preserves open boundaries while simplifying.
    pub lock_borders: bool,
    /// Records that the original authoring input contained additional levels.
    pub source_had_levels: bool,
}
impl ValidateLinks for LodPreparation {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if self
            .triangle_ratios
            .iter()
            .any(|value| !value.is_finite() || *value < 0.)
        {
            visitor(Link::InvalidValue);
        }
    }
}
