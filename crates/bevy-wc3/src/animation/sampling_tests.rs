use super::*;
use wc3::model::animation::{Sequence, ValueKeyframe};
#[test]
fn sequence_clock_loops_and_global_clock_runs_independently() {
    let animation = Wc3Animation {
        sequence: 0,
        elapsed_ms: 150.0,
        speed: 1.0,
        playing: true,
        sequences: vec![Sequence::new("Stand", [1000, 1100]).unwrap()],
        global_sequences: vec![200],
        event_playback: Default::default(),
    };
    let sequence_track = Track::linear(
        vec![
            ValueKeyframe {
                frame: 1000,
                value: 0.0f32,
            },
            ValueKeyframe {
                frame: 1100,
                value: 1.0f32,
            },
        ],
        None,
    )
    .unwrap();
    let global_track = Track::linear(
        vec![
            ValueKeyframe {
                frame: 0,
                value: 0.0f32,
            },
            ValueKeyframe {
                frame: 200,
                value: 1.0f32,
            },
        ],
        Some(0),
    )
    .unwrap();
    assert_eq!(sample(&sequence_track, &animation), Some(0.5));
    assert_eq!(sample(&global_track, &animation), Some(0.75));
}

#[test]
fn color_tracks_use_sequence_and_global_clocks_with_neutral_fallback() {
    let mut animation = Wc3Animation {
        sequence: 0,
        elapsed_ms: 50.0,
        speed: 1.0,
        playing: false,
        sequences: vec![
            Sequence::new("Stand", [0, 100]).unwrap(),
            Sequence::new("Other", [200, 300]).unwrap(),
        ],
        global_sequences: vec![100],
        event_playback: Default::default(),
    };
    let track = Track::linear(
        vec![
            ValueKeyframe {
                frame: 0,
                value: [1.0, 0.0, 0.0],
            },
            ValueKeyframe {
                frame: 100,
                value: [0.0, 0.0, 1.0],
            },
        ],
        None,
    )
    .unwrap();
    assert_eq!(sample(&track, &animation), Some([0.5, 0.0, 0.5]));
    animation.play(1);
    assert_eq!(sample(&track, &animation), None);
    let global = Track::linear(
        vec![
            ValueKeyframe {
                frame: 0,
                value: [1.0, 0.0, 0.0],
            },
            ValueKeyframe {
                frame: 100,
                value: [0.0, 0.0, 1.0],
            },
        ],
        Some(0),
    )
    .unwrap();
    animation.elapsed_ms = 150.0;
    assert_eq!(sample(&global, &animation), Some([0.5, 0.0, 0.5]));
}
