use wc3_mdx::{AnimationTrack, Encodable, Error, Geoset, Node, ValueError};

#[test]
fn construction_and_mutation_report_value_errors() {
    let bad_name: Result<Node, ValueError> = Node::new("bad\0name", 1);
    assert_eq!(
        bad_name.unwrap_err(),
        ValueError::InvalidString { max_bytes: 79 }
    );

    let mut node = Node::new("Valid", 1).unwrap();
    let invalid_track = AnimationTrack {
        tag: *b"KATV",
        interpolation: 0,
        global_sequence_id: u32::MAX,
        keyframes: Vec::new(),
    };
    assert_eq!(
        node.set_tracks(&[invalid_track]),
        Err(ValueError::InvalidTrackTag {
            record: Node::TAG,
            track: *b"KATV",
        })
    );

    let mut geoset = Geoset::new(800, &[], &[], &[]).unwrap();
    assert_eq!(
        geoset.set_vertex(0, [0.0; 3]),
        Err(ValueError::IndexOutOfBounds {
            tag: *b"GEOS",
            index: 0,
            len: 0,
        })
    );

    let encoded: Result<Vec<u8>, Error> = node.encode();
    assert!(encoded.is_ok());
}
