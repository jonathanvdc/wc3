use super::*;
#[test]
fn shortened_splines_keep_values_between_new_boundaries() {
    for interpolation in [Interpolation::Hermite, Interpolation::Bezier] {
        let track = Track {
            layer: Default::default(),
            target: Target::Translation(0),
            interpolation,
            global_duration: None,
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
        let cropped = crop(&track, 0., 1.).unwrap();
        assert_eq!(
            cropped.keys.iter().map(|k| k.time).collect::<Vec<_>>(),
            [0., 1.]
        );
        for i in 0..101 {
            let t = i as f32 / 100.;
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
    let track = Track {
        layer: Default::default(),
        target: Target::Rotation(0),
        interpolation: Interpolation::Linear,
        global_duration: None,
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
    let cropped = crop(&track, 0., 1.).unwrap();
    for i in 0..101 {
        let t = i as f32 / 100.;
        for (a, b) in sample(&track, t)
            .unwrap()
            .iter()
            .zip(sample(&cropped, t).unwrap())
        {
            assert!((a - b).abs() < 0.000002);
        }
    }
    let clamped = crop(&track, 3., 4.).unwrap();
    assert_eq!(clamped.keys[0].value, track.keys[1].value);
}
