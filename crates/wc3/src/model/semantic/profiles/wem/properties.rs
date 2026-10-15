//! Structural property destinations for WEM's distinct actor vocabularies.
use super::actors::{starcraft, warcraft, wow};
use crate::model::semantic::{OwnerId, PropertyPayload};
use std::any::TypeId;
/// A producer-specific actor property; equal spelling never merges distinct vocabularies.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Warcraft actor field.
    Warcraft(warcraft::properties::Target),
    /// StarCraft actor field.
    Starcraft(starcraft::properties::Target),
    /// World of Warcraft actor field.
    Wow(wow::properties::Target),
}
impl PropertyPayload for Target {
    fn value_type(&self) -> Option<TypeId> {
        match self {
            Self::Warcraft(v) => v.value_type(),
            Self::Starcraft(v) => v.value_type(),
            Self::Wow(v) => v.value_type(),
        }
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Self::Warcraft(v) => v.accepts_owner(owner),
            Self::Starcraft(v) => v.accepts_owner(owner),
            Self::Wow(v) => v.accepts_owner(owner),
        }
    }
}

impl Target {
    pub(crate) fn supports_feature(&self, feature: &super::Feature<'_>) -> bool {
        match (self, feature) {
            (Self::Warcraft(key), super::Feature::Warcraft(feature)) => {
                use warcraft::properties::Target as Key;
                match feature.as_ref() {
                    warcraft::Feature::ResourceParticles(_) => matches!(
                        key,
                        Key::EmissionRate
                            | Key::Gravity
                            | Key::Longitude
                            | Key::Latitude
                            | Key::Lifespan
                            | Key::Speed
                    ),
                    warcraft::Feature::SpriteParticles(_) => matches!(
                        key,
                        Key::Speed
                            | Key::Variation
                            | Key::Latitude
                            | Key::Gravity
                            | Key::EmissionRate
                            | Key::Width
                            | Key::Length
                    ),
                    warcraft::Feature::Ribbon(_) => {
                        matches!(key, Key::HeightAbove | Key::HeightBelow)
                    }
                    warcraft::Feature::Popcorn(_) => {
                        matches!(key, Key::Lifespan | Key::EmissionRate | Key::Speed)
                    }
                    _ => false,
                }
            }
            (Self::Wow(_), super::Feature::Wow(_)) => true,
            (Self::Starcraft(key), super::Feature::Starcraft(feature)) => {
                if let starcraft::properties::Target::SplinePoint { index, .. } = key {
                    return match feature {
                        starcraft::Feature::Particle(value) => *index < value.spline_points.len(),
                        starcraft::Feature::Ribbon(value) => *index < value.spline_points.len(),
                        _ => false,
                    };
                }
                match feature {
                    starcraft::Feature::ParticleCopy(_) => matches!(
                        key,
                        starcraft::properties::Target::EmissionRate
                            | starcraft::properties::Target::SquirtAmount
                    ),
                    _ => true,
                }
            }
            _ => false,
        }
    }
}
