use crate::model::semantic::profiles::wem::actors::starcraft::Feature;
use crate::model::semantic::PropertyTarget;
use crate::model::wem::records;
pub(in crate::model::conversion::wem) fn property_code(
    actor: &Feature<'_>,
    target: &PropertyTarget,
) -> Option<(u32, Option<usize>)> {
    match actor {
        Feature::Particle(value) => {
            if value
                .initial_speed
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::InitialSpeed.raw(), None));
            }
            if value
                .initial_speed_random
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::InitialSpeedRandom.raw(), None));
            }
            if value
                .initial_yaw
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::InitialYaw.raw(), None));
            }
            if value
                .initial_pitch
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::InitialPitch.raw(), None));
            }
            if value
                .initial_horizontal
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::InitialHorizontal.raw(), None));
            }
            if value
                .initial_vertical
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::InitialVertical.raw(), None));
            }
            if value
                .lifetime
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::Lifetime.raw(), None));
            }
            if value
                .lifetime_random
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::LifetimeRandom.raw(), None));
            }
            if value
                .size_animation
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::SizeAnimation.raw(), None));
            }
            if value
                .rotation_animation
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::RotationAnimation.raw(), None));
            }
            if value
                .color_start
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ColorStart.raw(), None));
            }
            if value
                .color_mid
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ColorMid.raw(), None));
            }
            if value
                .color_end
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ColorEnd.raw(), None));
            }
            if value
                .emission_rate
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::EmissionRate.raw(), None));
            }
            if value
                .shape_outer
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ShapeOuter.raw(), None));
            }
            if value
                .shape_inner
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ShapeInner.raw(), None));
            }
            if value
                .outer_radius
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::OuterRadius.raw(), None));
            }
            if value
                .inner_radius
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::InnerRadius.raw(), None));
            }
            if value
                .size_random_animation
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((
                    records::Sc2ParticleProperty::SizeRandomAnimation.raw(),
                    None,
                ));
            }
            if value
                .rotation_random_animation
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((
                    records::Sc2ParticleProperty::RotationRandomAnimation.raw(),
                    None,
                ));
            }
            if value
                .color_start_random
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ColorStartRandom.raw(), None));
            }
            if value
                .color_mid_random
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ColorMidRandom.raw(), None));
            }
            if value
                .color_end_random
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ColorEndRandom.raw(), None));
            }
            if value
                .squirt_amount
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::SquirtAmount.raw(), None));
            }
            if value
                .pitch
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::PitchAmplitude.raw(), None));
            }
            if value
                .pitch
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::PitchFrequency.raw(), None));
            }
            if value
                .yaw
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::YawAmplitude.raw(), None));
            }
            if value
                .yaw
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::YawFrequency.raw(), None));
            }
            if value
                .speed
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::SpeedAmplitude.raw(), None));
            }
            if value
                .speed
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::SpeedFrequency.raw(), None));
            }
            if value
                .size
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::SizeAmplitude.raw(), None));
            }
            if value
                .size
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::SizeFrequency.raw(), None));
            }
            if value
                .alpha
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::AlphaAmplitude.raw(), None));
            }
            if value
                .alpha
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::AlphaFrequency.raw(), None));
            }
            if value
                .color
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ColorAmplitude.raw(), None));
            }
            if value
                .color
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ColorFrequency.raw(), None));
            }
            if value
                .rotation
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::RotationAmplitude.raw(), None));
            }
            if value
                .rotation
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::RotationFrequency.raw(), None));
            }
            if value
                .horizontal
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((
                    records::Sc2ParticleProperty::HorizontalAmplitude.raw(),
                    None,
                ));
            }
            if value
                .horizontal
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((
                    records::Sc2ParticleProperty::HorizontalFrequency.raw(),
                    None,
                ));
            }
            if value
                .vertical
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::VerticalAmplitude.raw(), None));
            }
            if value
                .vertical
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::VerticalFrequency.raw(), None));
            }
            if value
                .particle_velocity
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::ParticleVelocity.raw(), None));
            }
            if value
                .phase_shift
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::PhaseShift.raw(), None));
            }
            if value
                .alpha_threshold
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::AlphaThreshold.raw(), None));
            }
            if value
                .uv_offset
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::UvOffset.raw(), None));
            }
            if value
                .uv_angle
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::UvAngle.raw(), None));
            }
            if value
                .uv_tiling
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::UvTiling.raw(), None));
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point.binding.as_ref().is_some_and(|p| p.target() == target) {
                    return Some((records::Sc2ParticleProperty::SplinePoint.raw(), Some(index)));
                }
            }
            if value
                .lower_bound
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::LowerBound.raw(), None));
            }
            if value
                .upper_bound
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::UpperBound.raw(), None));
            }
            if value
                .trail_emission_rate
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::TrailEmissionRate.raw(), None));
            }
            None
        }
        Feature::Ribbon(value) => {
            if value
                .initial_speed
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::InitialSpeed.raw(), None));
            }
            if value
                .initial_speed_random
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::InitialSpeedRandom.raw(), None));
            }
            if value
                .initial_yaw
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::InitialYaw.raw(), None));
            }
            if value
                .initial_pitch
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::InitialPitch.raw(), None));
            }
            if value
                .initial_horizontal
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::InitialHorizontal.raw(), None));
            }
            if value
                .initial_vertical
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::InitialVertical.raw(), None));
            }
            if value
                .lifetime
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::Lifetime.raw(), None));
            }
            if value
                .lifetime_random
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::LifetimeRandom.raw(), None));
            }
            if value
                .size_animation
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::SizeAnimation.raw(), None));
            }
            if value
                .rotation_animation
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::RotationAnimation.raw(), None));
            }
            if value
                .color_start
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::ColorStart.raw(), None));
            }
            if value
                .color_mid
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::ColorMid.raw(), None));
            }
            if value
                .color_end
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::ColorEnd.raw(), None));
            }
            if value
                .max_length
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::MaxLength.raw(), None));
            }
            if value
                .active
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::Active.raw(), None));
            }
            if value
                .yaw
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::YawAmplitude.raw(), None));
            }
            if value
                .yaw
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::YawFrequency.raw(), None));
            }
            if value
                .pitch
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::PitchAmplitude.raw(), None));
            }
            if value
                .pitch
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::PitchFrequency.raw(), None));
            }
            if value
                .speed
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::SpeedAmplitude.raw(), None));
            }
            if value
                .speed
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::SpeedFrequency.raw(), None));
            }
            if value
                .size
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::SizeAmplitude.raw(), None));
            }
            if value
                .size
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::SizeFrequency.raw(), None));
            }
            if value
                .alpha
                .amplitude
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::AlphaAmplitude.raw(), None));
            }
            if value
                .alpha
                .frequency
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::AlphaFrequency.raw(), None));
            }
            if value
                .particle_velocity
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::ParticleVelocity.raw(), None));
            }
            if value
                .overlay
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2RibbonProperty::Overlay.raw(), None));
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .velocity
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((
                        records::Sc2RibbonProperty::SplineVelocity.raw(),
                        Some(index),
                    ));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .velocity_base_factor
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((
                        records::Sc2RibbonProperty::SplineVelocityBaseFactor.raw(),
                        Some(index),
                    ));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .velocity_end_factor
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((
                        records::Sc2RibbonProperty::SplineVelocityEndFactor.raw(),
                        Some(index),
                    ));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .yaw_variation
                    .amplitude
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((
                        records::Sc2RibbonProperty::SplineYawAmplitude.raw(),
                        Some(index),
                    ));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .yaw_variation
                    .frequency
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((
                        records::Sc2RibbonProperty::SplineYawFrequency.raw(),
                        Some(index),
                    ));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .pitch_variation
                    .amplitude
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((
                        records::Sc2RibbonProperty::SplinePitchAmplitude.raw(),
                        Some(index),
                    ));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .pitch_variation
                    .frequency
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((
                        records::Sc2RibbonProperty::SplinePitchFrequency.raw(),
                        Some(index),
                    ));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .velocity_variation
                    .amplitude
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((
                        records::Sc2RibbonProperty::SplineVelocityAmplitude.raw(),
                        Some(index),
                    ));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .velocity_variation
                    .frequency
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((
                        records::Sc2RibbonProperty::SplineVelocityFrequency.raw(),
                        Some(index),
                    ));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .yaw
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((records::Sc2RibbonProperty::SplineYaw.raw(), Some(index)));
                }
            }
            for (index, point) in value.spline_points.iter().enumerate() {
                if point
                    .pitch
                    .binding
                    .as_ref()
                    .is_some_and(|p| p.target() == target)
                {
                    return Some((records::Sc2RibbonProperty::SplinePitch.raw(), Some(index)));
                }
            }
            None
        }
        Feature::ParticleCopy(value) => {
            if value
                .emission_rate
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::EmissionRate.raw(), None));
            }
            if value
                .squirt_amount
                .binding
                .as_ref()
                .is_some_and(|p| p.target() == target)
            {
                return Some((records::Sc2ParticleProperty::SquirtAmount.raw(), None));
            }
            None
        }
    }
}
