use wc3::model::animation::{TangentKeyframe, Track, ValueKeyframe};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::Read as _;

#[test]
fn constructors_stably_sort_all_interpolation_modes() {
    let keys = vec![
        ValueKeyframe {
            frame: 10,
            value: 1.0,
        },
        ValueKeyframe {
            frame: i32::MAX,
            value: 2.0,
        },
        ValueKeyframe {
            frame: 10,
            value: 3.0,
        },
        ValueKeyframe {
            frame: i32::MIN,
            value: 4.0,
        },
    ];
    let expected = vec![
        keys[3].clone(),
        keys[0].clone(),
        keys[2].clone(),
        keys[1].clone(),
    ];
    assert_eq!(
        Track::step(keys.clone(), None)
            .unwrap()
            .step_keys()
            .unwrap(),
        expected
    );
    assert_eq!(
        Track::linear(keys.clone(), None)
            .unwrap()
            .linear_keys()
            .unwrap(),
        expected
    );
    let tangent = |key: ValueKeyframe<f32>| TangentKeyframe {
        frame: key.frame,
        value: key.value,
        in_tangent: key.value + 10.0,
        out_tangent: key.value + 20.0,
    };
    let keys: Vec<_> = keys.into_iter().map(tangent).collect();
    let expected: Vec<_> = expected.into_iter().map(tangent).collect();
    assert_eq!(
        Track::hermite(keys.clone(), None)
            .unwrap()
            .hermite_keys()
            .unwrap(),
        expected
    );
    assert_eq!(
        Track::bezier(keys, None).unwrap().bezier_keys().unwrap(),
        expected
    );
}

#[test]
fn both_readers_sort_all_modes_without_losing_duplicate_tangents() {
    for (mode, name) in [
        (0u32, "DontInterp"),
        (1, "Linear"),
        (2, "Hermite"),
        (3, "Bezier"),
    ] {
        let mut bytes = Vec::new();
        for header in [4u32, mode, u32::MAX] {
            bytes.extend(header.to_le_bytes());
        }
        let mut text = format!("Track 4 {{ {name}, ");
        for (frame, value) in [(10i32, 1.0f32), (-10, 2.0), (10, 3.0), (-10, 4.0)] {
            bytes.extend(frame.to_le_bytes());
            bytes.extend(value.to_le_bytes());
            text.push_str(&format!("{frame}: {value}, "));
            if mode >= 2 {
                bytes.extend((value + 10.0).to_le_bytes());
                bytes.extend((value + 20.0).to_le_bytes());
                text.push_str(&format!(
                    "InTan {}, OutTan {}, ",
                    value + 10.0,
                    value + 20.0
                ));
            }
        }
        text.push('}');
        let binary = Track::<f32>::decode_mdx(&bytes).unwrap();
        let textual = Track::<f32>::decode_mdl(&text).unwrap();
        assert_eq!(binary, textual);
        let values: Vec<_> = if mode < 2 {
            binary
                .step_keys()
                .or_else(|| binary.linear_keys())
                .unwrap()
                .iter()
                .map(|key| (key.frame, key.value))
                .collect()
        } else {
            let keys = binary
                .hermite_keys()
                .or_else(|| binary.bezier_keys())
                .unwrap();
            for key in keys {
                assert_eq!(key.in_tangent, key.value + 10.0);
                assert_eq!(key.out_tangent, key.value + 20.0);
            }
            keys.iter().map(|key| (key.frame, key.value)).collect()
        };
        assert_eq!(values, [(-10, 2.0), (-10, 4.0), (10, 1.0), (10, 3.0)]);
        assert_eq!(binary.evaluate(-100.0), Some(4.0));
        assert_eq!(binary.evaluate(-10.0), Some(4.0));
        assert_eq!(binary.evaluate(10.0), Some(3.0));
        assert_eq!(binary.evaluate(100.0), Some(3.0));
        assert_eq!(binary.evaluate_in(0.0, -10..=-10), Some(4.0));
        assert_eq!(binary.evaluate_in(0.0, 10..=10), Some(3.0));
        let expected = match mode {
            0 => 4.0,
            1 => 3.5,
            2 => 4.875,
            3 => 14.75,
            _ => unreachable!(),
        };
        assert_eq!(binary.evaluate(0.0), Some(expected));
        assert_eq!(
            binary.key_frame_at_or_before_in(100.0, -10..=-10),
            Some(-10)
        );
        assert_eq!(binary.key_frame_at_or_before_in(0.0, 10..=10), None);
        assert_eq!(binary.key_frame_at_or_before(f64::INFINITY), None);
        let canonical = binary.encode_mdl().unwrap();
        assert!(canonical.find("-10:").unwrap() < canonical.find("10: 1").unwrap());
    }
}
