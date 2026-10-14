use super::Bound;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::actors::starcraft::Feature;
use crate::model::semantic::{FeatureId, OwnerId, Property, Values};
use crate::model::wem::records;
pub(in crate::model::conversion::wem) fn bind_property(
    actor: &mut Feature<'_>,
    id: FeatureId,
    code: u32,
    element: Option<usize>,
) -> Result<Option<Bound>, Error> {
    let owner = OwnerId::Feature(id);
    Ok(Some(match actor {
        Feature::Particle(value) => match records::Sc2ParticleProperty::from_raw(code) {
            records::Sc2ParticleProperty::InitialSpeed => {
                let property = Property::named(owner, "initial_speed");
                value.initial_speed.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialSpeedRandom => {
                let property = Property::named(owner, "initial_speed_random");
                value.initial_speed_random.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialYaw => {
                let property = Property::named(owner, "initial_yaw");
                value.initial_yaw.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialPitch => {
                let property = Property::named(owner, "initial_pitch");
                value.initial_pitch.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialHorizontal => {
                let property = Property::named(owner, "initial_horizontal");
                value.initial_horizontal.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InitialVertical => {
                let property = Property::named(owner, "initial_vertical");
                value.initial_vertical.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::Lifetime => {
                let property = Property::named(owner, "lifetime");
                value.lifetime.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::LifetimeRandom => {
                let property = Property::named(owner, "lifetime_random");
                value.lifetime_random.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SizeAnimation => {
                let property = Property::named(owner, "size_animation");
                value.size_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::RotationAnimation => {
                let property = Property::named(owner, "rotation_animation");
                value.rotation_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::ColorStart => {
                let property = Property::named(owner, "color_start");
                value.color_start.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::ColorMid => {
                let property = Property::named(owner, "color_mid");
                value.color_mid.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::ColorEnd => {
                let property = Property::named(owner, "color_end");
                value.color_end.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::EmissionRate => {
                let property = Property::named(owner, "emission_rate");
                value.emission_rate.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::ShapeOuter => {
                let property = Property::named(owner, "shape_outer");
                value.shape_outer.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::ShapeInner => {
                let property = Property::named(owner, "shape_inner");
                value.shape_inner.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::OuterRadius => {
                let property = Property::named(owner, "outer_radius");
                value.outer_radius.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::InnerRadius => {
                let property = Property::named(owner, "inner_radius");
                value.inner_radius.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SizeRandomAnimation => {
                let property = Property::named(owner, "size_random_animation");
                value.size_random_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::RotationRandomAnimation => {
                let property = Property::named(owner, "rotation_random_animation");
                value.rotation_random_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::ColorStartRandom => {
                let property = Property::named(owner, "color_start_random");
                value.color_start_random.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::ColorMidRandom => {
                let property = Property::named(owner, "color_mid_random");
                value.color_mid_random.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::ColorEndRandom => {
                let property = Property::named(owner, "color_end_random");
                value.color_end_random.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2ParticleProperty::SquirtAmount => {
                let property = Property::named(owner, "squirt_amount");
                value.squirt_amount.binding = Some(property.clone());
                Bound::Unsigned(property)
            }
            records::Sc2ParticleProperty::PitchAmplitude => {
                let property = Property::named(owner, "pitch.amplitude");
                value.pitch.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::PitchFrequency => {
                let property = Property::named(owner, "pitch.frequency");
                value.pitch.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::YawAmplitude => {
                let property = Property::named(owner, "yaw.amplitude");
                value.yaw.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::YawFrequency => {
                let property = Property::named(owner, "yaw.frequency");
                value.yaw.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SpeedAmplitude => {
                let property = Property::named(owner, "speed.amplitude");
                value.speed.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SpeedFrequency => {
                let property = Property::named(owner, "speed.frequency");
                value.speed.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SizeAmplitude => {
                let property = Property::named(owner, "size.amplitude");
                value.size.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SizeFrequency => {
                let property = Property::named(owner, "size.frequency");
                value.size.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::AlphaAmplitude => {
                let property = Property::named(owner, "alpha.amplitude");
                value.alpha.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::AlphaFrequency => {
                let property = Property::named(owner, "alpha.frequency");
                value.alpha.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::ColorAmplitude => {
                let property = Property::named(owner, "color.amplitude");
                value.color.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::ColorFrequency => {
                let property = Property::named(owner, "color.frequency");
                value.color.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::RotationAmplitude => {
                let property = Property::named(owner, "rotation.amplitude");
                value.rotation.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::RotationFrequency => {
                let property = Property::named(owner, "rotation.frequency");
                value.rotation.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::HorizontalAmplitude => {
                let property = Property::named(owner, "horizontal.amplitude");
                value.horizontal.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::HorizontalFrequency => {
                let property = Property::named(owner, "horizontal.frequency");
                value.horizontal.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::VerticalAmplitude => {
                let property = Property::named(owner, "vertical.amplitude");
                value.vertical.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::VerticalFrequency => {
                let property = Property::named(owner, "vertical.frequency");
                value.vertical.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::ParticleVelocity => {
                let property = Property::named(owner, "particle_velocity");
                value.particle_velocity.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::PhaseShift => {
                let property = Property::named(owner, "phase_shift");
                value.phase_shift.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::AlphaThreshold => {
                let property = Property::named(owner, "alpha_threshold");
                value.alpha_threshold.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::UvOffset => {
                let property = Property::named(owner, "uv_offset");
                value.uv_offset.binding = Some(property.clone());
                Bound::Vector2(property)
            }
            records::Sc2ParticleProperty::UvAngle => {
                let property = Property::named(owner, "uv_angle");
                value.uv_angle.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::UvTiling => {
                let property = Property::named(owner, "uv_tiling");
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
                let property = Property::named(owner, format!("spline_points[{index}]"));
                control.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, control);
                Bound::Vector3(property)
            }
            records::Sc2ParticleProperty::LowerBound => {
                let property = Property::named(owner, "lower_bound");
                value.lower_bound.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::UpperBound => {
                let property = Property::named(owner, "upper_bound");
                value.upper_bound.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::TrailEmissionRate => {
                let property = Property::named(owner, "trail_emission_rate");
                value.trail_emission_rate.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            _ => return Ok(None),
        },
        Feature::Ribbon(value) => match records::Sc2RibbonProperty::from_raw(code) {
            records::Sc2RibbonProperty::InitialSpeed => {
                let property = Property::named(owner, "initial_speed");
                value.initial_speed.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialSpeedRandom => {
                let property = Property::named(owner, "initial_speed_random");
                value.initial_speed_random.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialYaw => {
                let property = Property::named(owner, "initial_yaw");
                value.initial_yaw.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialPitch => {
                let property = Property::named(owner, "initial_pitch");
                value.initial_pitch.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialHorizontal => {
                let property = Property::named(owner, "initial_horizontal");
                value.initial_horizontal.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::InitialVertical => {
                let property = Property::named(owner, "initial_vertical");
                value.initial_vertical.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::Lifetime => {
                let property = Property::named(owner, "lifetime");
                value.lifetime.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::LifetimeRandom => {
                let property = Property::named(owner, "lifetime_random");
                value.lifetime_random.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SizeAnimation => {
                let property = Property::named(owner, "size_animation");
                value.size_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2RibbonProperty::RotationAnimation => {
                let property = Property::named(owner, "rotation_animation");
                value.rotation_animation.binding = Some(property.clone());
                Bound::Vector3(property)
            }
            records::Sc2RibbonProperty::ColorStart => {
                let property = Property::named(owner, "color_start");
                value.color_start.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2RibbonProperty::ColorMid => {
                let property = Property::named(owner, "color_mid");
                value.color_mid.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2RibbonProperty::ColorEnd => {
                let property = Property::named(owner, "color_end");
                value.color_end.binding = Some(property.clone());
                Bound::Vector4(property)
            }
            records::Sc2RibbonProperty::MaxLength => {
                let property = Property::named(owner, "max_length");
                value.max_length.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::Active => {
                let property = Property::named(owner, "active");
                value.active.binding = Some(property.clone());
                Bound::Unsigned(property)
            }
            records::Sc2RibbonProperty::YawAmplitude => {
                let property = Property::named(owner, "yaw.amplitude");
                value.yaw.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::YawFrequency => {
                let property = Property::named(owner, "yaw.frequency");
                value.yaw.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::PitchAmplitude => {
                let property = Property::named(owner, "pitch.amplitude");
                value.pitch.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::PitchFrequency => {
                let property = Property::named(owner, "pitch.frequency");
                value.pitch.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SpeedAmplitude => {
                let property = Property::named(owner, "speed.amplitude");
                value.speed.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SpeedFrequency => {
                let property = Property::named(owner, "speed.frequency");
                value.speed.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SizeAmplitude => {
                let property = Property::named(owner, "size.amplitude");
                value.size.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::SizeFrequency => {
                let property = Property::named(owner, "size.frequency");
                value.size.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::AlphaAmplitude => {
                let property = Property::named(owner, "alpha.amplitude");
                value.alpha.amplitude.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::AlphaFrequency => {
                let property = Property::named(owner, "alpha.frequency");
                value.alpha.frequency.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::ParticleVelocity => {
                let property = Property::named(owner, "particle_velocity");
                value.particle_velocity.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2RibbonProperty::Overlay => {
                let property = Property::named(owner, "overlay");
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
                let property = Property::named(owner, format!("spline_points[{index}].velocity"));
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
                let property = Property::named(
                    owner,
                    format!("spline_points[{index}].velocity_base_factor"),
                );
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
                let property =
                    Property::named(owner, format!("spline_points[{index}].velocity_end_factor"));
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
                let property = Property::named(
                    owner,
                    format!("spline_points[{index}].yaw_variation.amplitude"),
                );
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
                let property = Property::named(
                    owner,
                    format!("spline_points[{index}].yaw_variation.frequency"),
                );
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
                let property = Property::named(
                    owner,
                    format!("spline_points[{index}].pitch_variation.amplitude"),
                );
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
                let property = Property::named(
                    owner,
                    format!("spline_points[{index}].pitch_variation.frequency"),
                );
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
                let property = Property::named(
                    owner,
                    format!("spline_points[{index}].velocity_variation.amplitude"),
                );
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
                let property = Property::named(
                    owner,
                    format!("spline_points[{index}].velocity_variation.frequency"),
                );
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
                let property = Property::named(owner, format!("spline_points[{index}].yaw"));
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
                let property = Property::named(owner, format!("spline_points[{index}].pitch"));
                point.pitch.binding = Some(property.clone());
                replace_point(&mut value.spline_points, index, point);
                Bound::Scalar(property)
            }
            _ => return Ok(None),
        },
        Feature::ParticleCopy(value) => match records::Sc2ParticleProperty::from_raw(code) {
            records::Sc2ParticleProperty::EmissionRate => {
                let property = Property::named(owner, "emission_rate");
                value.emission_rate.binding = Some(property.clone());
                Bound::Scalar(property)
            }
            records::Sc2ParticleProperty::SquirtAmount => {
                let property = Property::named(owner, "squirt_amount");
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
