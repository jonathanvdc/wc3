//! Typed World of Warcraft emitter destinations in the WEM vocabulary.
use crate::model::semantic::profiles::wem::properties::Target as WemTarget;
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{FeatureId, OwnerId, PropertyKey, PropertyPayload};
use std::any::TypeId;
/// Animated emission rate.
pub const EMISSION_RATE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Wow(Target::EmissionRate));
/// Animated horizontal range.
pub const HORIZONTAL_RANGE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Wow(Target::HorizontalRange));
/// Animated length.
pub const LENGTH: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Wow(Target::Length));
/// Animated lifespan.
pub const LIFESPAN: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Wow(Target::Lifespan));
/// Animated speed.
pub const SPEED: PropertyKey<FeatureId, f32, Wem> = PropertyKey::new(WemTarget::Wow(Target::Speed));
/// Animated speed variation.
pub const SPEED_VARIATION: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Wow(Target::SpeedVariation));
/// Animated vertical range.
pub const VERTICAL_RANGE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Wow(Target::VerticalRange));
/// Animated width.
pub const WIDTH: PropertyKey<FeatureId, f32, Wem> = PropertyKey::new(WemTarget::Wow(Target::Width));
/// Animated z source.
pub const Z_SOURCE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Wow(Target::ZSource));
/// Animated gravity vector.
pub const GRAVITY: PropertyKey<FeatureId, [f32; 3], Wem> =
    PropertyKey::new(WemTarget::Wow(Target::Gravity));

/// Structural destinations owned by this property vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Animated emission rate.
    EmissionRate,
    /// Animated horizontal range.
    HorizontalRange,
    /// Animated length.
    Length,
    /// Animated lifespan.
    Lifespan,
    /// Animated speed.
    Speed,
    /// Animated speed variation.
    SpeedVariation,
    /// Animated vertical range.
    VerticalRange,
    /// Animated width.
    Width,
    /// Animated z source.
    ZSource,
    /// Animated gravity.
    Gravity,
}

impl PropertyPayload for Target {
    fn value_type(&self) -> Option<TypeId> {
        Some(match self {
            Target::EmissionRate => TypeId::of::<f32>(),
            Target::HorizontalRange => TypeId::of::<f32>(),
            Target::Length => TypeId::of::<f32>(),
            Target::Lifespan => TypeId::of::<f32>(),
            Target::Speed => TypeId::of::<f32>(),
            Target::SpeedVariation => TypeId::of::<f32>(),
            Target::VerticalRange => TypeId::of::<f32>(),
            Target::Width => TypeId::of::<f32>(),
            Target::ZSource => TypeId::of::<f32>(),
            Target::Gravity => TypeId::of::<[f32; 3]>(),
        })
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Target::EmissionRate => matches!(owner, OwnerId::Feature(_)),
            Target::HorizontalRange => matches!(owner, OwnerId::Feature(_)),
            Target::Length => matches!(owner, OwnerId::Feature(_)),
            Target::Lifespan => matches!(owner, OwnerId::Feature(_)),
            Target::Speed => matches!(owner, OwnerId::Feature(_)),
            Target::SpeedVariation => matches!(owner, OwnerId::Feature(_)),
            Target::VerticalRange => matches!(owner, OwnerId::Feature(_)),
            Target::Width => matches!(owner, OwnerId::Feature(_)),
            Target::ZSource => matches!(owner, OwnerId::Feature(_)),
            Target::Gravity => matches!(owner, OwnerId::Feature(_)),
        }
    }
}
