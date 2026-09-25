use wc3_mdx::Record;
use wc3_mdx::{Chunk, Error, Model, Sequence, SequenceFlags};

#[test]
fn sequence_fields_round_trip() {
    let mut stand = Sequence::new("Stand", [0, 1000]).unwrap();
    stand.set_move_speed(270.0);
    stand.set_flags(SequenceFlags::NON_LOOPING);
    stand.set_rarity(0.5);
    stand.set_sync_point(500);
    stand.set_bounds_radius(42.0);
    stand.set_minimum_extent([-2.0, -3.0, -4.0]);
    stand.set_maximum_extent([2.0, 3.0, 4.0]);
    let mut model = Model::new(800);
    model.set_sequences(&[stand]).unwrap();
    let decoded = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let sequence = &decoded.sequences().unwrap()[0];
    assert_eq!(sequence.name(), "Stand");
    assert_eq!(sequence.interval(), [0, 1000]);
    assert_eq!(sequence.move_speed(), 270.0);
    assert!(sequence.flags().contains(SequenceFlags::NON_LOOPING));
    assert_eq!(sequence.rarity(), 0.5);
    assert_eq!(sequence.sync_point(), 500);
    assert_eq!(sequence.bounds_radius(), 42.0);
    assert_eq!(sequence.minimum_extent(), [-2.0, -3.0, -4.0]);
    assert_eq!(sequence.maximum_extent(), [2.0, 3.0, 4.0]);
}

#[test]
fn malformed_sequence_chunk_is_reported() {
    let mut model = Model::new(1800);
    model.push(Chunk::new(*b"SEQS", vec![0; 131]));
    assert_eq!(
        model.sequences(),
        Err(Error::MalformedChunk {
            tag: *b"SEQS",
            size: 131,
            expected: 132,
        })
    );
}
