use wc3_mdx::{Error, Model, ModelChunk, RawChunk, Record, Sequence, SequencesChunk};

#[test]
fn model_stores_known_unknown_and_malformed_chunks() {
    let mut model = Model::new(800);
    let ModelChunk::Version(version) = &model.chunks()[0] else {
        panic!("expected decoded version chunk");
    };
    assert_eq!(version.version, 800);
    model.push(ModelChunk::Sequences(SequencesChunk::new(vec![
        Sequence::new("Stand", [0, 100]).unwrap(),
    ])));
    model.push(ModelChunk::from_raw(
        RawChunk::new(*b"FUTR", vec![1, 2, 3]),
        800,
    ));
    model.push(ModelChunk::from_raw(
        RawChunk::new(*b"TEXS", vec![0; 267]),
        800,
    ));

    assert!(matches!(model.chunks()[1], ModelChunk::Sequences(_)));
    assert!(matches!(model.chunks()[2], ModelChunk::Unknown(_)));
    assert!(matches!(model.chunks()[3], ModelChunk::Malformed(_)));
    assert_eq!(
        model.validate(),
        Err(Error::MalformedChunk {
            tag: *b"TEXS",
            size: 267,
            expected: 268
        })
    );

    let bytes = model.encode().unwrap();
    let decoded = Model::decode(&bytes, 800).unwrap();
    assert_eq!(decoded.encode().unwrap(), bytes);
    assert!(matches!(decoded.chunks()[3], ModelChunk::Malformed(_)));
}

#[test]
fn edits_to_decoded_records_are_written() {
    let mut model = Model::new(800);
    model.push(ModelChunk::from_raw(
        RawChunk::new(*b"SEQS", Vec::new()),
        800,
    ));
    let ModelChunk::Sequences(decoded) = &mut model.chunks_mut()[1] else {
        panic!("expected typed sequence chunk");
    };
    decoded
        .records
        .push(Sequence::new("Walk", [0, 100]).unwrap());

    let bytes = model.encode().unwrap();
    let reopened = Model::decode(&bytes, 800).unwrap();
    assert_eq!(reopened.sequences().unwrap()[0].name(), "Walk");
}

#[test]
fn replacing_a_malformed_chunk_clears_its_error() {
    let mut model = Model::new(800);
    model.push(ModelChunk::from_raw(
        RawChunk::new(*b"SEQS", vec![0; 131]),
        800,
    ));
    assert!(matches!(model.chunks()[1], ModelChunk::Malformed(_)));

    *model.chunk_mut(*b"SEQS").unwrap() = ModelChunk::Sequences(SequencesChunk::new(vec![
        Sequence::new("Stand", [0, 100]).unwrap(),
    ]));
    assert_eq!(model.sequences().unwrap().len(), 1);
    assert!(model.validate().is_ok());
    assert!(matches!(model.chunks()[1], ModelChunk::Sequences(_)));
}

#[test]
fn collection_setter_keeps_records_decoded_and_collapses_repeated_chunks() {
    let mut model = Model::new(800);
    let first = Sequence::new("Stand", [0, 100]).unwrap();
    let second = Sequence::new("Walk", [101, 200]).unwrap();
    model.push(ModelChunk::Sequences(SequencesChunk::new(vec![
        first.clone()
    ])));
    model.push(ModelChunk::Sequences(SequencesChunk::new(vec![
        second.clone()
    ])));
    assert_eq!(model.sequences().unwrap(), vec![first, second.clone()]);

    model.set_sequences(&[second.clone()]);
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
    assert_eq!(model.sequences().unwrap(), vec![second]);
}
