use mdl::{Read as _, Write as _};
use mdx::{Read as _, Write as _};
use wc3::model::animation::{AnimationTime, Sequence};
use wc3::model::scene::{EventObject, Node};
use wc3::model::{mdl, mdx};

fn event(frames: &[i32], global: u32) -> EventObject {
    EventObject::new(Node::new("Arbitrary", 0).unwrap(), global, frames)
}

fn crossed(
    sequence: Option<&Sequence>,
    durations: &[u32],
    event: &EventObject,
    from: f64,
    to: f64,
    include: bool,
) -> Vec<(usize, i32, f64)> {
    AnimationTime {
        sequence,
        elapsed_ms: to,
        global_sequences: durations,
    }
    .event_occurrences(event, from, include)
    .into_iter()
    .map(|key| (key.key_index, key.frame, key.elapsed_ms))
    .collect()
}

#[test]
fn every_construction_and_replacement_path_sorts_and_retains_duplicates() {
    let mut value = event(&[30, -10, 20, 20], u32::MAX);
    assert_eq!(value.frames(), [-10, 20, 20, 30]);
    value.set_frames(&[100, 0, -5, 0]);
    assert_eq!(value.frames(), [-5, 0, 0, 100]);
    let parsed = EventObject::decode_mdl(
        "EventObject \"Arbitrary\" { ObjectId 0, EventTrack 4 { 30, -10, 20, 20, } }",
    )
    .unwrap();
    assert_eq!(parsed.frames(), [-10, 20, 20, 30]);
    // Rewrite independent wire timestamps to verify sorting by the MDX reader.
    let mut bytes = value.encode_mdx().unwrap();
    let tail = bytes.len() - 16;
    for (chunk, frame) in bytes[tail..]
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip([30i32, -10, 20, 20])
    {
        chunk.copy_from_slice(&frame.to_le_bytes());
    }
    assert_eq!(
        EventObject::decode_mdx(&bytes).unwrap().frames(),
        [-10, 20, 20, 30]
    );
    assert_eq!(
        EventObject::decode_mdx(&parsed.encode_mdx().unwrap()).unwrap(),
        parsed
    );
    assert_eq!(
        EventObject::decode_mdl(&parsed.encode_mdl().unwrap()).unwrap(),
        parsed
    );
}

#[test]
fn traversal_enumerates_multiple_loops_duplicates_and_both_boundary_keys() {
    let sequence = Sequence::new("Loop", [100, 200]).unwrap();
    let value = event(&[-10, 99, 100, 125, 125, 200, 201], u32::MAX);
    assert_eq!(
        crossed(Some(&sequence), &[], &value, 0.0, 225.0, true),
        [
            (2, 100, 0.0),
            (3, 125, 25.0),
            (4, 125, 25.0),
            (2, 100, 100.0),
            (5, 200, 100.0),
            (3, 125, 125.0),
            (4, 125, 125.0),
            (2, 100, 200.0),
            (5, 200, 200.0),
            (3, 125, 225.0),
            (4, 125, 225.0)
        ]
    );
    assert_eq!(
        crossed(Some(&sequence), &[], &value, 25.0, 25.0, true).len(),
        2
    );
    assert!(crossed(Some(&sequence), &[], &value, 25.0, 25.0, false).is_empty());
    let mut split = crossed(Some(&sequence), &[], &value, 0.0, 100.0, true);
    split.extend(crossed(Some(&sequence), &[], &value, 100.0, 225.0, false));
    assert_eq!(
        split,
        crossed(Some(&sequence), &[], &value, 0.0, 225.0, true)
    );
}

#[test]
fn nonlooping_sequences_stop_but_global_tracks_keep_their_own_period() {
    let mut sequence = Sequence::new("Once", [100, 200]).unwrap();
    sequence.flags.set_non_looping(true);
    assert_eq!(
        crossed(
            Some(&sequence),
            &[],
            &event(&[100, 125, 200], u32::MAX),
            0.0,
            400.0,
            true
        ),
        [(0, 100, 0.0), (1, 125, 25.0), (2, 200, 100.0)]
    );
    let global = event(&[-1, 0, 25, 50, 51], 0);
    assert_eq!(
        crossed(Some(&sequence), &[50], &global, 100.0, 175.0, false),
        [
            (2, 25, 125.0),
            (1, 0, 150.0),
            (3, 50, 150.0),
            (2, 25, 175.0)
        ]
    );
    assert_eq!(
        crossed(None, &[50], &global, 0.0, 25.0, true),
        [(1, 0, 0.0), (2, 25, 25.0)]
    );
}

#[test]
fn zero_duration_missing_and_invalid_clocks_are_bounded() {
    let sequence = Sequence::new("Still", [100, 100]).unwrap();
    let value = event(&[99, 100, 100, 101], u32::MAX);
    assert_eq!(
        crossed(Some(&sequence), &[], &value, 0.0, 100.0, true),
        [(1, 100, 0.0), (2, 100, 0.0)]
    );
    assert!(crossed(Some(&sequence), &[], &value, 0.0, 100.0, false).is_empty());
    assert_eq!(
        crossed(None, &[0], &event(&[-1, 0, 1], 0), 0.0, 100.0, true),
        [(1, 0, 0.0)]
    );
    assert!(crossed(None, &[], &value, 0.0, 100.0, true).is_empty());
    assert!(crossed(None, &[100], &event(&[10], 1), 0.0, 100.0, true).is_empty());
    let invalid = Sequence::new("Invalid", [200, 100]).unwrap();
    assert!(crossed(Some(&invalid), &[], &value, 0.0, 100.0, true).is_empty());
    for (from, to) in [
        (100.0, 0.0),
        (0.0, f64::NAN),
        (f64::INFINITY, 100.0),
        (-20.0, -1.0),
    ] {
        assert!(crossed(Some(&sequence), &[], &value, from, to, true).is_empty());
    }
    assert_eq!(
        crossed(Some(&sequence), &[], &value, -10.0, 0.0, false).len(),
        2
    );
}
