//! Level-of-detail membership belongs to its actual mesh owner.
use crate::model::semantic::{Link, ValidateLinks};
/// Editable-model geometry-selection controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MeshControls {
    /// Game-defined level-of-detail identity; absence shares the mesh across levels.
    pub level_of_detail: Option<u32>,
}
impl ValidateLinks for MeshControls {
    fn visit_links(&self, _visitor: &mut dyn FnMut(Link)) {}
}
