use std::slice::from_ref;

use wc3_mdx::animation::Sequence;
use wc3_mdx::chunks::{ModelChunk, RawChunk, SequencesChunk, UnknownChunk};
use wc3_mdx::io::{Decodable, Encodable};
use wc3_mdx::{Model, V800};

#[test]
fn model_stores_known_and_unknown_chunks() {
    let mut model = Model::<V800>::new();
    model.push(ModelChunk::from(SequencesChunk::new(vec![Sequence::new(
        "Stand",
        [0, 100],
    )
    .unwrap()])));
    let unknown = UnknownChunk::<V800>::new(RawChunk::new(*b"FUTR", vec![1, 2, 3])).unwrap();
    model.push(ModelChunk::Unknown(unknown));

    assert!(matches!(model.chunks()[1], ModelChunk::Sequences(_)));
    assert!(matches!(model.chunks()[2], ModelChunk::Unknown(_)));
    let bytes = model.encode().unwrap();
    let decoded = Model::<V800>::decode(&bytes).unwrap();
    assert_eq!(decoded.encode().unwrap(), bytes);
}

#[test]
fn edits_to_decoded_records_are_written() {
    let mut model = Model::<V800>::new();
    model.push(ModelChunk::from(SequencesChunk::new(Vec::new())));
    let ModelChunk::Sequences(decoded) = &mut model.chunks_mut()[1] else {
        panic!("expected typed sequence chunk");
    };
    decoded
        .records
        .push(Sequence::new("Walk", [0, 100]).unwrap());

    let bytes = model.encode().unwrap();
    let reopened = Model::<V800>::decode(&bytes).unwrap();
    assert_eq!(reopened.sequences()[0].name(), "Walk");
}

#[test]
fn malformed_known_chunk_cannot_enter_typed_model() {
    assert!(ModelChunk::<V800>::from_raw(RawChunk::new(*b"SEQS", vec![0; 131])).is_err());
    assert!(UnknownChunk::<V800>::new(RawChunk::new(*b"SEQS", Vec::new())).is_none());
}

#[test]
fn collection_setter_collapses_repeated_chunks() {
    let mut model = Model::<V800>::new();
    let first = Sequence::new("Stand", [0, 100]).unwrap();
    let second = Sequence::new("Walk", [101, 200]).unwrap();
    model.push(ModelChunk::from(SequencesChunk::new(vec![first.clone()])));
    model.push(ModelChunk::from(SequencesChunk::new(vec![second.clone()])));
    assert_eq!(model.sequences(), vec![first, second.clone()]);

    model.set_sequences(from_ref(&second));
    assert!(matches!(
        model.chunk(*b"SEQS"),
        Some(ModelChunk::Sequences(_))
    ));
    assert_eq!(
        model
            .chunks()
            .iter()
            .filter(|chunk| chunk.tag() == *b"SEQS")
            .count(),
        1
    );
    assert_eq!(model.sequences(), vec![second]);
}
