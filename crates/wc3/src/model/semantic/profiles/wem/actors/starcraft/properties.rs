//! Typed StarCraft emitter destinations, independent of WEM selectors.
use crate::model::semantic::profiles::wem::properties::Target as WemTarget;
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{FeatureId, OwnerId, PropertyKey, PropertyPayload};
use std::any::TypeId;
/// Animated initial speed.
pub const INITIAL_SPEED: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::InitialSpeed));
/// Animated initial speed random.
pub const INITIAL_SPEED_RANDOM: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::InitialSpeedRandom));
/// Animated initial yaw.
pub const INITIAL_YAW: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::InitialYaw));
/// Animated initial pitch.
pub const INITIAL_PITCH: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::InitialPitch));
/// Animated initial horizontal.
pub const INITIAL_HORIZONTAL: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::InitialHorizontal));
/// Animated initial vertical.
pub const INITIAL_VERTICAL: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::InitialVertical));
/// Animated lifetime.
pub const LIFETIME: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::Lifetime));
/// Animated lifetime random.
pub const LIFETIME_RANDOM: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::LifetimeRandom));
/// Animated size animation.
pub const SIZE_ANIMATION: PropertyKey<FeatureId, [f32; 3], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::SizeAnimation));
/// Animated rotation animation.
pub const ROTATION_ANIMATION: PropertyKey<FeatureId, [f32; 3], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::RotationAnimation));
/// Animated color start.
pub const COLOR_START: PropertyKey<FeatureId, [f32; 4], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ColorStart));
/// Animated color mid.
pub const COLOR_MID: PropertyKey<FeatureId, [f32; 4], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ColorMid));
/// Animated color end.
pub const COLOR_END: PropertyKey<FeatureId, [f32; 4], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ColorEnd));
/// Animated emission rate.
pub const EMISSION_RATE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::EmissionRate));
/// Animated shape outer.
pub const SHAPE_OUTER: PropertyKey<FeatureId, [f32; 3], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ShapeOuter));
/// Animated shape inner.
pub const SHAPE_INNER: PropertyKey<FeatureId, [f32; 3], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ShapeInner));
/// Animated outer radius.
pub const OUTER_RADIUS: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::OuterRadius));
/// Animated inner radius.
pub const INNER_RADIUS: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::InnerRadius));
/// Animated size random animation.
pub const SIZE_RANDOM_ANIMATION: PropertyKey<FeatureId, [f32; 3], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::SizeRandomAnimation));
/// Animated rotation random animation.
pub const ROTATION_RANDOM_ANIMATION: PropertyKey<FeatureId, [f32; 3], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::RotationRandomAnimation));
/// Animated color start random.
pub const COLOR_START_RANDOM: PropertyKey<FeatureId, [f32; 4], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ColorStartRandom));
/// Animated color mid random.
pub const COLOR_MID_RANDOM: PropertyKey<FeatureId, [f32; 4], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ColorMidRandom));
/// Animated color end random.
pub const COLOR_END_RANDOM: PropertyKey<FeatureId, [f32; 4], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ColorEndRandom));
/// Animated squirt amount.
pub const SQUIRT_AMOUNT: PropertyKey<FeatureId, u32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::SquirtAmount));
/// Animated pitch.amplitude.
pub const PITCH_AMPLITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::PitchAmplitude));
/// Animated pitch.frequency.
pub const PITCH_FREQUENCY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::PitchFrequency));
/// Animated yaw.amplitude.
pub const YAW_AMPLITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::YawAmplitude));
/// Animated yaw.frequency.
pub const YAW_FREQUENCY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::YawFrequency));
/// Animated speed.amplitude.
pub const SPEED_AMPLITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::SpeedAmplitude));
/// Animated speed.frequency.
pub const SPEED_FREQUENCY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::SpeedFrequency));
/// Animated size.amplitude.
pub const SIZE_AMPLITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::SizeAmplitude));
/// Animated size.frequency.
pub const SIZE_FREQUENCY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::SizeFrequency));
/// Animated alpha.amplitude.
pub const ALPHA_AMPLITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::AlphaAmplitude));
/// Animated alpha.frequency.
pub const ALPHA_FREQUENCY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::AlphaFrequency));
/// Animated color.amplitude.
pub const COLOR_AMPLITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ColorAmplitude));
/// Animated color.frequency.
pub const COLOR_FREQUENCY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ColorFrequency));
/// Animated rotation.amplitude.
pub const ROTATION_AMPLITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::RotationAmplitude));
/// Animated rotation.frequency.
pub const ROTATION_FREQUENCY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::RotationFrequency));
/// Animated horizontal.amplitude.
pub const HORIZONTAL_AMPLITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::HorizontalAmplitude));
/// Animated horizontal.frequency.
pub const HORIZONTAL_FREQUENCY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::HorizontalFrequency));
/// Animated vertical.amplitude.
pub const VERTICAL_AMPLITUDE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::VerticalAmplitude));
/// Animated vertical.frequency.
pub const VERTICAL_FREQUENCY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::VerticalFrequency));
/// Animated particle velocity.
pub const PARTICLE_VELOCITY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::ParticleVelocity));
/// Animated phase shift.
pub const PHASE_SHIFT: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::PhaseShift));
/// Animated alpha threshold.
pub const ALPHA_THRESHOLD: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::AlphaThreshold));
/// Animated uv offset.
pub const UV_OFFSET: PropertyKey<FeatureId, [f32; 2], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::UvOffset));
/// Animated uv angle.
pub const UV_ANGLE: PropertyKey<FeatureId, [f32; 3], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::UvAngle));
/// Animated uv tiling.
pub const UV_TILING: PropertyKey<FeatureId, [f32; 2], Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::UvTiling));
/// Animated lower bound.
pub const LOWER_BOUND: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::LowerBound));
/// Animated upper bound.
pub const UPPER_BOUND: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::UpperBound));
/// Animated trail emission rate.
pub const TRAIL_EMISSION_RATE: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::TrailEmissionRate));
/// Animated max length.
pub const MAX_LENGTH: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::MaxLength));
/// Animated active.
pub const ACTIVE: PropertyKey<FeatureId, u32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::Active));
/// Animated overlay.
pub const OVERLAY: PropertyKey<FeatureId, f32, Wem> =
    PropertyKey::new(WemTarget::Starcraft(Target::Overlay));

