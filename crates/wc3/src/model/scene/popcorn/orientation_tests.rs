use super::*;
use std::f32::consts::FRAC_PI_2;
#[test]
fn shortest_arc_points_plus_z_forward_with_explicit_antiparallel_and_zero_cases() {
    for axis in [
        Vec3::X,
        Vec3::Y,
        Vec3::Z,
        -Vec3::Z,
        Vec3::new(1., 2., 3.).normalize(),
    ] {
        let q = from_forward(axis).unwrap();
        assert!((q * Vec3::Z - axis).length() < 1e-5);
        assert!((q.length() - 1.).abs() < 1e-6)
    }
    assert_eq!(
        from_forward(-Vec3::Z).unwrap(),
        Quat::from_xyzw(1., 0., 0., 0.)
    );
    assert_eq!(from_forward(Vec3::ZERO).unwrap(), Quat::IDENTITY);
    let nonunit = Vec3::new(1., 0., 1.);
    let raw = from_forward(nonunit).unwrap();
    let unit = from_forward(nonunit.normalize()).unwrap();
    assert!((raw * Vec3::Z - unit * Vec3::Z).length() > 0.05);
}
#[test]
fn quaternion_rotation_uses_authored_components_and_axis_rotation_normalizes_axis() {
    let q = Quat::from_rotation_z(FRAC_PI_2);
    assert!((rotate(Vec3::X, q).unwrap() - Vec3::Y).length() < 1e-6);
    let nonunit = Quat::from_xyzw(0., 0., 2., 2.);
    assert!((rotate(Vec3::X, nonunit).unwrap() - Vec3::new(-7., 8., 0.)).length() < 1e-6);
    assert!((rotate_about(Vec3::X, Vec3::Z * 7., FRAC_PI_2).unwrap() - Vec3::Y).length() < 1e-6);
    assert_eq!(
        rotate(Vec3::new(1., 2., 3.), Quat::from_xyzw(0., 0., 0., 2.)).unwrap(),
        Vec3::new(1., 2., 3.)
    );
    assert_eq!(
        rotate(Vec3::Y, Quat::from_xyzw(1., 0., 0., 1.)).unwrap(),
        Vec3::new(0., -1., 2.)
    );
    assert!(rotate_about(Vec3::X, Vec3::ZERO, 1.).is_err());
    assert!(from_forward(Vec3::NAN).is_err());
}
#[test]
fn emitter_axes_read_scene_basis_columns_without_translation_and_normalize_scale() {
    let world = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 3., 4.),
        Quat::from_rotation_z(FRAC_PI_2),
        Vec3::splat(100.),
    );
    assert!((emitter_axis(world, false).unwrap() - Vec3::Y).length() < 1e-6);
    assert!((emitter_axis(world, true).unwrap() - Vec3::Z).length() < 1e-6);
    assert_eq!(emitter_axis(Mat4::ZERO, false).unwrap(), Vec3::X);
    assert_eq!(emitter_axis(Mat4::ZERO, true).unwrap(), Vec3::Z);
}
