//! Authoring controls belong to their logical playback owners.
use crate::model::profiles::{m2, mdlx};
use crate::model::{Link, ValidateLinks};
/// Native authoring behavior supplementing one logical clip.
#[derive(Clone, Debug)]
pub enum ClipProperties {
    /// Warcraft III sequence selection and authored interval controls.
    Mdlx(mdlx::SequenceProperties),
    /// World of Warcraft sequence selection and transition controls.
    M2(m2::SequenceProperties),
}
impl ValidateLinks for ClipProperties {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
        }
    }
}