/// Indexed spline-point destinations on an emitter.
#[derive(Clone, Copy, Debug)]
pub struct SplinePoint(pub usize);
impl SplinePoint {
    /// Animated point position.
    pub fn position(self) -> PropertyKey<FeatureId, [f32; 3], Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::Position,
        }))
    }
    /// Animated point velocity.
    pub fn velocity(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::Velocity,
        }))
    }
    /// Animated point velocity base factor.
    pub fn velocity_base_factor(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::VelocityBaseFactor,
        }))
    }
    /// Animated point velocity end factor.
    pub fn velocity_end_factor(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::VelocityEndFactor,
        }))
    }
    /// Animated point yaw variation.amplitude.
    pub fn yaw_variation_amplitude(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::YawVariationAmplitude,
        }))
    }
    /// Animated point yaw variation.frequency.
    pub fn yaw_variation_frequency(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::YawVariationFrequency,
        }))
    }
    /// Animated point pitch variation.amplitude.
    pub fn pitch_variation_amplitude(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::PitchVariationAmplitude,
        }))
    }
    /// Animated point pitch variation.frequency.
    pub fn pitch_variation_frequency(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::PitchVariationFrequency,
        }))
    }
    /// Animated point velocity variation.amplitude.
    pub fn velocity_variation_amplitude(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::VelocityVariationAmplitude,
        }))
    }
    /// Animated point velocity variation.frequency.
    pub fn velocity_variation_frequency(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::VelocityVariationFrequency,
        }))
    }
    /// Animated point yaw.
    pub fn yaw(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::Yaw,
        }))
    }
    /// Animated point pitch.
    pub fn pitch(self) -> PropertyKey<FeatureId, f32, Wem> {
        PropertyKey::new(WemTarget::Starcraft(Target::SplinePoint {
            index: self.0,
            field: SplinePointField::Pitch,
        }))
    }
}

