//! Authoring controls belong to their logical playback and controller owners.
use crate::model::semantic::{profiles, Link, ValidateLinks};
/// Native authoring behavior supplementing one logical clip.
#[derive(Clone, Debug)]
pub enum ClipControls<'a> {
    /// Diablo III permutation selection and timing.
    D3(profiles::diablo::d3::ClipControls),
    /// Warcraft III sequence selection and authored interval controls.
    Mdlx(profiles::mdlx::Sequence),
    /// World of Warcraft sequence selection and transition controls.
    M2(profiles::m2::SequenceControls),
    /// StarCraft II sequence and controller selection.
    M3(profiles::m3::SequenceControls<'a>),
    /// Editable-model playback declarations and authored metadata.
    Wem(Box<profiles::wem::ClipControls<'a>>),
}
impl ValidateLinks for ClipControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::D3(value) => value.visit_links(visitor),
            Self::Wem(value) => value.visit_links(visitor),
            Self::Mdlx(value) => value.visit_links(visitor),
            Self::M2(value) => value.visit_links(visitor),
            Self::M3(value) => value.visit_links(visitor),
        }
    }
}
/// Native behavior belonging to one controller program.
#[derive(Clone, Debug)]
pub enum ControllerControls<'a> {
    /// StarCraft II channel-program evaluation controls.
    M3(profiles::m3::ControllerControls<'a>),
    /// Editable-model layer metadata.
    Wem(profiles::wem::ControllerControls<'a>),
}
impl ValidateLinks for ControllerControls<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Wem(value) => value.visit_links(visitor),
            Self::M3(value) => value.visit_links(visitor),
        }
    }
}
