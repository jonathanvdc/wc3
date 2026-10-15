//! World scene vocabulary selected by the world profile.
use crate::model::profiles::Wmo;
pub use crate::model::scene::world::*;
use crate::model::FeatureSet;
/// Complete world feature attached to a graph node.
pub type Node<'a> = FeatureSet<'a, Wmo>;
