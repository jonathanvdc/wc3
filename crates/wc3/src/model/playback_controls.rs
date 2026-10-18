//! Authoring controls belong to their logical playback and controller owners.
use crate::model::controls::{layered, model};
use crate::model::{Link, ValidateLinks};
/// Native authoring behavior supplementing one logical clip.
#[derive(Clone, Debug)]
pub enum ClipControls {
    /// Warcraft III sequence selection and authored interval controls.
    Mdlx(layered::Sequence),
    /// World of Warcraft sequence selection and transition controls.
    M2(model::SequenceControls),
}
impl ValidateLinks for ClipControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
        }
    }
}
