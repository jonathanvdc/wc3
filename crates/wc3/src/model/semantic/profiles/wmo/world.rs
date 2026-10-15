//! World scene vocabulary selected by the world profile.
use crate::model::semantic::profiles::Wmo;
pub use crate::model::semantic::scene::world::*;
use crate::model::semantic::FeatureSet;
/// Complete world feature attached to a graph node.
pub type Node<'a> = FeatureSet<'a, Wmo>;
