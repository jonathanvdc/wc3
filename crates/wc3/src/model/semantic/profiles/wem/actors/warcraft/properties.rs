//! Typed Warcraft emitter destinations in the WEM actor vocabulary.
use crate::model::semantic::profiles::wem::properties::Target as WemTarget;
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{FeatureId, OwnerId, PropertyKey, PropertyPayload};
use std::any::TypeId;
/// Animated emission rate.
pub const EMISSION_RATE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::EmissionRate));
/// Animated gravity.
pub const GRAVITY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::Gravity));
/// Animated height above.
pub const HEIGHT_ABOVE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::HeightAbove));
/// Animated height below.
pub const HEIGHT_BELOW: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::HeightBelow));
/// Animated latitude.
pub const LATITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::Latitude));
/// Animated length.
pub const LENGTH: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::Length));
/// Animated lifespan.
pub const LIFESPAN: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::Lifespan));
/// Animated longitude.
pub const LONGITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::Longitude));
/// Animated speed.
pub const SPEED: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::Speed));
/// Animated variation.
pub const VARIATION: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::Variation));
/// Animated width.
pub const WIDTH: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Warcraft(Target::Width));

/// Structural destinations owned by this property vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Animated emission rate.
    EmissionRate,
    /// Animated gravity.
    Gravity,
    /// Animated height above.
    HeightAbove,
    /// Animated height below.
    HeightBelow,
    /// Animated latitude.
    Latitude,
    /// Animated length.
    Length,
    /// Animated lifespan.
    Lifespan,
    /// Animated longitude.
    Longitude,
    /// Animated speed.
    Speed,
    /// Animated variation.
    Variation,
    /// Animated width.
    Width,
}

impl PropertyPayload for Target {
    fn value_type(&self) -> Option<TypeId> {
        Some(match self {
            Target::EmissionRate => TypeId::of::<f32>(),
            Target::Gravity => TypeId::of::<f32>(),
            Target::HeightAbove => TypeId::of::<f32>(),
            Target::HeightBelow => TypeId::of::<f32>(),
            Target::Latitude => TypeId::of::<f32>(),
            Target::Length => TypeId::of::<f32>(),
            Target::Lifespan => TypeId::of::<f32>(),
            Target::Longitude => TypeId::of::<f32>(),
            Target::Speed => TypeId::of::<f32>(),
            Target::Variation => TypeId::of::<f32>(),
            Target::Width => TypeId::of::<f32>(),
        })
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Target::EmissionRate => matches!(owner, OwnerId::Feature(_)),
            Target::Gravity => matches!(owner, OwnerId::Feature(_)),
            Target::HeightAbove => matches!(owner, OwnerId::Feature(_)),
            Target::HeightBelow => matches!(owner, OwnerId::Feature(_)),
            Target::Latitude => matches!(owner, OwnerId::Feature(_)),
            Target::Length => matches!(owner, OwnerId::Feature(_)),
            Target::Lifespan => matches!(owner, OwnerId::Feature(_)),
            Target::Longitude => matches!(owner, OwnerId::Feature(_)),
            Target::Speed => matches!(owner, OwnerId::Feature(_)),
            Target::Variation => matches!(owner, OwnerId::Feature(_)),
            Target::Width => matches!(owner, OwnerId::Feature(_)),
        }
    }
}
