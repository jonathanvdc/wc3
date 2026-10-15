use super::Bound;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::actors::starcraft::{properties, Feature};
use crate::model::semantic::{FeatureId, Values};
use crate::model::wem::records;
pub(in crate::model::conversion::wem) fn bind_property(
    actor: &mut Feature<'_>,
    id: FeatureId,
    code: u32,
    element: Option<usize>,
) -> Result<Option<Bound>, Error> {
    Ok(Some(match actor {
        Feature::Particle(value) => match records::Sc2ParticleProperty::from_raw(code) {
            records::Sc2ParticleProperty::InitialSpeed => {
                let property = properties::INITIAL_SPEED.bind(id).widen();
                value.initial_speed.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialSpeedRandom => {
                let property = properties::INITIAL_SPEED_RANDOM.bind(id).widen();
                value.initial_speed_random.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialYaw => {
                let property = properties::INITIAL_YAW.bind(id).widen();
                value.initial_yaw.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialPitch => {
                let property = properties::INITIAL_PITCH.bind(id).widen();
                value.initial_pitch.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialHorizontal => {
                let property = properties::INITIAL_HORIZONTAL.bind(id).widen();
                value.initial_horizontal.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialVertical => {
                let property = properties::INITIAL_VERTICAL.bind(id).widen();
                value.initial_vertical.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::Lifetime => {
                let property = properties::LIFETIME.bind(id).widen();
                value.lifetime.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::LifetimeRandom => {
                let property = properties::LIFETIME_RANDOM.bind(id).widen();
                value.lifetime_random.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SizeAnimation => {
                let property = properties::SIZE_ANIMATION.bind(id).widen();
                value.size_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::RotationAnimation => {
                let property = properties::ROTATION_ANIMATION.bind(id).widen();
                value.rotation_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::ColorStart => {
                let property = properties::COLOR_START.bind(id).widen();
                value.color_start.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::ColorMid => {
                let property = properties::COLOR_MID.bind(id).widen();
                value.color_mid.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::ColorEnd => {
                let property = properties::COLOR_END.bind(id).widen();
                value.color_end.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::EmissionRate => {
                let property = properties::EMISSION_RATE.bind(id).widen();
                value.emission_rate.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::ShapeOuter => {
                let property = properties::SHAPE_OUTER.bind(id).widen();
                value.shape_outer.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::ShapeInner => {
                let property = properties::SHAPE_INNER.bind(id).widen();
                value.shape_inner.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::OuterRadius => {
                let property = properties::OUTER_RADIUS.bind(id).widen();
                value.outer_radius.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InnerRadius => {
                let property = properties::INNER_RADIUS.bind(id).widen();
                value.inner_radius.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SizeRandomAnimation => {
                let property = properties::SIZE_RANDOM_ANIMATION.bind(id).widen();
                value.size_random_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::RotationRandomAnimation => {
                let property = properties::ROTATION_RANDOM_ANIMATION.bind(id).widen();
                value.rotation_random_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::ColorStartRandom => {
                let property = properties::COLOR_START_RANDOM.bind(id).widen();
                value.color_start_random.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::ColorMidRandom => {
                let property = properties::COLOR_MID_RANDOM.bind(id).widen();
                value.color_mid_random.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::ColorEndRandom => {
                let property = properties::COLOR_END_RANDOM.bind(id).widen();
                value.color_end_random.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::SquirtAmount => {
                let property = properties::SQUIRT_AMOUNT.bind(id).widen();
                value.squirt_amount.binding = Some(property.clone());
                Bound::Unsigned(property)
            }
            records::Sc2ParticleProperty::PitchAmplitude => {
                let property = properties::PITCH_AMPLITUDE.bind(id).widen();
                value.pitch.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::PitchFrequency => {
                let property = properties::PITCH_FREQUENCY.bind(id).widen();
                value.pitch.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::YawAmplitude => {
                let property = properties::YAW_AMPLITUDE.bind(id).widen();
                value.yaw.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::YawFrequency => {
                let property = properties::YAW_FREQUENCY.bind(id).widen();
                value.yaw.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SpeedAmplitude => {
                let property = properties::SPEED_AMPLITUDE.bind(id).widen();
                value.speed.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SpeedFrequency => {
                let property = properties::SPEED_FREQUENCY.bind(id).widen();
                value.speed.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SizeAmplitude => {
                let property = properties::SIZE_AMPLITUDE.bind(id).widen();
                value.size.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SizeFrequency => {
                let property = properties::SIZE_FREQUENCY.bind(id).widen();
                value.size.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::AlphaAmplitude => {
                let property = properties::ALPHA_AMPLITUDE.bind(id).widen();
                value.alpha.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::AlphaFrequency => {
                let property = properties::ALPHA_FREQUENCY.bind(id).widen();
                value.alpha.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::ColorAmplitude => {
                let property = properties::COLOR_AMPLITUDE.bind(id).widen();
                value.color.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::ColorFrequency => {
                let property = properties::COLOR_FREQUENCY.bind(id).widen();
                value.color.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::RotationAmplitude => {
                let property = properties::ROTATION_AMPLITUDE.bind(id).widen();
                value.rotation.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::RotationFrequency => {
                let property = properties::ROTATION_FREQUENCY.bind(id).widen();
                value.rotation.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::HorizontalAmplitude => {
                let property = properties::HORIZONTAL_AMPLITUDE.bind(id).widen();
                value.horizontal.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::HorizontalFrequency => {
                let property = properties::HORIZONTAL_FREQUENCY.bind(id).widen();
                value.horizontal.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::VerticalAmplitude => {
                let property = properties::VERTICAL_AMPLITUDE.bind(id).widen();
                value.vertical.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::VerticalFrequency => {
                let property = properties::VERTICAL_FREQUENCY.bind(id).widen();
                value.vertical.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::ParticleVelocity => {
                let property = properties::PARTICLE_VELOCITY.bind(id).widen();
                value.particle_velocity.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::PhaseShift => {
                let property = properties::PHASE_SHIFT.bind(id).widen();
                value.phase_shift.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::AlphaThreshold => {
                let property = properties::ALPHA_THRESHOLD.bind(id).widen();
                value.alpha_threshold.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::UvOffset => {
                let property = properties::UV_OFFSET.bind(id).widen();
                value.uv_offset.binding = Some(property.clone());
                Bound::Vector2(property)
            }
            records::Sc2ParticleProperty::UvAngle => {
                let property = properties::UV_ANGLE.bind(id).widen();
                value.uv_angle.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::UvTiling => {
                let property = properties::UV_TILING.bind(id).widen();
                value.uv_tiling.binding = Some(property.clone());
                Bound::Vector2(property)
            }
            records::Sc2ParticleProperty::SplinePoint => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut control = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index).position().bind(id).widen();
                control.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, control);
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::LowerBound => {
                let property = properties::LOWER_BOUND.bind(id).widen();
                value.lower_bound.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::UpperBound => {
                let property = properties::UPPER_BOUND.bind(id).widen();
                value.upper_bound.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::TrailEmissionRate => {
                let property = properties::TRAIL_EMISSION_RATE.bind(id).widen();
                value.trail_emission_rate.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            _ => return Ok(None),
        },
        Feature::Ribbon(value) => match records::Sc2RibbonProperty::from_raw(code) {
            records::Sc2RibbonProperty::InitialSpeed => {
                let property = properties::INITIAL_SPEED.bind(id).widen();
                value.initial_speed.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialSpeedRandom => {
                let property = properties::INITIAL_SPEED_RANDOM.bind(id).widen();
                value.initial_speed_random.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialYaw => {
                let property = properties::INITIAL_YAW.bind(id).widen();
                value.initial_yaw.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialPitch => {
                let property = properties::INITIAL_PITCH.bind(id).widen();
                value.initial_pitch.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialHorizontal => {
                let property = properties::INITIAL_HORIZONTAL.bind(id).widen();
                value.initial_horizontal.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialVertical => {
                let property = properties::INITIAL_VERTICAL.bind(id).widen();
                value.initial_vertical.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::Lifetime => {
                let property = properties::LIFETIME.bind(id).widen();
                value.lifetime.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::LifetimeRandom => {
                let property = properties::LIFETIME_RANDOM.bind(id).widen();
                value.lifetime_random.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SizeAnimation => {
                let property = properties::SIZE_ANIMATION.bind(id).widen();
                value.size_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2RibbonProperty::RotationAnimation => {
                let property = properties::ROTATION_ANIMATION.bind(id).widen();
                value.rotation_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2RibbonProperty::ColorStart => {
                let property = properties::COLOR_START.bind(id).widen();
                value.color_start.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2RibbonProperty::ColorMid => {
                let property = properties::COLOR_MID.bind(id).widen();
                value.color_mid.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2RibbonProperty::ColorEnd => {
                let property = properties::COLOR_END.bind(id).widen();
                value.color_end.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2RibbonProperty::MaxLength => {
                let property = properties::MAX_LENGTH.bind(id).widen();
                value.max_length.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::Active => {
                let property = properties::ACTIVE.bind(id).widen();
                value.active.binding = Some(property.clone());
                Bound::Unsigned(property)
            }
            records::Sc2RibbonProperty::YawAmplitude => {
                let property = properties::YAW_AMPLITUDE.bind(id).widen();
                value.yaw.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::YawFrequency => {
                let property = properties::YAW_FREQUENCY.bind(id).widen();
                value.yaw.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::PitchAmplitude => {
                let property = properties::PITCH_AMPLITUDE.bind(id).widen();
                value.pitch.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::PitchFrequency => {
                let property = properties::PITCH_FREQUENCY.bind(id).widen();
                value.pitch.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SpeedAmplitude => {
                let property = properties::SPEED_AMPLITUDE.bind(id).widen();
                value.speed.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SpeedFrequency => {
                let property = properties::SPEED_FREQUENCY.bind(id).widen();
                value.speed.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SizeAmplitude => {
                let property = properties::SIZE_AMPLITUDE.bind(id).widen();
                value.size.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SizeFrequency => {
                let property = properties::SIZE_FREQUENCY.bind(id).widen();
                value.size.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::AlphaAmplitude => {
                let property = properties::ALPHA_AMPLITUDE.bind(id).widen();
                value.alpha.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::AlphaFrequency => {
                let property = properties::ALPHA_FREQUENCY.bind(id).widen();
                value.alpha.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::ParticleVelocity => {
                let property = properties::PARTICLE_VELOCITY.bind(id).widen();
                value.particle_velocity.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::Overlay => {
                let property = properties::OVERLAY.bind(id).widen();
                value.overlay.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplineVelocity => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index).velocity().bind(id).widen();
                point.velocity.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplineVelocityBaseFactor => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index)
                    .velocity_base_factor()
                    .bind(id)
                    .widen();
                point.velocity_base_factor.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplineVelocityEndFactor => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index)
                    .velocity_end_factor()
                    .bind(id)
                    .widen();
                point.velocity_end_factor.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplineYawAmplitude => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index)
                    .yaw_variation_amplitude()
                    .bind(id)
                    .widen();
                point.yaw_variation.amplitude.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplineYawFrequency => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index)
                    .yaw_variation_frequency()
                    .bind(id)
                    .widen();
                point.yaw_variation.frequency.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplinePitchAmplitude => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index)
                    .pitch_variation_amplitude()
                    .bind(id)
                    .widen();
                point.pitch_variation.amplitude.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplinePitchFrequency => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index)
                    .pitch_variation_frequency()
                    .bind(id)
                    .widen();
                point.pitch_variation.frequency.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplineVelocityAmplitude => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index)
                    .velocity_variation_amplitude()
                    .bind(id)
                    .widen();
                point.velocity_variation.amplitude.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplineVelocityFrequency => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index)
                    .velocity_variation_frequency()
                    .bind(id)
                    .widen();
                point.velocity_variation.frequency.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplineYaw => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index).yaw().bind(id).widen();
                point.yaw.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SplinePitch => {
                let index = element.ok_or_else(|| {
                    Error::new("wem.starcraft.property", "spline point selector is absent")
                })?;
                let mut point = value.spline_points.get(index).ok_or_else(|| {
                    Error::new(
                        "wem.starcraft.property",
                        "spline point does not belong to emitter",
                    )
                })?;
                let property = properties::SplinePoint(index).pitch().bind(id).widen();
                point.pitch.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            _ => return Ok(None),
        },
        Feature::ParticleCopy(value) => match records::Sc2ParticleProperty::from_raw(code) {
            records::Sc2ParticleProperty::EmissionRate => {
                let property = properties::EMISSION_RATE.bind(id).widen();
                value.emission_rate.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SquirtAmount => {
                let property = properties::SQUIRT_AMOUNT.bind(id).widen();
                value.squirt_amount.binding = Some(property.clone());
                Bound::Unsigned(property)
            }
            _ => return Ok(None),
        },
    }))
}

fn replace_point<T: Clone + Send + Sync + 'static>(
    values: &mut Values<'_, T>,
    index: usize,
    value: T,
) {
    let source = values.clone();
    *values = Values::view(source.len(), move |i| {
        if i == index {
            value.clone()
        } else {
            source.get(i).expect("checked ribbon point")
        }
    });
}
