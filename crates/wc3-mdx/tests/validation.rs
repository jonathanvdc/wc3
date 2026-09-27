use wc3_mdx::chunks::{ModelChunk, RawChunk, UnknownChunk, VersionChunk};
use wc3_mdx::emitters::RibbonEmitter;
use wc3_mdx::geometry::Geoset;
use wc3_mdx::io::{DecodeError, Encodable, Readable};
use wc3_mdx::materials::Layer;
use wc3_mdx::scene::Node;
use wc3_mdx::{AnyVersionModel, Model, V1800, V800};

#[test]
fn typed_known_chunks_and_unknown_chunks_round_trip() {
    let mut model = Model::<V800>::new();
    model.set_geosets(&[
        Geoset::<V800>::new(&[[0.0, 0.0, 0.0]], &[[0.0, 0.0, 1.0]], &[0, 0, 0]).unwrap(),
    ]);
    model.set_ribbon_emitters(&[RibbonEmitter::new(Node::new("Trail", 1).unwrap())]);
    model.push(ModelChunk::Unknown(
        UnknownChunk::<V800>::new(RawChunk::new(*b"FUTR", vec![1, 2, 3])).unwrap(),
    ));
    let bytes = model.encode().unwrap();
    assert_eq!(
        Model::<V800>::decode(&bytes).unwrap().encode().unwrap(),
        bytes
    );
}

#[test]
fn empty_mdlx_uses_the_requested_type() {
    assert_eq!(Model::<V800>::decode(b"MDLX").unwrap().version(), 800);
}

#[test]
fn malformed_known_track_cannot_enter_typed_model() {
    let mut ribbon = RibbonEmitter::new(Node::new("Trail", 1).unwrap())
        .encode()
        .unwrap();
    ribbon.extend_from_slice(b"KRVS");
    let len = ribbon.len() as u32;
    ribbon[..4].copy_from_slice(&len.to_le_bytes());
    assert!(ModelChunk::<V800>::from_raw(RawChunk::new(*b"RIBB", ribbon)).is_err());
}

#[test]
fn repeated_versions_must_match_the_type() {
    let mut bytes = Model::<V800>::new().encode().unwrap();
    bytes.extend_from_slice(b"VERS");
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 2]);
    assert!(matches!(
        Model::<V800>::decode(&bytes),
        Err(DecodeError::InvalidVersionChunk)
    ));

    let wrong = RawChunk::new(*b"VERS", 1800u32.to_le_bytes().to_vec());
    assert!(matches!(
        ModelChunk::<V800>::from_raw(wrong),
        Err(DecodeError::VersionMismatch {
            expected: 800,
            actual: 1800
        })
    ));
}

#[test]
fn version_extension_is_preserved_without_mutable_version_number() {
    let mut model = Model::<V800>::new();
    let ModelChunk::Version(first) = &mut model.chunks_mut()[0] else {
        unreachable!()
    };
    first.extension = vec![7, 8];
    model.push(ModelChunk::from(VersionChunk::<V800>::new()));
    let bytes = model.encode().unwrap();
    let parsed = Model::<V800>::decode(&bytes).unwrap();
    let ModelChunk::Version(first) = &parsed.chunks()[0] else {
        unreachable!()
    };
    assert_eq!(first.extension, [7, 8]);
    assert_eq!(parsed.version(), 800);
}

#[test]
fn layer_layout_is_selected_by_its_type() {
    let classic = Layer::<V800>::new().encode().unwrap();
    assert!(Layer::<V1800>::decode(&classic).is_err());
}

#[test]
fn malformed_model_info_chunk_is_rejected() {
    assert!(ModelChunk::<V800>::from_raw(RawChunk::new(*b"MODL", vec![0; 12])).is_err());
}

#[test]
fn local_models_decode_when_available() {
    let Ok(directory) = std::env::var("WC3_MDX_FIXTURES") else {
        return;
    };
    let mut pending = vec![std::path::PathBuf::from(directory)];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "mdx") {
                let bytes = std::fs::read(&path).unwrap();
                AnyVersionModel::decode(&bytes, 800)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            }
        }
    }
}
