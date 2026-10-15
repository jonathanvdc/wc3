//! Keyframed scene forms selected by the Warcraft III profile.
use crate::model::profiles::Mdlx;
pub use crate::model::scene::keyframed::*;
use crate::model::FeatureSet;
/// Complete profile-specific feature attached to a hierarchy node.
pub type Node<'a> = FeatureSet<'a, Mdlx>;
