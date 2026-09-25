use wc3_mdx::Record;
use wc3_mdx::{Error, Geoset, Material, Model, ModelChunk, Node, RawChunk, RibbonEmitter};

#[test]
fn validates_synthetic_known_chunks_and_preserves_unknown() {
    let mut model = Model::new(800);
    model
        .set_geosets(&[
            Geoset::new(800, &[[0.0, 0.0, 0.0]], &[[0.0, 0.0, 1.0]], &[0, 0, 0]).unwrap(),
        ])
        .unwrap();
    model.set_ribbon_emitters(&[RibbonEmitter::new(Node::new("Trail", 1).unwrap())]);
    model.push(ModelChunk::from_raw(
        RawChunk::new(*b"FUTR", vec![1, 2, 3]),
        800,
    ));
    model.validate().unwrap();
    let bytes = model.encode().unwrap();
    assert_eq!(Model::decode(&bytes, 800).unwrap().encode().unwrap(), bytes);
}

#[test]
fn empty_mdlx_is_valid() {
    Model::decode(b"MDLX", 800).unwrap().validate().unwrap();
}

#[test]
fn rejects_malformed_known_track() {
    let mut model = Model::new(800);
    let mut ribbon = RibbonEmitter::new(Node::new("Trail", 1).unwrap())
        .encode()
        .unwrap()
        .to_vec();
    ribbon.extend_from_slice(b"KRVS");
    let len = ribbon.len() as u32;
    ribbon[..4].copy_from_slice(&len.to_le_bytes());
    model.push(ModelChunk::from_raw(RawChunk::new(*b"RIBB", ribbon), 800));
    assert!(model.validate().is_err());
}

#[test]
fn rejects_short_repeated_version_chunks() {
    let mut bytes = Model::new(800).encode().unwrap();
    bytes.extend_from_slice(b"VERS");
    bytes.extend_from_slice(&2u32.to_le_bytes());
    bytes.extend_from_slice(&[1, 2]);
    assert_eq!(Model::decode(&bytes, 800), Err(Error::InvalidVersionChunk));

    let mut model = Model::new(800);
    model.chunks_mut()[0] = ModelChunk::from_raw(RawChunk::new(*b"VERS", Vec::new()), 800);
    assert_eq!(model.stored_version(), None);
    assert_eq!(model.version(), 800);
    assert_eq!(model.validate(), Err(Error::InvalidVersionChunk));
    model.set_version(1800);
    assert_eq!(model.version(), 1800);
    assert_eq!(model.chunks().len(), 1);
    model.validate().unwrap();
}

#[test]
fn set_version_replaces_repeated_chunks_and_preserves_extension() {
    let mut model = Model::new(800);
    let ModelChunk::Version(first) = &mut model.chunks_mut()[0] else {
        unreachable!()
    };
    first.extension = vec![7, 8];
    model.push(ModelChunk::Version(wc3_mdx::VersionChunk::new(900)));

    model.set_version(1800);

    assert_eq!(model.chunks().len(), 1);
    let ModelChunk::Version(current) = &model.chunks()[0] else {
        unreachable!()
    };
    assert_eq!(current.version, 1800);
    assert_eq!(current.extension, [7, 8]);
}

#[test]
fn rejects_layer_shorter_than_its_versioned_header() {
    let mut material = Material::new(1800);
    let layer = wc3_mdx::Layer::new(800).encode().unwrap();
    assert!(wc3_mdx::Layer::decode(&layer, 1800).is_err());
    assert_eq!(
        material.set_layers(&[wc3_mdx::Layer::new(800)]),
        Err(Error::VersionMismatch {
            expected: 1800,
            actual: 800
        })
    );
}

#[test]
fn rejects_records_from_another_model_version() {
    let mut model = Model::new(1800);
    assert_eq!(
        model.set_materials(&[Material::new(800)]),
        Err(Error::VersionMismatch {
            expected: 1800,
            actual: 800
        })
    );
    assert_eq!(
        model.set_geosets(&[Geoset::new(800, &[], &[], &[]).unwrap()]),
        Err(Error::VersionMismatch {
            expected: 1800,
            actual: 800
        })
    );
}

#[test]
fn validates_every_repeated_model_info_chunk() {
    let mut model = Model::new(800);
    model.set_model_info(&wc3_mdx::ModelInfo::new("Good").unwrap());
    model.push(ModelChunk::from_raw(
        RawChunk::new(*b"MODL", vec![0; 12]),
        800,
    ));
    assert!(model.model_info().unwrap().is_some());
    assert!(model.validate().is_err());
}

#[test]
fn validates_local_models_when_available() {
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
                let model = Model::decode(&bytes, 800).unwrap();
                model
                    .validate()
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            }
        }
    }
}
