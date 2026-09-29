use wc3::model::chunks::{ModelChunk, RawChunk, VersionChunk};
use wc3::model::emitters::RibbonEmitter;
use wc3::model::materials::Layer;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::ReadError;
use wc3::model::mdx::ReadErrorKind;
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
        Err(ReadError {
            kind: ReadErrorKind::UnexpectedEnd { .. },
            ..
        })
    ));

    let wrong = RawChunk::new(*b"VERS", 1800u32.to_le_bytes().to_vec());
    assert!(matches!(
        ModelChunk::<V800>::from_raw(wrong),
        Err(ReadError {
            offset: 0,
            tag: Some([86, 69, 82, 83]),
            kind: ReadErrorKind::VersionMismatch {
                expected: 800,
                actual: 1800
            }
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

#[test]
fn nested_track_failures_keep_absolute_offsets_and_animation_tags() {
    let mut payload = RibbonEmitter::new(Node::new("Trail", 1).unwrap())
        .encode_mdx()
        .unwrap();
    let track_start = payload.len();
    payload.extend_from_slice(b"KRVS");
    payload.extend_from_slice(&0u32.to_le_bytes());
    payload.extend_from_slice(&9u32.to_le_bytes());
    payload.extend_from_slice(&u32::MAX.to_le_bytes());
    let len = payload.len() as u32;
    payload[..4].copy_from_slice(&len.to_le_bytes());
    let mut bytes = Model::<V800>::new().encode_mdx().unwrap();
    let payload_start = bytes.len() + 8;
    bytes.extend_from_slice(b"RIBB");
    bytes.extend_from_slice(&len.to_le_bytes());
    bytes.extend_from_slice(&payload);
    let error = Model::<V800>::decode_mdx(&bytes).unwrap_err();
    assert_eq!(error.offset, payload_start + track_start + 8);
    assert_eq!(error.tag, Some(*b"KRVS"));
    assert_eq!(
        error.kind,
        ReadErrorKind::UnknownEnumValue {
            enum_name: "Interpolation",
            value: 9,
        }
    );
    assert!(error.to_string().contains("unknown Interpolation value 9"));
}

#[test]
fn version_failures_identify_the_version_value() {
    let bytes = b"MDLXVERS\x04\0\0\0\x20\x03\0\0";
    let error = Model::<V1800>::decode_mdx(bytes).unwrap_err();
    assert_eq!(error.offset, 12);
    assert_eq!(error.tag, Some(*b"VERS"));
    assert_eq!(
        error.kind,
        ReadErrorKind::VersionMismatch {
            expected: 1800,
            actual: 800
        }
    );
}
