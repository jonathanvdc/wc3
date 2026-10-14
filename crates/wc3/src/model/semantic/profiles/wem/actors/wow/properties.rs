//! Typed World of Warcraft emitter destinations in the WEM vocabulary.
use crate::model::semantic::{FeatureId, PropertyKey};
/// Animated emission rate.
pub const EMISSION_RATE: PropertyKey<FeatureId, f32> = PropertyKey::new("emission_rate");
/// Animated horizontal range.
pub const HORIZONTAL_RANGE: PropertyKey<FeatureId, f32> = PropertyKey::new("horizontal_range");
/// Animated length.
pub const LENGTH: PropertyKey<FeatureId, f32> = PropertyKey::new("length");
/// Animated lifespan.
pub const LIFESPAN: PropertyKey<FeatureId, f32> = PropertyKey::new("lifespan");
/// Animated speed.
pub const SPEED: PropertyKey<FeatureId, f32> = PropertyKey::new("speed");
/// Animated speed variation.
pub const SPEED_VARIATION: PropertyKey<FeatureId, f32> = PropertyKey::new("speed_variation");
/// Animated vertical range.
pub const VERTICAL_RANGE: PropertyKey<FeatureId, f32> = PropertyKey::new("vertical_range");
/// Animated width.
pub const WIDTH: PropertyKey<FeatureId, f32> = PropertyKey::new("width");
/// Animated z source.
pub const Z_SOURCE: PropertyKey<FeatureId, f32> = PropertyKey::new("z_source");
/// Animated gravity vector.
pub const GRAVITY: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("gravity");
