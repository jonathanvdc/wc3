//! Bounded outward-motion detection for analytically simulated sphere particles.
//!
//! This deliberately samples movement intervals. It approximates a discrete
//! native integrator while retaining the renderer's existing analytic trajectory.
use super::emission::EmissionClock;
use bevy::math::Vec3;

const MAX_PARTICLE_STEPS: usize = 32_768;

/// Apply one shared particle-step budget to a chronological prefix. The common
/// emission clock keeps fractional boundary intervals consistent across frames.
pub(super) fn bounded_end(start: f64, end: f64, particles: usize) -> f64 {
    if particles == 0 {
        return end;
    }
    EmissionClock::new(start, end)
        .take(MAX_PARTICLE_STEPS / particles)
        .last()
        .map_or(start, |step| step.end)
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct OutwardCrossing {
    /// End of the first interval whose movement points away from the live center.
    pub time: Option<f64>,
    /// Tail omitted by the shared long-update work budget.
    pub skipped_seconds: f64,
}

/// Test attempted displacement against the final radial position in bounded
/// 60 Hz intervals. Position and center must use the same simulation frame.
/// An attached emitter supplies the local origin; a detached emitter supplies
/// its current/interpolated world-space position, independent of birth position.
pub(super) fn outward_crossing(
    start: f64,
    end: f64,
    mut position: impl FnMut(f64) -> Vec3,
    mut center: impl FnMut(f64) -> Vec3,
) -> OutwardCrossing {
    let mut clock = EmissionClock::new(start, end);
    let mut previous = position(start);
    for interval in clock.by_ref() {
        let current = position(interval.end);
        let radial = current - center(interval.end);
        if (current - previous).dot(radial) > 0. {
            return OutwardCrossing {
                time: Some(interval.end),
                skipped_seconds: 0.,
            };
        }
        previous = current;
    }
    OutwardCrossing {
        time: None,
        skipped_seconds: clock.omitted_seconds(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_population_keeps_a_normal_step_and_caps_long_updates() {
        let particles = 8192;
        let normal_end = 1. / 60.;
        assert_eq!(bounded_end(0., normal_end, particles), normal_end);
        let end = bounded_end(0., 100., particles);
        let mut checks = 0;
        for _ in 0..particles {
            let result = outward_crossing(
                0.,
                end,
                |_| Vec3::ZERO,
                |_| {
                    checks += 1;
                    Vec3::ZERO
                },
            );
            assert_eq!(result.skipped_seconds, 0.);
        }
        assert_eq!(checks, MAX_PARTICLE_STEPS);
        assert_eq!(bounded_end(0., 100., 0), 100.);
    }

    #[test]
    fn fractional_quanta_use_the_common_clock_boundaries() {
        let start = 0.01;
        let end = start + 1. / 60.;
        assert_eq!(bounded_end(start, end, 8192), end);
        let mut checked = Vec::new();
        outward_crossing(
            start,
            end,
            |_| Vec3::ZERO,
            |time| {
                checked.push(time);
                Vec3::ZERO
            },
        );
        assert_eq!(checked, [1. / 60., end]);
    }

    #[test]
    fn inward_particles_survive_until_they_recede_and_outward_births_retire() {
        let result = outward_crossing(0., 2., |time| Vec3::X * (1. - time as f32), |_| Vec3::ZERO);
        let time = result.time.unwrap();
        assert!((1. ..=1. + 1. / 60.).contains(&time));
        assert_eq!(
            outward_crossing(0., 0.5, |time| Vec3::X * (1. - time as f32), |_| Vec3::ZERO).time,
            None
        );
        assert!(
            outward_crossing(0., 1., |time| Vec3::X * (1. + time as f32), |_| Vec3::ZERO)
                .time
                .unwrap()
                <= 1. / 60.
        );
    }

    #[test]
    fn detached_filter_uses_live_center_and_stationary_motion_survives() {
        assert_eq!(
            outward_crossing(0., 1., |_| Vec3::X, |_| Vec3::ZERO).time,
            None
        );
        // Particle moves toward its birth center, away from the live center.
        let result = outward_crossing(
            0.,
            0.5,
            |time| Vec3::X * (1. - time as f32),
            |_| Vec3::X * 2.,
        );
        assert!(result.time.is_some());
    }

    #[test]
    fn a_long_interval_cannot_hide_an_earlier_outward_excursion() {
        let result = outward_crossing(
            0.,
            2.,
            |time| Vec3::X * (1. + (time as f32 * 3.).sin()),
            |_| Vec3::ZERO,
        );
        assert!(result.time.unwrap() <= 1. / 60.);
        let bounded = outward_crossing(0., 100., |_| Vec3::ZERO, |_| Vec3::ZERO);
        assert_eq!(bounded.time, None);
        assert!(bounded.skipped_seconds > 31.);
    }
}
