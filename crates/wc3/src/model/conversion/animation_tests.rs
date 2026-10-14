use super::*;
#[test]
fn shortened_splines_keep_values_between_new_boundaries() {
    for interpolation in [Interpolation::Hermite, Interpolation::Bezier] {
        let track = RawTrack {
            rotation: false,
            interpolation,
            keys: vec![
                Key {
                    time: -1.,
                    value: vec![0., 1., 2.],
                    incoming: Some(vec![0.; 3]),
                    outgoing: Some(vec![6., -2., 1.]),
                },
                Key {
                    time: 2.,
                    value: vec![3., 4., 5.],
                    incoming: Some(vec![-3., 2., 4.]),
                    outgoing: Some(vec![0.; 3]),
                },
            ],
        };
        let cropped = crop_raw(&track, 0., 1.).unwrap();
        assert_eq!(
            cropped.keys.iter().map(|k| k.time).collect::<Vec<_>>(),
            [0., 1.]
        );
        for i in 0..101 {
            let t = i as f64 / 100.;
            let a = sample(&track, t).unwrap();
            let b = sample(&cropped, t).unwrap();
            for (a, b) in a.iter().zip(b) {
                assert!((a - b).abs() < 0.000002);
            }
        }
    }
}
#[test]
fn spherical_restriction_preserves_rotation_and_clamped_intervals() {
    let track = RawTrack {
        rotation: true,
        interpolation: Interpolation::Linear,
        keys: vec![
            Key {
                time: -1.,
                value: vec![0., 0., 0., 1.],
                incoming: None,
                outgoing: None,
            },
            Key {
                time: 2.,
                value: vec![0., 0., 1., 0.],
                incoming: None,
                outgoing: None,
            },
        ],
    };
    let cropped = crop_raw(&track, 0., 1.).unwrap();
    for i in 0..101 {
        let t = i as f64 / 100.;
        for (a, b) in sample(&track, t)
            .unwrap()
            .iter()
            .zip(sample(&cropped, t).unwrap())
        {
            assert!((a - b).abs() < 0.000002);
        }
    }
    let clamped = crop_raw(&track, 3., 4.).unwrap();
    assert_eq!(clamped.keys[0].value, track.keys[1].value);
}

#[test]
fn typed_restriction_retains_clock_and_target() {
    use crate::model::semantic::{Clock, OwnerId, Property};
    let track = Track {
        target: Property::named(OwnerId::Model, "opacity"),
        curve: Curve::from_samples(
            [Time::seconds(-1.).unwrap(), Time::seconds(2.).unwrap()],
            [0f32, 3.],
            Interpolation::Linear,
        )
        .unwrap(),
        clock: Clock::Clip,
    };
    let result = crop(
        &track,
        Time::seconds(0.).unwrap(),
        Time::seconds(1.).unwrap(),
    )
    .unwrap();
    assert_eq!(result.target, track.target);
    assert_eq!(result.clock, track.clock);
    assert_eq!(result.curve.values().to_vec(), [1., 2.]);
}
