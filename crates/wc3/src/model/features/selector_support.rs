//! Structural selectors follow the actual component definitions.
use super::{
    Camera, CameraConvention, NodeFeature, ParticleDetails, ParticleEmitter, ParticleScalar,
    ParticleTrigger, RibbonEmitter,
};
use crate::model::{FeatureProperty, PropertyIdentityKey};
impl NodeFeature<'_> {
    pub(crate) fn supports_shared_feature_property(
        &self,
        key: &PropertyIdentityKey,
    ) -> Option<bool> {
        let property = key.downcast_ref::<FeatureProperty>()?;
        Some(match property {
            FeatureProperty::SocketVisibility => {
                matches!(self, Self::Attachment(value) if value.socket().is_some())
            }
            FeatureProperty::CameraVerticalFieldOfView => {
                matches!(self, Self::Camera(Camera::Aimed(camera)) if matches!(camera.convention, CameraConvention::Vertical(_)))
            }
            FeatureProperty::TrailColor
            | FeatureProperty::TrailOpacity
            | FeatureProperty::TrailHeightAbove
            | FeatureProperty::TrailHeightBelow => {
                matches!(self, Self::Ribbon(RibbonEmitter::Trail(_)))
            }
            FeatureProperty::ParticleContinuousRate => {
                matches!(self, Self::Particle(ParticleEmitter::Simulated(value)) if matches!(value.emission.trigger, ParticleTrigger::Continuous) && matches!(value.emission.rate, ParticleScalar::Animated(_)))
            }
            FeatureProperty::ParticleLifetime => {
                matches!(self, Self::Particle(ParticleEmitter::Simulated(value)) if matches!(value.lifetime.duration, ParticleScalar::Animated(_)))
            }
            FeatureProperty::ParticleSpeed => {
                matches!(self, Self::Particle(ParticleEmitter::Simulated(value)) if matches!(value.motion.speed, ParticleScalar::Animated(_)))
            }
            _ => return None,
        })
    }
    pub(crate) fn is_programmed_particle(&self) -> bool {
        matches!(self, Self::Particle(ParticleEmitter::Simulated(value)) if matches!(value.details, ParticleDetails::Programmed(_)))
    }
}

use super::{ParticleActivation, ParticleDefinition, ParticleGravity};
use crate::model::profiles::{m2, mdlx};
impl ParticleDefinition<'_> {
    pub(crate) fn supports_resource_property(
        &self,
        field: &mdlx::properties::ParticleField,
    ) -> bool {
        use mdlx::properties::ParticleField as Field;
        matches!(self.details, ParticleDetails::Model(_))
            && match field {
                Field::Gravity => matches!(self.motion.gravity, ParticleGravity::Directional(_)),
                Field::Visibility => {
                    matches!(self.emission.activation, ParticleActivation::Fractional(_))
                }
                Field::EmissionRate => {
                    matches!(self.emission.rate, ParticleScalar::Animated(_))
                        && matches!(self.emission.trigger, ParticleTrigger::Continuous)
                }
                Field::LifeSpan => matches!(self.lifetime.duration, ParticleScalar::Animated(_)),
                Field::InitialVelocity => matches!(self.motion.speed, ParticleScalar::Animated(_)),
                _ => true,
            }
    }
    pub(crate) fn supports_atlas_property(&self, field: &mdlx::properties::Particle2Field) -> bool {
        use mdlx::properties::Particle2Field as Field;
        matches!(self.details, ParticleDetails::Atlas(_))
            && match field {
                Field::Gravity => matches!(self.motion.gravity, ParticleGravity::Directional(_)),
                Field::Visibility => {
                    matches!(self.emission.activation, ParticleActivation::Fractional(_))
                }
                Field::EmissionRate => {
                    matches!(self.emission.rate, ParticleScalar::Animated(_))
                        && matches!(
                            self.emission.trigger,
                            ParticleTrigger::Continuous | ParticleTrigger::KeyedBurst
                        )
                }
                Field::Speed => matches!(self.motion.speed, ParticleScalar::Animated(_)),
                Field::Variation => matches!(self.motion.variation, ParticleScalar::Animated(_)),
                _ => true,
            }
    }
    pub(crate) fn supports_age_driven_property(
        &self,
        field: &m2::properties::ParticleField,
    ) -> bool {
        use m2::properties::ParticleField as Field;
        matches!(self.details, ParticleDetails::AgeDriven(_))
            && match field {
                Field::Gravity => matches!(self.motion.gravity, ParticleGravity::Vector(_)),
                Field::Enabled => {
                    matches!(self.emission.activation, ParticleActivation::Boolean(_))
                }
                Field::Rate => {
                    matches!(self.emission.rate, ParticleScalar::Animated(_))
                        && matches!(self.emission.trigger, ParticleTrigger::Continuous)
                }
                Field::Speed => matches!(self.motion.speed, ParticleScalar::Animated(_)),
                Field::SpeedVariation => {
                    matches!(self.motion.variation, ParticleScalar::Animated(_))
                }
                Field::Lifespan => matches!(self.lifetime.duration, ParticleScalar::Animated(_)),
                _ => true,
            }
    }
}

use super::{ParticleAppearance, ParticleLifetime, ParticleMotion, ProgrammedRibbon};
use crate::model::profiles::m3::properties::Field;
fn supports_programmed_motion(
    field: &Field,
    motion: &ParticleMotion,
    lifetime: &ParticleLifetime,
) -> bool {
    match field {
        Field::InitialSpeed => matches!(motion.speed, ParticleScalar::Programmed(_)),
        Field::InitialSpeedRandom => matches!(motion.variation, ParticleScalar::Programmed(_)),
        Field::Lifetime => matches!(lifetime.duration, ParticleScalar::Programmed(_)),
        Field::LifetimeRandom => matches!(lifetime.variation, ParticleScalar::Programmed(_)),
        _ => true,
    }
}
impl ParticleDefinition<'_> {
    pub(crate) fn supports_programmed_property(&self, field: &Field) -> bool {
        matches!(self.details, ParticleDetails::Programmed(_))
            && supports_programmed_motion(field, &self.motion, &self.lifetime)
            && match field {
                Field::EmissionRate => matches!(self.emission.rate, ParticleScalar::Programmed(_)),
                Field::SquirtAmount => {
                    matches!(self.emission.trigger, ParticleTrigger::Programmed(_))
                }
                Field::SizeAnimation
                | Field::RotationAnimation
                | Field::ColorStart
                | Field::ColorMid
                | Field::ColorEnd => matches!(self.appearance, ParticleAppearance::Programmed(_)),
                _ => true,
            }
    }
}
impl ProgrammedRibbon<'_> {
    pub(crate) fn supports_programmed_property(&self, field: &Field) -> bool {
        supports_programmed_motion(field, &self.motion, &self.lifetime)
    }
}
