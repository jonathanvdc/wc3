use wc3::model::animation::{Animatable, AnimationTime, Sequence, Track, ValueKeyframe};

fn ramp(global: Option<u32>) -> Track<f32> {
    Track::linear(
        vec![
            ValueKeyframe {
                frame: 100,
                value: 0.0,
            },
            ValueKeyframe {
                frame: 200,
                value: 1.0,
            },
            ValueKeyframe {
                frame: 300,
                value: 9.0,
            },
        ],
        global,
    )
    .unwrap()
}

fn time(sequence: &Sequence, elapsed_ms: f64) -> AnimationTime<'_> {
    AnimationTime {
        sequence: Some(sequence),
        elapsed_ms,
        global_sequences: &[],
    }
}

#[test]
fn model_clock_wraps_clamps_and_filters_sequence_keys() {
    let mut sequence = Sequence::new("Stand", [100, 200]).unwrap();
    let track = ramp(None);
    assert_eq!(track.sample(&time(&sequence, 150.0)), Some(0.5));
    assert_eq!(track.sample(&time(&sequence, -25.0)), Some(0.75));
    assert_eq!(track.sample(&time(&sequence, 100.0)), Some(0.0));
    sequence.flags.set_non_looping(true);
    assert_eq!(track.sample(&time(&sequence, 150.0)), Some(1.0));
    assert_eq!(track.sample(&time(&sequence, -25.0)), Some(0.0));
    sequence.interval = [100, 100];
    assert_eq!(track.sample(&time(&sequence, 150.0)), Some(0.0));
    sequence.interval = [400, 500];
    assert_eq!(track.sample(&time(&sequence, 50.0)), None);
}

#[test]
fn global_clock_needs_no_model_sequence_and_handles_missing_and_zero_durations() {
    let time = AnimationTime {
        sequence: None,
        elapsed_ms: 450.0,
        global_sequences: &[300, 0],
    };
    assert_eq!(ramp(Some(0)).sample(&time), Some(0.5));
    assert_eq!(ramp(Some(1)).sample(&time), Some(0.0));
    assert_eq!(ramp(Some(2)).sample(&time), None);
    assert_eq!(ramp(None).sample(&time), None);
    let negative = AnimationTime {
        elapsed_ms: -150.0,
        ..time
    };
    assert_eq!(ramp(Some(0)).sample(&negative), Some(0.5));
}

#[test]
fn properties_prefer_animation_then_base_and_leave_defaults_to_callers() {
    let sequence = Sequence::new("Stand", [100, 200]).unwrap();
    let time = AnimationTime {
        sequence: Some(&sequence),
        elapsed_ms: 50.0,
        global_sequences: &[],
    };
    let both = Animatable::Both {
        value: 0.8,
        track: ramp(None),
    };
    assert_eq!(both.sample(&time), Some(0.5));
    let missing = AnimationTime {
        sequence: None,
        ..time
    };
    assert_eq!(both.sample(&missing), Some(0.8));
    assert_eq!(Animatable::Static(0.7).sample(&missing), Some(0.7));
    assert_eq!(Animatable::Animated(ramp(None)).sample(&missing), None);
    assert_eq!(
        Animatable::Animated(ramp(None))
            .sample(&missing)
            .unwrap_or(1.0),
        1.0
    );
    let empty = Track::linear(vec![], None).unwrap();
    assert_eq!(
        Animatable::Both {
            value: 0.9f32,
            track: empty
        }
        .sample(&time),
        Some(0.9)
    );
}
