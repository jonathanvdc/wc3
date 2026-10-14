use super::*;
use crate::model::semantic::Quaternion;

#[test]
fn native_sampling_directives_cannot_bypass_value_capabilities() {
    for mode in [
        Interpolation::Linear,
        Interpolation::Slerp,
        Interpolation::Nlerp,
    ] {
        assert!(Curve::from_samples([Time::default()], [true], mode).is_err());
    }
    for mode in [Interpolation::Hermite, Interpolation::Bezier] {
        assert!(
            Curve::from_spline_samples([Time::default()], [true], [false], [false], mode).is_err()
        );
    }
    assert!(Curve::from_samples([Time::default()], [1f32], Interpolation::Slerp).is_err());
    assert!(Curve::from_samples([Time::default()], [1f32], Interpolation::Tcb).is_err());
    assert!(Curve::from_spline_samples(
        [Time::default()],
        [1f32],
        [0.],
        [0.],
        Interpolation::Linear
    )
    .is_err());
}

#[test]
fn native_quaternion_controls_round_trip_without_rotation_validation() {
    let controls = [Quaternion([0.; 4])];
    for mode in [Interpolation::Hermite, Interpolation::Bezier] {
        let curve = Curve::from_spline_samples(
            [Time::default()],
            [Quaternion::IDENTITY],
            controls,
            controls,
            mode,
        )
        .unwrap();
        assert_eq!(curve.incoming_value(0), Some(controls[0]));
        assert_eq!(curve.outgoing_value(0), Some(controls[0]));
        assert_eq!(curve.incoming_value(1), None);
        assert_eq!(curve.outgoing_value(1), None);
        assert_eq!(curve.interpolation(), mode);
    }
}