/// Structural destinations owned by this property vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Target {
    /// Animated initial speed.
    InitialSpeed,
    /// Animated initial speed random.
    InitialSpeedRandom,
    /// Animated initial yaw.
    InitialYaw,
    /// Animated initial pitch.
    InitialPitch,
    /// Animated initial horizontal.
    InitialHorizontal,
    /// Animated initial vertical.
    InitialVertical,
    /// Animated lifetime.
    Lifetime,
    /// Animated lifetime random.
    LifetimeRandom,
    /// Animated size animation.
    SizeAnimation,
    /// Animated rotation animation.
    RotationAnimation,
    /// Animated color start.
    ColorStart,
    /// Animated color mid.
    ColorMid,
    /// Animated color end.
    ColorEnd,
    /// Animated emission rate.
    EmissionRate,
    /// Animated shape outer.
    ShapeOuter,
    /// Animated shape inner.
    ShapeInner,
    /// Animated outer radius.
    OuterRadius,
    /// Animated inner radius.
    InnerRadius,
    /// Animated size random animation.
    SizeRandomAnimation,
    /// Animated rotation random animation.
    RotationRandomAnimation,
    /// Animated color start random.
    ColorStartRandom,
    /// Animated color mid random.
    ColorMidRandom,
    /// Animated color end random.
    ColorEndRandom,
    /// Animated squirt amount.
    SquirtAmount,
    /// Animated pitch amplitude.
    PitchAmplitude,
    /// Animated pitch frequency.
    PitchFrequency,
    /// Animated yaw amplitude.
    YawAmplitude,
    /// Animated yaw frequency.
    YawFrequency,
    /// Animated speed amplitude.
    SpeedAmplitude,
    /// Animated speed frequency.
    SpeedFrequency,
    /// Animated size amplitude.
    SizeAmplitude,
    /// Animated size frequency.
    SizeFrequency,
    /// Animated alpha amplitude.
    AlphaAmplitude,
    /// Animated alpha frequency.
    AlphaFrequency,
    /// Animated color amplitude.
    ColorAmplitude,
    /// Animated color frequency.
    ColorFrequency,
    /// Animated rotation amplitude.
    RotationAmplitude,
    /// Animated rotation frequency.
    RotationFrequency,
    /// Animated horizontal amplitude.
    HorizontalAmplitude,
    /// Animated horizontal frequency.
    HorizontalFrequency,
    /// Animated vertical amplitude.
    VerticalAmplitude,
    /// Animated vertical frequency.
    VerticalFrequency,
    /// Animated particle velocity.
    ParticleVelocity,
    /// Animated phase shift.
    PhaseShift,
    /// Animated alpha threshold.
    AlphaThreshold,
    /// Animated uv offset.
    UvOffset,
    /// Animated uv angle.
    UvAngle,
    /// Animated uv tiling.
    UvTiling,
    /// Animated lower bound.
    LowerBound,
    /// Animated upper bound.
    UpperBound,
    /// Animated trail emission rate.
    TrailEmissionRate,
    /// Animated max length.
    MaxLength,
    /// Animated active.
    Active,
    /// Animated overlay.
    Overlay,
    /// Indexed splinepoint property.
    SplinePoint {
        /// Owner-local element ordinal.
        index: usize,
        /// Field of the selected element.
        field: SplinePointField,
    },
}
/// Structural splinepoint field selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SplinePointField {
    /// Spline-point position.
    Position,
    /// Spline-point velocity.
    Velocity,
    /// Spline-point velocity base factor.
    VelocityBaseFactor,
    /// Spline-point velocity end factor.
    VelocityEndFactor,
    /// Spline-point yaw variation amplitude.
    YawVariationAmplitude,
    /// Spline-point yaw variation frequency.
    YawVariationFrequency,
    /// Spline-point pitch variation amplitude.
    PitchVariationAmplitude,
    /// Spline-point pitch variation frequency.
    PitchVariationFrequency,
    /// Spline-point velocity variation amplitude.
    VelocityVariationAmplitude,
    /// Spline-point velocity variation frequency.
    VelocityVariationFrequency,
    /// Spline-point yaw.
    Yaw,
    /// Spline-point pitch.
    Pitch,
}

