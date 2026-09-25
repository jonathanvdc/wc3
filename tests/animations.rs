use wc3_mdx::{AnimationTrack, Keyframe, Node};

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
    node.set_tracks(&[track.clone()]).unwrap();
    let parsed = Node::from_bytes(node.as_bytes()).unwrap();
    assert_eq!(parsed.tracks().unwrap(), vec![track]);
}
