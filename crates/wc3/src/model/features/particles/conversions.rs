//! Checked projection of shared components into constrained simulation vocabularies.
use super::*;
use crate::model::features::{FromFeature, TryFromFeature};
use crate::model::PropertyValues;
impl<'a> FromFeature<mdlx::Particle> for ParticleDefinition<'a> {
    fn from_feature(v: mdlx::Particle, values: &mut PropertyValues) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Property(v.emission.rate),
                variation: 0.0,
                trigger: ParticleTrigger::Continuous,
                activation: ParticleActivation::Fractional(v.emission.activation),
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Property(v.motion.speed),
                variation: ParticleScalar::Property(values.add(0.0)),
                gravity: ParticleGravity::Directional(v.motion.gravity),
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Property(v.lifetime.duration),
                variation: ParticleScalar::Property(values.add(0.0)),
            },
            appearance: ParticleAppearance::Resource,
            details: ParticleDetails::Model(v.details),
        }
    }
}
impl<'a> TryFromFeature<ParticleDefinition<'a>> for mdlx::Particle {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from_feature(
        v: ParticleDefinition<'a>,
        values: &PropertyValues,
    ) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.activation, ParticleActivation::Fractional(_))
            && matches!(&v.motion.gravity, ParticleGravity::Directional(_))
            && v.emission.variation == 0.0
            && matches!(&v.motion.variation, ParticleScalar::Property(a) if values.get(*a) == Some(&0.))
            && matches!(&v.lifetime.variation, ParticleScalar::Property(a) if values.get(*a) == Some(&0.))
            && matches!(v.emission.trigger, ParticleTrigger::Continuous)
            && matches!(&v.details, ParticleDetails::Model(_))
            && matches!(&v.appearance, ParticleAppearance::Resource))
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Property(emission_rate) = v.emission.rate;
        let ParticleActivation::Fractional(visibility) = v.emission.activation else {
            unreachable!()
        };
        let ParticleScalar::Property(initial_velocity) = v.motion.speed;
        let ParticleGravity::Directional(gravity) = v.motion.gravity else {
            unreachable!()
        };
        let ParticleScalar::Property(life_span) = v.lifetime.duration;
        let ParticleDetails::Model(d) = v.details else {
            unreachable!()
        };
        Ok(Self {
            details: d,
            emission: ParticleEmission {
                rate: emission_rate,
                activation: visibility,
                variation: (),
                trigger: (),
            },
            motion: ParticleMotion {
                gravity,
                speed: initial_velocity,
                variation: (),
            },
            lifetime: ParticleLifetime {
                duration: life_span,
                variation: (),
            },
            appearance: (),
        })
    }
}
impl<'a> FromFeature<mdlx::Particle2> for ParticleDefinition<'a> {
    fn from_feature(v: mdlx::Particle2, values: &mut PropertyValues) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Property(v.emission.rate),
                variation: 0.0,
                trigger: if v.emission.trigger {
                    ParticleTrigger::KeyedBurst
                } else {
                    ParticleTrigger::Continuous
                },
                activation: ParticleActivation::Fractional(v.emission.activation),
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Property(v.motion.speed),
                variation: ParticleScalar::Property(v.motion.variation),
                gravity: ParticleGravity::Directional(v.motion.gravity),
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Property(values.add(v.lifetime.duration)),
                variation: ParticleScalar::Property(values.add(0.0)),
            },
            appearance: ParticleAppearance::Staged(v.appearance),
            details: ParticleDetails::Atlas(Box::new(v.details)),
        }
    }
}
impl<'a> TryFromFeature<ParticleDefinition<'a>> for mdlx::Particle2 {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from_feature(
        v: ParticleDefinition<'a>,
        values: &PropertyValues,
    ) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.activation, ParticleActivation::Fractional(_))
            && matches!(&v.motion.gravity, ParticleGravity::Directional(_))
            && matches!(&v.lifetime.duration, ParticleScalar::Property(a) if values.get(*a).is_some())
            && matches!(
                &v.emission.trigger,
                ParticleTrigger::Continuous | ParticleTrigger::KeyedBurst
            )
            && v.emission.variation == 0.0
            && matches!(&v.lifetime.variation, ParticleScalar::Property(a) if values.get(*a) == Some(&0.))
            && matches!(&v.details, ParticleDetails::Atlas(_))
            && matches!(&v.appearance, ParticleAppearance::Staged(_)))
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Property(emission_rate) = v.emission.rate;
        let ParticleActivation::Fractional(visibility) = v.emission.activation else {
            unreachable!()
        };
        let ParticleScalar::Property(speed) = v.motion.speed;
        let ParticleScalar::Property(variation) = v.motion.variation;
        let ParticleGravity::Directional(gravity) = v.motion.gravity else {
            unreachable!()
        };
        let ParticleScalar::Property(life_span) = v.lifetime.duration;
        let life_span = *values.get(life_span).expect("validated source slot");
        let squirt = matches!(v.emission.trigger, ParticleTrigger::KeyedBurst);
        let ParticleAppearance::Staged(appearance) = v.appearance else {
            unreachable!()
        };
        let ParticleDetails::Atlas(d) = v.details else {
            unreachable!()
        };
        Ok(Self {
            motion: ParticleMotion {
                speed,
                variation,
                gravity,
            },
            details: *d,
            lifetime: ParticleLifetime {
                duration: life_span,
                variation: (),
            },
            emission: ParticleEmission {
                rate: emission_rate,
                trigger: squirt,
                activation: visibility,
                variation: (),
            },
            appearance,
        })
    }
}
impl<'a> FromFeature<m2::Particle<'a>> for ParticleDefinition<'a> {
    fn from_feature(v: m2::Particle<'a>, values: &mut PropertyValues) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Property(v.emission.rate),
                variation: v.emission.variation,
                trigger: ParticleTrigger::Continuous,
                activation: ParticleActivation::Boolean(v.emission.activation),
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Property(v.motion.speed),
                variation: ParticleScalar::Property(v.motion.variation),
                gravity: ParticleGravity::Vector(v.motion.gravity),
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Property(v.lifetime.duration),
                variation: ParticleScalar::Property(values.add(v.lifetime.variation)),
            },
            appearance: ParticleAppearance::Curves(Box::new(v.appearance)),
            details: ParticleDetails::AgeDriven(Box::new(v.details)),
        }
    }
}
impl<'a> TryFromFeature<ParticleDefinition<'a>> for m2::Particle<'a> {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from_feature(
        v: ParticleDefinition<'a>,
        values: &PropertyValues,
    ) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.activation, ParticleActivation::Boolean(_))
            && matches!(&v.motion.gravity, ParticleGravity::Vector(_))
            && matches!(&v.lifetime.variation, ParticleScalar::Property(a) if values.get(*a).is_some())
            && matches!(v.emission.trigger, ParticleTrigger::Continuous)
            && matches!(&v.details, ParticleDetails::AgeDriven(_))
            && matches!(&v.appearance, ParticleAppearance::Curves(_)))
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Property(rate) = v.emission.rate;
        let ParticleActivation::Boolean(enabled) = v.emission.activation else {
            unreachable!()
        };
        let ParticleScalar::Property(speed) = v.motion.speed;
        let ParticleScalar::Property(speed_variation) = v.motion.variation;
        let ParticleGravity::Vector(gravity) = v.motion.gravity else {
            unreachable!()
        };
        let ParticleScalar::Property(lifespan) = v.lifetime.duration;
        let ParticleScalar::Property(lifespan_variation) = v.lifetime.variation;
        let lifespan_variation = *values
            .get(lifespan_variation)
            .expect("validated source slot");
        let ParticleAppearance::Curves(appearance) = v.appearance else {
            unreachable!()
        };
        let ParticleDetails::AgeDriven(d) = v.details else {
            unreachable!()
        };
        Ok(Self {
            details: *d,
            motion: ParticleMotion {
                speed,
                variation: speed_variation,
                gravity,
            },
            lifetime: ParticleLifetime {
                duration: lifespan,
                variation: lifespan_variation,
            },
            emission: ParticleEmission {
                rate,
                variation: v.emission.variation,
                activation: enabled,
                trigger: (),
            },
            appearance: *appearance,
        })
    }
}
