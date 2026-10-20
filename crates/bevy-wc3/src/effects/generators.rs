//! Shape sampling independent of particle lifetime, placement, and transport.
use bevy::math::Vec3;

/// Sample the ordered path by segment index, independently of segment length.
/// Area controls select normalized path endpoints for this generator family.
pub(super) fn spline(points: &[Vec3], start: f32, end: f32, unit_sample: f32) -> (Vec3, Vec3) {
    let normalized = |value: f32| {
        if value.is_finite() {
            value.clamp(0., 1.)
        } else {
            0.
        }
    };
    let Some(&first) = points.first() else {
        return (Vec3::ZERO, Vec3::Z);
    };
    if points.len() == 1 {
        return (first, Vec3::Z);
    }
    let start = normalized(start);
    let end = normalized(end);
    let parameter = start + (end - start) * normalized(unit_sample);
    let segment_position = parameter * (points.len() - 1) as f32;
    let segment = (segment_position as usize).min(points.len() - 2);
    let a = points[segment];
    let b = points[segment + 1];
    (
        a.lerp(b, (segment_position - segment as f32).clamp(0., 1.)),
        (b - a).try_normalize().unwrap_or(Vec3::Z),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_parameters_follow_segments_instead_of_distance() {
        let path = [Vec3::ZERO, Vec3::X, Vec3::X + Vec3::Y * 9.];
        assert_eq!(spline(&path, 0., 1., 0.), (Vec3::ZERO, Vec3::X));
        assert_eq!(spline(&path, 0., 1., 0.25), (Vec3::X * 0.5, Vec3::X));
        assert_eq!(spline(&path, 0., 1., 0.5), (Vec3::X, Vec3::Y));
        assert_eq!(
            spline(&path, 0., 1., 0.75),
            (Vec3::X + Vec3::Y * 4.5, Vec3::Y)
        );
        assert_eq!(spline(&path, 0., 1., 1.), (path[2], Vec3::Y));
        assert_eq!(spline(&path, 1., 0., 0.25), spline(&path, 0., 1., 0.75));
        assert_eq!(spline(&path, -2., 3., 0.75), spline(&path, 0., 1., 0.75));
        assert_eq!(spline(&path, 0.25, 0.75, 0.5), (Vec3::X, Vec3::Y));
    }

    #[test]
    fn degenerate_paths_and_nonfinite_controls_have_finite_frames() {
        assert_eq!(spline(&[], 0., 1., 0.5), (Vec3::ZERO, Vec3::Z));
        assert_eq!(spline(&[Vec3::Y], 0., 1., 0.5), (Vec3::Y, Vec3::Z));
        assert_eq!(spline(&[Vec3::X, Vec3::X], 0., 1., 0.5), (Vec3::X, Vec3::Z));
        assert_eq!(
            spline(&[Vec3::ZERO, Vec3::X], f32::NAN, f32::INFINITY, 0.5),
            (Vec3::ZERO, Vec3::X)
        );
    }
}
