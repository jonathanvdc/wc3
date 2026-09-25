use wc3_mdx::{Error, Model, RawChunk};
use wc3_mdx::{ModelChunk, Record};

#[test]
fn global_sequences_and_pivots_round_trip() {
    let mut model = Model::new(1100);
    model.set_global_sequences(&[1000, 2500]).unwrap();
    model
        .set_pivot_points(&[[1.0, 2.0, 3.0], [-4.0, 5.5, 0.0]])
        .unwrap();
    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    assert_eq!(parsed.global_sequences().unwrap(), vec![1000, 2500]);
    assert_eq!(
        parsed.pivot_points().unwrap(),
        vec![[1.0, 2.0, 3.0], [-4.0, 5.5, 0.0]]
    );
}

#[test]
fn malformed_scalar_chunks_are_reported() {
    let mut model = Model::new(800);
    model.push(ModelChunk::from_raw(RawChunk::new(*b"GLBS", vec![1]), 800));
    model.push(ModelChunk::from_raw(
        RawChunk::new(*b"PIVT", vec![0; 11]),
        800,
    ));
    assert_eq!(
        model.global_sequences(),
        Err(Error::MalformedChunk {
            tag: *b"GLBS",
            size: 1,
            expected: 4,
        })
    );
    assert_eq!(
        model.pivot_points(),
        Err(Error::MalformedChunk {
            tag: *b"PIVT",
            size: 11,
            expected: 12,
        })
    );
}
