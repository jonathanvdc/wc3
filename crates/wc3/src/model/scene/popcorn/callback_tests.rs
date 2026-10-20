use super::*;
fn color(channels: [f32; 4]) -> Value {
    Value::from_words(ValueType::Float4, channels.map(f32::to_bits))
}
fn near(actual: Value, expected: [f32; 4]) {
    for (a, b) in actual.words().map(f32::from_bits).into_iter().zip(expected) {
        assert!((a - b).abs() < 2e-6, "{a} != {b}");
    }
}
#[test]
fn primary_color_hues_hdr_and_alpha_roundtrip() {
    for (rgb, hsv) in [
        ([1., 0., 0., 2.5], [0., 1., 1., 2.5]),
        ([0., 1., 0., 2.5], [1. / 3., 1., 1., 2.5]),
        ([0., 0., 1., 2.5], [2. / 3., 1., 1., 2.5]),
        ([2., 0., 0., 2.5], [0., 1., 2., 2.5]),
        ([0.25, 0.25, 0.25, 2.5], [0., 0., 0.25, 2.5]),
        ([0., 0., 0., 2.5], [0., 0., 0., 2.5]),
    ] {
        near(convert_color(color(rgb), false).unwrap(), hsv);
        near(convert_color(color(hsv), true).unwrap(), rgb);
    }
}
#[test]
fn hsv_wraps_hue_clamps_saturation_and_preserves_hdr() {
    near(
        convert_color(color([-2. / 3., 2., 3., 4.]), true).unwrap(),
        [0., 3., 0., 4.],
    );
    near(
        convert_color(color([7. / 3., 1., 3., 4.]), true).unwrap(),
        [0., 3., 0., 4.],
    );
    near(
        convert_color(color([0., -1., 3., 4.]), true).unwrap(),
        [3., 3., 3., 4.],
    );
    near(
        convert_color(color([0., 1., -1., 4.]), true).unwrap(),
        [0., 0., 0., 4.],
    );
}
#[test]
fn color_conversion_rejects_nonfinite_results_instead_of_sanitizing() {
    assert!(convert_color(color([f32::NAN, 1., 1., 1.]), true).is_err());
    assert!(convert_color(color([0., 0., 0., f32::INFINITY]), false).is_err());
    assert!(convert_color(color([-1., 0., 0., 1.]), false).is_err());
}
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
