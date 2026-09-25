use wc3_mdx::Record;
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
    let parsed = Node::decode(&node.encode().unwrap(), 800).unwrap();
    assert_eq!(parsed.tracks(), vec![track]);
}

#[test]
fn node_keeps_name_padding_and_track_order() {
    let mut node = Node::new("N", 2).unwrap();
    let tracks = [*b"KGSC", *b"KGTR"].map(|tag| AnimationTrack {
        tag,
        interpolation: 1,
        global_sequence_id: u32::MAX,
        keyframes: vec![Keyframe {
            frame: 7,
            value: vec![1.0, 2.0, 3.0],
            in_tangent: None,
            out_tangent: None,
        }],
    });
    node.set_tracks(&tracks).unwrap();
    let mut bytes = node.encode().unwrap();
    bytes[20] = 0xe1;
    let parsed = Node::decode(&bytes, 800).unwrap();
    assert_eq!(parsed.encode().unwrap(), bytes);
    assert_eq!(parsed.tracks()[0].tag, *b"KGSC");
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
    let bytes = track.encode().unwrap();
    let parsed = AnimationTrack::decode(&bytes, 800).unwrap();
    assert_eq!(parsed.keyframes[0].integer_value(), Some(u32::MAX));
    assert!(AnimationTrack::decode(&[bytes, vec![0]].concat(), 800).is_err());
}

#[test]
fn setter_does_not_serialize_tracks_to_validate_them() {
    let mut node = Node::new("Animated", 4).unwrap();
    let track = AnimationTrack {
        tag: *b"KGTR",
        interpolation: 4,
        global_sequence_id: u32::MAX,
        keyframes: Vec::new(),
    };

    node.set_tracks(&[track]).unwrap();
    assert!(node.encode().is_err());
}
