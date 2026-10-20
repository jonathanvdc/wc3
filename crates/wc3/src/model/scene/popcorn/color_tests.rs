use super::*;
fn near(actual: [f32; 4], expected: [f32; 4]) {
    for (a, b) in actual.into_iter().zip(expected) {
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
        near(convert(rgb, false).unwrap(), hsv);
        near(convert(hsv, true).unwrap(), rgb);
    }
}
#[test]
fn hsv_wraps_hue_clamps_saturation_and_preserves_hdr() {
    near(
        convert([-2. / 3., 2., 3., 4.], true).unwrap(),
        [0., 3., 0., 4.],
    );
    near(
        convert([7. / 3., 1., 3., 4.], true).unwrap(),
        [0., 3., 0., 4.],
    );
    near(convert([0., -1., 3., 4.], true).unwrap(), [3., 3., 3., 4.]);
    near(convert([0., 1., -1., 4.], true).unwrap(), [0., 0., 0., 4.]);
}
#[test]
fn color_conversion_rejects_nonfinite_results_instead_of_sanitizing() {
    assert!(convert([f32::NAN, 1., 1., 1.], true).is_err());
    assert!(convert([0., 0., 0., f32::INFINITY], false).is_err());
    assert!(convert([-1., 0., 0., 1.], false).is_err());
}
