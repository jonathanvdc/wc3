use super::*;
#[test]
fn transform_filters_separate_translation_point_basis_and_direction_basis() {
    let world = Mat4::from_scale_rotation_translation(
        Vec3::new(2., 3., 4.),
        glam::Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
        Vec3::new(10., 20., 30.),
    );
    let value = Vec3::new(1., 2., 3.);
    let t = transform(world, value, 17, false).unwrap();
    let tq = transform(world, value, 19, false).unwrap();
    let q = transform(world, value, 18, true).unwrap();
    assert!((t - Vec3::new(11., 22., 33.)).length() < 1e-5);
    assert!((tq - Vec3::new(4., 22., 42.)).length() < 1e-5);
    assert!((q - Vec3::new(-6., 2., 12.)).length() < 1e-5);
    assert!(transform(world, value, 18, false).is_err());
    assert!(transform(world, value, 17, true).is_err());
    assert!(transform(world, value, 67, false).is_err());
}
