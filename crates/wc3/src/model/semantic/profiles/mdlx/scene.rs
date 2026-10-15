//! Keyframed scene forms selected by the Warcraft III profile.
use crate::model::semantic::profiles::Mdlx;
pub use crate::model::semantic::scene::keyframed::*;
use crate::model::semantic::FeatureSet;
/// Complete profile-specific feature attached to a hierarchy node.
pub type Node<'a> = FeatureSet<'a, Mdlx>;
