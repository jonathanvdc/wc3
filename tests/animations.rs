use wc3_mdx::{AnimationTrack, Keyframe, Node, TrackValueKind};

#[test]
fn node_transform_tracks_round_trip() {
    let track = AnimationTrack {
        tag: *b"KGTR",
        interpolation: 2,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 100,
            value: vec![1.0, 2.0, 3.0],
            in_tangent: Some(vec![0.0, 0.0, 0.0]),
            out_tangent: Some(vec![4.0, 5.0, 6.0]),
        }],
    };
    let mut node = Node::new("Animated", 4).unwrap();
    node.set_tracks(std::slice::from_ref(&track)).unwrap();
    let parsed = Node::from_bytes(node.as_bytes()).unwrap();
    assert_eq!(parsed.tracks().unwrap(), vec![track]);
}

#[test]
fn standalone_integer_track_preserves_values() {
    let mut key = Keyframe {
        frame: 10,
        value: vec![0.0],
        in_tangent: None,
        out_tangent: None,
    };
    key.set_integer_value(u32::MAX);
    let track = AnimationTrack {
        tag: *b"KMTF",
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![key],
    };
    assert_eq!(track.component_count(), Some(1));
    assert_eq!(track.value_kind(), Some(TrackValueKind::Integer));
    let bytes = track.to_bytes().unwrap();
    let parsed = AnimationTrack::from_bytes(&bytes).unwrap();
    assert_eq!(parsed.keyframes[0].integer_value(), Some(u32::MAX));
    assert!(AnimationTrack::from_bytes(&[bytes, vec![0]].concat()).is_err());
}
