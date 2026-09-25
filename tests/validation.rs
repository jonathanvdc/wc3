use wc3_mdx::{Chunk, Geoset, Model, Node, RibbonEmitter};

#[test]
fn validates_synthetic_known_chunks_and_preserves_unknown() {
    let mut model = Model::new(800);
    model
        .set_geosets(&[
            Geoset::new(800, &[[0.0, 0.0, 0.0]], &[[0.0, 0.0, 1.0]], &[0, 0, 0]).unwrap(),
        ])
        .unwrap();
    model
        .set_ribbon_emitters(&[RibbonEmitter::new(Node::new("Trail", 1).unwrap()).unwrap()])
        .unwrap();
    model.push(Chunk::new(*b"FUTR", vec![1, 2, 3]));
    model.validate().unwrap();
    let bytes = model.to_bytes().unwrap();
    assert_eq!(
        Model::from_bytes(&bytes).unwrap().to_bytes().unwrap(),
        bytes
    );
}

#[test]
fn empty_mdlx_is_valid() {
    Model::from_bytes(b"MDLX").unwrap().validate().unwrap();
}

#[test]
fn rejects_malformed_known_track() {
    let mut model = Model::new(800);
    let mut ribbon = RibbonEmitter::new(Node::new("Trail", 1).unwrap())
        .unwrap()
        .as_bytes()
        .to_vec();
    ribbon.extend_from_slice(b"KRVS");
    let len = ribbon.len() as u32;
    ribbon[..4].copy_from_slice(&len.to_le_bytes());
    model.push(Chunk::new(*b"RIBB", ribbon));
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
                let model = Model::from_bytes(&bytes).unwrap();
                model
                    .validate()
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            }
        }
    }
}
