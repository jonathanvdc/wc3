//! Programmed feature vocabulary selection.
use crate::model::semantic::{profiles::M3, FeatureSet};
pub use crate::model::semantic::{IkCcd, IkJoint, IkTwoJoint, ShadowBox, ViewVolume};
/// Programmed behaviors attached to nodes.
pub type Feature<'a> = FeatureSet<'a, M3>;
