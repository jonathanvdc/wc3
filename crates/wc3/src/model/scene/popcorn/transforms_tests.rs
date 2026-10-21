use super::*;
use crate::model::scene::popcorn::payload::snapshot;
use glam::Quat;
#[test]
fn forward_inverse_roundtrip_uses_identical_selected_frames_including_nonuniform_scale() {
    let ordinary = Mat4::from_scale_rotation_translation(
        Vec3::new(-2., 3., 0.25),
        Quat::from_rotation_z(0.7) * Quat::from_rotation_x(0.3),
        Vec3::new(12., -4., 9.),
    );
    let position = Vec3::new(-20., 30., 5.);
    let orientation = Quat::from_rotation_y(-0.8);
    let value = Vec3::new(2., -3., 4.);
    for payload in [
        None,
        snapshot(Some(position), None).unwrap(),
        snapshot(None, Some(orientation)).unwrap(),
        snapshot(Some(position), Some(orientation)).unwrap(),
    ] {
        for direction in [false, true] {
            let world = forward(
                ordinary,
                payload,
                value,
                if direction { 18 } else { 19 },
                direction,
            )
            .unwrap();
            let restored = inverse(
                ordinary,
                payload,
                world,
                if direction { 66 } else { 67 },
                direction,
            )
            .unwrap();
            assert!(
                (restored - value).length() < 1e-4,
                "{direction} {payload:?} {restored:?}"
            );
        }
    }
    let world = forward(ordinary, None, value, 19, false).unwrap();
    let transposed_wrong = (ordinary.transpose() * world.extend(1.)).truncate();
    assert!((transposed_wrong - value).length() > 1.);
}
#[test]
fn inverse_rejects_singular_nonaffine_nonfinite_and_wrong_direction_masks() {
    let singular = Mat4::from_scale(Vec3::new(1., 0., 1.));
    assert!(inverse(singular, None, Vec3::X, 67, false).is_err());
    assert!(inverse(
        Mat4::IDENTITY,
        None,
        Vec3::new(f32::INFINITY, 0., 0.),
        67,
        false
    )
    .is_err());
    assert!(inverse(Mat4::IDENTITY, None, Vec3::X, 66, false).is_err());
    assert!(inverse(Mat4::IDENTITY, None, Vec3::X, 67, true).is_err());
    let mut projective = Mat4::IDENTITY;
    projective.x_axis.w = 1.;
    assert!(inverse(projective, None, Vec3::X, 67, false).is_err());
}
#[test]
fn forward_filters_separate_translation_basis_and_ordinary_space_from_payload_space() {
    let ordinary = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 3., 4.),
        Quat::from_rotation_z(0.5),
        Vec3::new(10., 20., 30.),
    );
    let value = Vec3::new(1., 2., 3.);
    assert_eq!(
        forward(ordinary, None, value, 17, false).unwrap(),
        value + Vec3::new(10., 20., 30.)
    );
    let payload = snapshot(Some(Vec3::X * 2.), None).unwrap();
    assert_eq!(
        forward(ordinary, payload, value, 19, false).unwrap(),
        value + Vec3::X * 2.
    );
    assert_eq!(
        forward(ordinary, payload, value, 11, false).unwrap(),
        ordinary.transform_point3(value)
    );
}

#[test]
fn unavailable_position_survives_supported_transforms_without_hiding_invalid_frames() {
    let missing = Vec3::splat(f32::INFINITY);
    let matrix = Mat4::from_scale_rotation_translation(
        Vec3::new(-2., 3., 4.),
        Quat::from_rotation_x(0.7),
        Vec3::splat(10.),
    );
    for (direction, forward_mask, inverse_mask) in [(false, 19, 67), (true, 18, 66)] {
        assert_eq!(
            forward(matrix, None, missing, forward_mask, direction).unwrap(),
            missing
        );
        assert_eq!(
            inverse(matrix, None, missing, inverse_mask, direction).unwrap(),
            missing
        );
        assert!(inverse(Mat4::ZERO, None, missing, inverse_mask, direction).is_err());
        assert!(inverse(matrix, None, Vec3::NAN, inverse_mask, direction).is_err());
    }
    assert!(forward(Mat4::ZERO, None, missing, 19, false).is_err());
    assert!(forward(matrix, None, missing, 67, false).is_err());
    assert!(inverse(matrix, None, missing, 19, false).is_err());
}

#[test]
fn unavailable_camera_does_not_admit_singular_frames_or_numeric_overflow() {
    let missing = Vec3::splat(f32::INFINITY);
    let singular = Mat4::from_scale(Vec3::new(1., 0., 1.));
    assert!(forward(singular, None, missing, 19, false).is_err());
    assert!(inverse(singular, None, missing, 67, false).is_err());
    assert!(forward(
        Mat4::from_scale(Vec3::splat(2.)),
        None,
        Vec3::splat(f32::MAX),
        19,
        false
    )
    .is_err());
    assert!(inverse(
        Mat4::from_scale(Vec3::splat(0.5)),
        None,
        Vec3::splat(f32::MAX),
        67,
        false
    )
    .is_err());
}
