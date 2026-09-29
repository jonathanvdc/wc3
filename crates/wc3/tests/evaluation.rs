use std::ops::RangeInclusive;
use wc3::model::animation::{Animatable, Interpolate, TangentKeyframe, Track, ValueKeyframe};
use wc3::model::Quaternion;

fn key(frame: i32, value: f32) -> ValueKeyframe<f32> {
    ValueKeyframe { frame, value }
}
fn close(a: f32, b: f32) {
    assert!((a - b).abs() < 0.00001, "{a} != {b}");
}

#[test]
fn boundaries_fractional_time_and_unordered_duplicates() {
    let track = Track::linear(vec![key(10, 10.0), key(-10, -10.0), key(10, 20.0)], None).unwrap();
    assert_eq!(track.evaluate(-100.0), Some(-10.0));
    assert_eq!(track.evaluate(100.0), Some(20.0));
    assert_eq!(track.evaluate(10.0), Some(20.0));
    assert_eq!(track.evaluate(0.0), Some(5.0));
    assert_eq!(track.evaluate(-9.5), Some(-9.25));
    assert_eq!(track.evaluate(f64::NAN), None);
    assert_eq!(track.evaluate(f64::INFINITY), None);
    let step = Track::step(vec![key(0, 1.0), key(10, 2.0)], None).unwrap();
    assert_eq!(step.evaluate(9.9), Some(1.0));
    assert_eq!(step.evaluate(10.0), Some(2.0));
}

#[test]
fn intervals_and_base_fallback() {
    let empty = Track::<f32>::linear(vec![], None).unwrap();
    assert_eq!(empty.evaluate(0.0), None);
    assert_eq!(Animatable::Animated(empty.clone()).evaluate(0.0), None);
    assert_eq!(
        Animatable::Both {
            value: 3.0,
            track: empty
        }
        .evaluate(0.0),
        Some(3.0)
    );
    assert_eq!(Animatable::Static(4.0).evaluate(f64::NAN), Some(4.0));
    let track = Track::linear(vec![key(0, 0.0), key(10, 10.0), key(100, 100.0)], Some(0)).unwrap();
    assert_eq!(track.evaluate_in(50.0, 0..=10), Some(10.0));
    assert_eq!(track.evaluate_in(50.0, 20..=90), None);
    let backwards_interval = RangeInclusive::new(10, 0);
    assert_eq!(track.evaluate_in(0.0, backwards_interval), None);
    let both = Animatable::Both { value: 7.0, track };
    assert_eq!(both.evaluate_in(50.0, 20..=90), Some(7.0));
    assert_eq!(both.evaluate(5.0), Some(5.0));
    assert_eq!(Track::constant(2.0).evaluate(-50.0), Some(2.0));
}

#[test]
fn splines_use_left_outgoing_and_right_incoming_tangents() {
    let keys = vec![
        TangentKeyframe {
            frame: 0,
            value: 0.0,
            in_tangent: 999.0,
            out_tangent: 4.0,
        },
        TangentKeyframe {
            frame: 100,
            value: 8.0,
            in_tangent: 0.0,
            out_tangent: 999.0,
        },
    ];
    close(
        Track::hermite(keys.clone(), None)
            .unwrap()
            .evaluate(50.0)
            .unwrap(),
        4.5,
    );
    close(
        Track::bezier(keys, None).unwrap().evaluate(50.0).unwrap(),
        2.5,
    );
    assert_eq!(
        <[f32; 3]>::linear([0.0; 3], [2.0, 4.0, 6.0], 0.5),
        [1.0, 2.0, 3.0]
    );
}

#[test]
fn integer_tracks_are_discrete_and_rotation_uses_shortest_path() {
    let track = Track::linear(
        vec![
            ValueKeyframe {
                frame: 0,
                value: 3u32,
            },
            ValueKeyframe {
                frame: 10,
                value: 8,
            },
        ],
        None,
    )
    .unwrap();
    assert_eq!(track.evaluate(9.0), Some(3));
    assert_eq!(track.evaluate(10.0), Some(8));
    let identity = [0.0, 0.0, 0.0, 1.0];
    let opposite = [0.0, 0.0, 0.0, -1.0];
    assert_eq!(Quaternion::linear(identity, opposite, 0.5), identity);
    let halfway = Quaternion::linear(identity, [0.0, 0.0, 1.0, 0.0], 0.5);
    close(halfway[2], 0.5f32.sqrt());
    close(halfway[3], 0.5f32.sqrt());
    for value in [
        Quaternion::hermite(identity, identity, identity, identity, 0.3),
        Quaternion::bezier(identity, identity, identity, identity, 0.3),
    ] {
        assert_eq!(value, identity);
    }
}
