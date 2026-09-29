use wc3::model::chunks::{ModelChunk, RawChunk, VersionChunk};
use wc3::model::emitters::RibbonEmitter;
use wc3::model::materials::Layer;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::ReadError;
use wc3::model::mdx::Write as _;
use wc3::model::scene::Node;
use wc3::model::{Model, V1800, V800};

#[test]
fn empty_mdlx_uses_the_requested_type() {
    assert_eq!(Model::<V800>::decode_mdx(b"MDLX").unwrap().version(), 800);
}

#[test]
fn malformed_known_track_cannot_enter_typed_model() {
    let mut ribbon = RibbonEmitter::new(Node::new("Trail", 1).unwrap())
        .encode_mdx()
        .unwrap();
    ribbon.extend_from_slice(b"KRVS");
    let len = ribbon.len() as u32;
    ribbon[..4].copy_from_slice(&len.to_le_bytes());
    assert!(ModelChunk::<V800>::from_raw(RawChunk::new(*b"RIBB", ribbon)).is_err());
}

#[test]
fn repeated_versions_must_match_the_type() {
    let mut bytes = Model::<V800>::new().encode_mdx().unwrap();
    bytes.extend_from_slice(b"VERS");
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 2]);
    assert!(matches!(
        Model::<V800>::decode_mdx(&bytes),
        Err(ReadError::InvalidVersionChunk)
    ));

    let wrong = RawChunk::new(*b"VERS", 1800u32.to_le_bytes().to_vec());
    assert!(matches!(
        ModelChunk::<V800>::from_raw(wrong),
        Err(ReadError::VersionMismatch {
            expected: 800,
            actual: 1800
        })
    ));
}

#[test]
fn version_extension_is_preserved_without_mutable_version_number() {
    let mut model = Model::<V800>::new();
    let ModelChunk::Version(first) = &mut model.chunks[0] else {
        unreachable!()
    };
    first.extension = vec![7, 8];
    model
        .chunks
        .push(ModelChunk::from(VersionChunk::<V800>::new()));
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<V800>::decode_mdx(&bytes).unwrap();
    let ModelChunk::Version(first) = &parsed.chunks[0] else {
        unreachable!()
    };
    assert_eq!(first.extension, [7, 8]);
    assert_eq!(parsed.version(), 800);
}

#[test]
fn layer_layout_is_selected_by_its_type() {
    let classic = Layer::<V800>::new().encode_mdx().unwrap();
    assert!(Layer::<V1800>::decode_mdx(&classic).is_err());
}

#[test]
fn malformed_model_info_chunk_is_rejected() {
    assert!(ModelChunk::<V800>::from_raw(RawChunk::new(*b"MODL", vec![0; 12])).is_err());
}
