//! Typed Warcraft emitter destinations in the WEM actor vocabulary.
use crate::model::semantic::{FeatureId, PropertyKey};
/// Animated emission rate.
pub const EMISSION_RATE: PropertyKey<FeatureId, f32> = PropertyKey::new("emission_rate");
/// Animated gravity.
pub const GRAVITY: PropertyKey<FeatureId, f32> = PropertyKey::new("gravity");
/// Animated height above.
pub const HEIGHT_ABOVE: PropertyKey<FeatureId, f32> = PropertyKey::new("height_above");
/// Animated height below.
pub const HEIGHT_BELOW: PropertyKey<FeatureId, f32> = PropertyKey::new("height_below");
/// Animated latitude.
pub const LATITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("latitude");
/// Animated length.
pub const LENGTH: PropertyKey<FeatureId, f32> = PropertyKey::new("length");
/// Animated lifespan.
pub const LIFESPAN: PropertyKey<FeatureId, f32> = PropertyKey::new("lifespan");
/// Animated longitude.
pub const LONGITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("longitude");
/// Animated speed.
pub const SPEED: PropertyKey<FeatureId, f32> = PropertyKey::new("speed");
/// Animated variation.
pub const VARIATION: PropertyKey<FeatureId, f32> = PropertyKey::new("variation");
/// Animated width.
pub const WIDTH: PropertyKey<FeatureId, f32> = PropertyKey::new("width");
