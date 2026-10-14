//! Typed StarCraft II destinations for hierarchy and animated bounds.
use crate::model::semantic::{FeatureId, NodeId, PropertyKey};
/// Animated hierarchy batching selection.
pub const BATCHING: PropertyKey<NodeId, u32> = PropertyKey::new("batching");
/// Animated minimum of a bounding declaration.
pub const EXTENT_MINIMUM: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("extent.minimum");
/// Animated maximum of a bounding declaration.
pub const EXTENT_MAXIMUM: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("extent.maximum");
/// Animated radius of a bounding declaration.
pub const EXTENT_RADIUS: PropertyKey<FeatureId, f32> = PropertyKey::new("extent.radius");
