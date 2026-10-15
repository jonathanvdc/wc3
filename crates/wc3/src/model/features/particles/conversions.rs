//! Checked projection of shared components into constrained simulation vocabularies.
use super::*;
impl<'a> From<mdlx::Particle<'a>> for ParticleDefinition<'a> {
    fn from(v: mdlx::Particle<'a>) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Animated(v.emission.rate),
                variation: 0.0,
                trigger: ParticleTrigger::Continuous,
                activation: ParticleActivation::Fractional(v.emission.activation),
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Animated(v.motion.speed),
                variation: ParticleScalar::Animated(Animated::constant(0.0)),
                gravity: ParticleGravity::Directional(v.motion.gravity),
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Animated(v.lifetime.duration),
                variation: ParticleScalar::Animated(Animated::constant(0.0)),
            },
            appearance: ParticleAppearance::Resource,
            details: ParticleDetails::Model(v.details),
        }
    }
}
impl<'a> TryFrom<ParticleDefinition<'a>> for mdlx::Particle<'a> {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from(v: ParticleDefinition<'a>) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.rate, ParticleScalar::Animated(_))
            && matches!(&v.emission.activation, ParticleActivation::Fractional(_))
            && matches!(&v.motion.speed, ParticleScalar::Animated(_))
            && matches!(&v.motion.gravity, ParticleGravity::Directional(_))
            && matches!(&v.lifetime.duration, ParticleScalar::Animated(_))
            && v.emission.variation == 0.0
            && matches!(&v.motion.variation, ParticleScalar::Animated(a) if a.property.is_none() && a.default == 0.0)
            && matches!(&v.lifetime.variation, ParticleScalar::Animated(a) if a.property.is_none() && a.default == 0.0)
            && matches!(v.emission.trigger, ParticleTrigger::Continuous))
            || !matches!(&v.details, ParticleDetails::Model(_))
            || !matches!(&v.appearance, ParticleAppearance::Resource)
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Animated(emission_rate) = v.emission.rate else {
            unreachable!()
        };
        let ParticleActivation::Fractional(visibility) = v.emission.activation else {
            unreachable!()
        };
        let ParticleScalar::Animated(initial_velocity) = v.motion.speed else {
            unreachable!()
        };
        let ParticleGravity::Directional(gravity) = v.motion.gravity else {
            unreachable!()
        };
        let ParticleScalar::Animated(life_span) = v.lifetime.duration else {
            unreachable!()
        };
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
impl<'a> From<mdlx::Particle2> for ParticleDefinition<'a> {
    fn from(v: mdlx::Particle2) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Animated(v.emission.rate),
                variation: 0.0,
                trigger: if v.emission.trigger {
                    ParticleTrigger::KeyedBurst
                } else {
                    ParticleTrigger::Continuous
                },
                activation: ParticleActivation::Fractional(v.emission.activation),
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Animated(v.motion.speed),
                variation: ParticleScalar::Animated(v.motion.variation),
                gravity: ParticleGravity::Directional(v.motion.gravity),
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Animated(Animated::constant(v.lifetime.duration)),
                variation: ParticleScalar::Animated(Animated::constant(0.0)),
            },
            appearance: ParticleAppearance::Staged(v.appearance),
            details: ParticleDetails::Atlas(Box::new(v.details)),
        }
    }
}
impl<'a> TryFrom<ParticleDefinition<'a>> for mdlx::Particle2 {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from(v: ParticleDefinition<'a>) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.rate, ParticleScalar::Animated(_))
            && matches!(&v.emission.activation, ParticleActivation::Fractional(_))
            && matches!(&v.motion.speed, ParticleScalar::Animated(_))
            && matches!(&v.motion.variation, ParticleScalar::Animated(_))
            && matches!(&v.motion.gravity, ParticleGravity::Directional(_))
            && matches!(&v.lifetime.duration, ParticleScalar::Animated(a) if a.property.is_none())
            && matches!(
                &v.emission.trigger,
                ParticleTrigger::Continuous | ParticleTrigger::KeyedBurst
            )
            && v.emission.variation == 0.0
            && matches!(&v.lifetime.variation, ParticleScalar::Animated(a) if a.property.is_none() && a.default == 0.0))
            || !matches!(&v.details, ParticleDetails::Atlas(_))
            || !matches!(&v.appearance, ParticleAppearance::Staged(_))
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Animated(emission_rate) = v.emission.rate else {
            unreachable!()
        };
        let ParticleActivation::Fractional(visibility) = v.emission.activation else {
            unreachable!()
        };
        let ParticleScalar::Animated(speed) = v.motion.speed else {
            unreachable!()
        };
        let ParticleScalar::Animated(variation) = v.motion.variation else {
            unreachable!()
        };
        let ParticleGravity::Directional(gravity) = v.motion.gravity else {
            unreachable!()
        };
        let ParticleScalar::Animated(life_span) = v.lifetime.duration else {
            unreachable!()
        };
        let life_span = life_span.default;
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
impl<'a> From<m2::Particle<'a>> for ParticleDefinition<'a> {
    fn from(v: m2::Particle<'a>) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Animated(v.emission.rate),
                variation: v.emission.variation,
                trigger: ParticleTrigger::Continuous,
                activation: ParticleActivation::Boolean(v.emission.activation),
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Animated(v.motion.speed),
                variation: ParticleScalar::Animated(v.motion.variation),
                gravity: ParticleGravity::Vector(v.motion.gravity),
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Animated(v.lifetime.duration),
                variation: ParticleScalar::Animated(Animated::constant(v.lifetime.variation)),
            },
            appearance: ParticleAppearance::Curves(Box::new(v.appearance)),
            details: ParticleDetails::AgeDriven(Box::new(v.details)),
        }
    }
}
impl<'a> TryFrom<ParticleDefinition<'a>> for m2::Particle<'a> {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from(v: ParticleDefinition<'a>) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.rate, ParticleScalar::Animated(_))
            && matches!(&v.emission.activation, ParticleActivation::Boolean(_))
            && matches!(&v.motion.speed, ParticleScalar::Animated(_))
            && matches!(&v.motion.variation, ParticleScalar::Animated(_))
            && matches!(&v.motion.gravity, ParticleGravity::Vector(_))
            && matches!(&v.lifetime.duration, ParticleScalar::Animated(_))
            && matches!(&v.lifetime.variation, ParticleScalar::Animated(a) if a.property.is_none())
            && matches!(v.emission.trigger, ParticleTrigger::Continuous))
            || !matches!(&v.details, ParticleDetails::AgeDriven(_))
            || !matches!(&v.appearance, ParticleAppearance::Curves(_))
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Animated(rate) = v.emission.rate else {
            unreachable!()
        };
        let ParticleActivation::Boolean(enabled) = v.emission.activation else {
            unreachable!()
        };
        let ParticleScalar::Animated(speed) = v.motion.speed else {
            unreachable!()
        };
        let ParticleScalar::Animated(speed_variation) = v.motion.variation else {
            unreachable!()
        };
        let ParticleGravity::Vector(gravity) = v.motion.gravity else {
            unreachable!()
        };
        let ParticleScalar::Animated(lifespan) = v.lifetime.duration else {
            unreachable!()
        };
        let ParticleScalar::Animated(lifespan_variation) = v.lifetime.variation else {
            unreachable!()
        };
        let lifespan_variation = lifespan_variation.default;
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
impl<'a> From<m3::Particle<'a>> for ParticleDefinition<'a> {
    fn from(v: m3::Particle<'a>) -> Self {
        Self {
            emission: ParticleEmission {
                rate: ParticleScalar::Programmed(v.emission.rate),
                variation: 0.0,
                trigger: ParticleTrigger::Programmed(v.emission.trigger),
                activation: ParticleActivation::Programmed,
            },
            motion: ParticleMotion {
                speed: ParticleScalar::Programmed(v.motion.speed),
                variation: ParticleScalar::Programmed(v.motion.variation),
                gravity: ParticleGravity::Programmed {
                    x: v.motion.gravity.x,
                    y: v.motion.gravity.y,
                    magnitude: v.motion.gravity.magnitude,
                },
            },
            lifetime: ParticleLifetime {
                duration: ParticleScalar::Programmed(v.lifetime.duration),
                variation: ParticleScalar::Programmed(v.lifetime.variation),
            },
            appearance: ParticleAppearance::Programmed(Box::new(v.appearance)),
            details: ParticleDetails::Programmed(Box::new(v.details)),
        }
    }
}
impl<'a> TryFrom<ParticleDefinition<'a>> for m3::Particle<'a> {
    type Error = Box<ParticleDefinition<'a>>;
    fn try_from(v: ParticleDefinition<'a>) -> Result<Self, Self::Error> {
        if !(matches!(&v.emission.activation, ParticleActivation::Programmed)
            && matches!(&v.emission.rate, ParticleScalar::Programmed(_))
            && matches!(&v.emission.trigger, ParticleTrigger::Programmed(_))
            && matches!(&v.motion.speed, ParticleScalar::Programmed(_))
            && matches!(&v.motion.variation, ParticleScalar::Programmed(_))
            && matches!(&v.lifetime.duration, ParticleScalar::Programmed(_))
            && matches!(&v.lifetime.variation, ParticleScalar::Programmed(_))
            && matches!(&v.motion.gravity, ParticleGravity::Programmed { .. })
            && v.emission.variation == 0.0)
            || !matches!(&v.details, ParticleDetails::Programmed(_))
            || !matches!(&v.appearance, ParticleAppearance::Programmed(_))
        {
            return Err(Box::new(v));
        }
        let ParticleScalar::Programmed(emission_rate) = v.emission.rate else {
            unreachable!()
        };
        let ParticleTrigger::Programmed(squirt_amount) = v.emission.trigger else {
            unreachable!()
        };
        let ParticleScalar::Programmed(initial_speed) = v.motion.speed else {
            unreachable!()
        };
        let ParticleScalar::Programmed(initial_speed_random) = v.motion.variation else {
            unreachable!()
        };
        let ParticleScalar::Programmed(lifetime) = v.lifetime.duration else {
            unreachable!()
        };
        let ParticleScalar::Programmed(lifetime_random) = v.lifetime.variation else {
            unreachable!()
        };
        let ParticleGravity::Programmed {
            x: gravity_x,
            y: gravity_y,
            magnitude: gravity,
        } = v.motion.gravity
        else {
            unreachable!()
        };
        let ParticleAppearance::Programmed(appearance) = v.appearance else {
            unreachable!()
        };
        let ParticleDetails::Programmed(d) = v.details else {
            unreachable!()
        };
        Ok(Self {
            details: *d,
            motion: ParticleMotion {
                speed: initial_speed,
                variation: initial_speed_random,
                gravity: ProgrammedParticleGravity {
                    x: gravity_x,
                    y: gravity_y,
                    magnitude: gravity,
                },
            },
            lifetime: ParticleLifetime {
                duration: lifetime,
                variation: lifetime_random,
            },
            appearance: *appearance,
            emission: ParticleEmission {
                rate: emission_rate,
                trigger: squirt_amount,
                variation: (),
                activation: (),
            },
        })
    }
}
