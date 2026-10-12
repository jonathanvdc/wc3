use wc3::model::mdlx::chunks::{ModelChunk, RawChunk, UnknownChunk};
use wc3::model::mdlx::mdx::Read as _;
use wc3::model::mdlx::mdx::Write as _;

use wc3::model::mdlx::{
    Model, ModelVersion, V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

fn check_version<V: ModelVersion>() {
    let mut model = Model::<V>::new();
    model
        .chunks
        .push(ModelChunk::from_raw(RawChunk::new(*b"MODL", vec![0; 372])).unwrap());
    model.chunks.push(ModelChunk::Unknown(
        UnknownChunk::<V>::new(RawChunk::new(*b"FUTR", vec![0, 1, 2, 255])).unwrap(),
    ));
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<V>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.version(), V::NUMBER);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
}

#[test]
fn synthetic_versions_and_unknown_chunks_round_trip() {
    check_version::<V800>();
    check_version::<V900>();
    check_version::<V1000>();
    check_version::<V1100>();
    check_version::<V1200>();
    check_version::<V1300>();
    check_version::<V1400>();
    check_version::<V1600>();
    check_version::<V1800>();
}

#[test]
fn preserves_repeated_chunks_and_order() {
    let mut model = Model::<V800>::new();
    for data in [vec![1], vec![2]] {
        model.chunks.push(ModelChunk::Unknown(
            UnknownChunk::<V800>::new(RawChunk::new(*b"ABCD", data)).unwrap(),
        ));
    }
    let parsed = Model::<V800>::decode_mdx(&model.encode_mdx().unwrap())
        .unwrap()
        .into_owned();
    assert!(
        matches!(&parsed.chunks[1], ModelChunk::Unknown(raw) if raw.raw().data.as_ref() == [1])
    );
    assert!(
        matches!(&parsed.chunks[2], ModelChunk::Unknown(raw) if raw.raw().data.as_ref() == [2])
    );
}
