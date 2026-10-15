//! Authoring controls belong to their logical playback and controller owners.
use crate::model::controls::{diablo3, layered, model, playback};
use crate::model::{Link, ValidateLinks};
/// Native authoring behavior supplementing one logical clip.
#[derive(Clone, Debug)]
pub enum ClipControls<'a> {
    /// Diablo III permutation selection and timing.
    D3(diablo3::ClipControls),
    /// Warcraft III sequence selection and authored interval controls.
    Mdlx(layered::Sequence),
    /// World of Warcraft sequence selection and transition controls.
    M2(model::SequenceControls),
    /// StarCraft II sequence and controller selection.
    M3(playback::SequenceControls<'a>),
}
impl ValidateLinks for ClipControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::D3(value) => value.visit_links(visitor),

            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
            Self::M3(value) => value.visit_links(visitor),
        }
    }
}