impl PropertyPayload for Target {
    fn value_type(&self) -> Option<TypeId> {
        Some(match self {
            Target::InitialSpeed => TypeId::of::<f32>(),
            Target::InitialSpeedRandom => TypeId::of::<f32>(),
            Target::InitialYaw => TypeId::of::<f32>(),
            Target::InitialPitch => TypeId::of::<f32>(),
            Target::InitialHorizontal => TypeId::of::<f32>(),
            Target::InitialVertical => TypeId::of::<f32>(),
            Target::Lifetime => TypeId::of::<f32>(),
            Target::LifetimeRandom => TypeId::of::<f32>(),
            Target::SizeAnimation => TypeId::of::<[f32; 3]>(),
            Target::RotationAnimation => TypeId::of::<[f32; 3]>(),
            Target::ColorStart => TypeId::of::<[f32; 4]>(),
            Target::ColorMid => TypeId::of::<[f32; 4]>(),
            Target::ColorEnd => TypeId::of::<[f32; 4]>(),
            Target::EmissionRate => TypeId::of::<f32>(),
            Target::ShapeOuter => TypeId::of::<[f32; 3]>(),
            Target::ShapeInner => TypeId::of::<[f32; 3]>(),
            Target::OuterRadius => TypeId::of::<f32>(),
            Target::InnerRadius => TypeId::of::<f32>(),
            Target::SizeRandomAnimation => TypeId::of::<[f32; 3]>(),
            Target::RotationRandomAnimation => TypeId::of::<[f32; 3]>(),
            Target::ColorStartRandom => TypeId::of::<[f32; 4]>(),
            Target::ColorMidRandom => TypeId::of::<[f32; 4]>(),
            Target::ColorEndRandom => TypeId::of::<[f32; 4]>(),
            Target::SquirtAmount => TypeId::of::<u32>(),
            Target::PitchAmplitude => TypeId::of::<f32>(),
            Target::PitchFrequency => TypeId::of::<f32>(),
            Target::YawAmplitude => TypeId::of::<f32>(),
            Target::YawFrequency => TypeId::of::<f32>(),
            Target::SpeedAmplitude => TypeId::of::<f32>(),
            Target::SpeedFrequency => TypeId::of::<f32>(),
            Target::SizeAmplitude => TypeId::of::<f32>(),
            Target::SizeFrequency => TypeId::of::<f32>(),
            Target::AlphaAmplitude => TypeId::of::<f32>(),
            Target::AlphaFrequency => TypeId::of::<f32>(),
            Target::ColorAmplitude => TypeId::of::<f32>(),
            Target::ColorFrequency => TypeId::of::<f32>(),
            Target::RotationAmplitude => TypeId::of::<f32>(),
            Target::RotationFrequency => TypeId::of::<f32>(),
            Target::HorizontalAmplitude => TypeId::of::<f32>(),
            Target::HorizontalFrequency => TypeId::of::<f32>(),
            Target::VerticalAmplitude => TypeId::of::<f32>(),
            Target::VerticalFrequency => TypeId::of::<f32>(),
            Target::ParticleVelocity => TypeId::of::<f32>(),
            Target::PhaseShift => TypeId::of::<f32>(),
            Target::AlphaThreshold => TypeId::of::<f32>(),
            Target::UvOffset => TypeId::of::<[f32; 2]>(),
            Target::UvAngle => TypeId::of::<[f32; 3]>(),
            Target::UvTiling => TypeId::of::<[f32; 2]>(),
            Target::LowerBound => TypeId::of::<f32>(),
            Target::UpperBound => TypeId::of::<f32>(),
            Target::TrailEmissionRate => TypeId::of::<f32>(),
            Target::MaxLength => TypeId::of::<f32>(),
            Target::Active => TypeId::of::<u32>(),
            Target::Overlay => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::Position,
                ..
            } => TypeId::of::<[f32; 3]>(),
            Target::SplinePoint {
                field: SplinePointField::Velocity,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::VelocityBaseFactor,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::VelocityEndFactor,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::YawVariationAmplitude,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::YawVariationFrequency,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::PitchVariationAmplitude,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::PitchVariationFrequency,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::VelocityVariationAmplitude,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::VelocityVariationFrequency,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::Yaw,
                ..
            } => TypeId::of::<f32>(),
            Target::SplinePoint {
                field: SplinePointField::Pitch,
                ..
            } => TypeId::of::<f32>(),
        })
    }
    fn accepts_owner(&self, owner: OwnerId) -> bool {
        match self {
            Target::InitialSpeed => matches!(owner, OwnerId::Feature(_)),
            Target::InitialSpeedRandom => matches!(owner, OwnerId::Feature(_)),
            Target::InitialYaw => matches!(owner, OwnerId::Feature(_)),
            Target::InitialPitch => matches!(owner, OwnerId::Feature(_)),
            Target::InitialHorizontal => matches!(owner, OwnerId::Feature(_)),
            Target::InitialVertical => matches!(owner, OwnerId::Feature(_)),
            Target::Lifetime => matches!(owner, OwnerId::Feature(_)),
            Target::LifetimeRandom => matches!(owner, OwnerId::Feature(_)),
            Target::SizeAnimation => matches!(owner, OwnerId::Feature(_)),
            Target::RotationAnimation => matches!(owner, OwnerId::Feature(_)),
            Target::ColorStart => matches!(owner, OwnerId::Feature(_)),
            Target::ColorMid => matches!(owner, OwnerId::Feature(_)),
            Target::ColorEnd => matches!(owner, OwnerId::Feature(_)),
            Target::EmissionRate => matches!(owner, OwnerId::Feature(_)),
            Target::ShapeOuter => matches!(owner, OwnerId::Feature(_)),
            Target::ShapeInner => matches!(owner, OwnerId::Feature(_)),
            Target::OuterRadius => matches!(owner, OwnerId::Feature(_)),
            Target::InnerRadius => matches!(owner, OwnerId::Feature(_)),
            Target::SizeRandomAnimation => matches!(owner, OwnerId::Feature(_)),
            Target::RotationRandomAnimation => matches!(owner, OwnerId::Feature(_)),
            Target::ColorStartRandom => matches!(owner, OwnerId::Feature(_)),
            Target::ColorMidRandom => matches!(owner, OwnerId::Feature(_)),
            Target::ColorEndRandom => matches!(owner, OwnerId::Feature(_)),
            Target::SquirtAmount => matches!(owner, OwnerId::Feature(_)),
            Target::PitchAmplitude => matches!(owner, OwnerId::Feature(_)),
            Target::PitchFrequency => matches!(owner, OwnerId::Feature(_)),
            Target::YawAmplitude => matches!(owner, OwnerId::Feature(_)),
            Target::YawFrequency => matches!(owner, OwnerId::Feature(_)),
            Target::SpeedAmplitude => matches!(owner, OwnerId::Feature(_)),
            Target::SpeedFrequency => matches!(owner, OwnerId::Feature(_)),
            Target::SizeAmplitude => matches!(owner, OwnerId::Feature(_)),
            Target::SizeFrequency => matches!(owner, OwnerId::Feature(_)),
            Target::AlphaAmplitude => matches!(owner, OwnerId::Feature(_)),
            Target::AlphaFrequency => matches!(owner, OwnerId::Feature(_)),
            Target::ColorAmplitude => matches!(owner, OwnerId::Feature(_)),
            Target::ColorFrequency => matches!(owner, OwnerId::Feature(_)),
            Target::RotationAmplitude => matches!(owner, OwnerId::Feature(_)),
            Target::RotationFrequency => matches!(owner, OwnerId::Feature(_)),
            Target::HorizontalAmplitude => matches!(owner, OwnerId::Feature(_)),
            Target::HorizontalFrequency => matches!(owner, OwnerId::Feature(_)),
            Target::VerticalAmplitude => matches!(owner, OwnerId::Feature(_)),
            Target::VerticalFrequency => matches!(owner, OwnerId::Feature(_)),
            Target::ParticleVelocity => matches!(owner, OwnerId::Feature(_)),
            Target::PhaseShift => matches!(owner, OwnerId::Feature(_)),
            Target::AlphaThreshold => matches!(owner, OwnerId::Feature(_)),
            Target::UvOffset => matches!(owner, OwnerId::Feature(_)),
            Target::UvAngle => matches!(owner, OwnerId::Feature(_)),
            Target::UvTiling => matches!(owner, OwnerId::Feature(_)),
            Target::LowerBound => matches!(owner, OwnerId::Feature(_)),
            Target::UpperBound => matches!(owner, OwnerId::Feature(_)),
            Target::TrailEmissionRate => matches!(owner, OwnerId::Feature(_)),
            Target::MaxLength => matches!(owner, OwnerId::Feature(_)),
            Target::Active => matches!(owner, OwnerId::Feature(_)),
            Target::Overlay => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::Position,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::Velocity,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::VelocityBaseFactor,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::VelocityEndFactor,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::YawVariationAmplitude,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::YawVariationFrequency,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::PitchVariationAmplitude,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::PitchVariationFrequency,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::VelocityVariationAmplitude,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::VelocityVariationFrequency,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::Yaw,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
            Target::SplinePoint {
                field: SplinePointField::Pitch,
                ..
            } => matches!(owner, OwnerId::Feature(_)),
        }
    }
}
