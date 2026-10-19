use super::*;
use crate::model::common::Borrowed as _;
use crate::model::Quat;

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
    let controls = [Quat::from_array([0.; 4])];
    for mode in [Interpolation::Hermite, Interpolation::Bezier] {
        let curve = Curve::from_spline_samples(
            [Time::default()],
            [Quat::IDENTITY],
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

#[test]
fn packed_curve_arrays_borrow_and_detach_without_changing_keys() {
    use crate::model::common::PackedArray;
    let times =
        PackedArray::alloc_from_slice(&[Time::seconds(-0.5).unwrap(), Time::seconds(1.).unwrap()]);
    let values = PackedArray::alloc_from_slice(&[2f32, 4.]);
    let curve = Curve::linear(times.as_borrowed(), values.as_borrowed()).unwrap();
    assert_eq!(curve.times().as_bytes().as_ptr(), times.as_bytes().as_ptr());
    assert_eq!(
        curve.values().as_bytes().as_ptr(),
        values.as_bytes().as_ptr()
    );
    assert_eq!(curve.values().clone().into_owned().to_vec(), vec![2., 4.]);
}

#[test]
fn packed_time_cannot_bypass_finite_curve_validation() {
    use crate::model::common::PackedArray;
    let bytes = f64::NAN.to_le_bytes();
    let times = PackedArray::<Time>::parse_bytes(&bytes).unwrap();
    assert!(Curve::step(times, [1f32]).is_err());
}
