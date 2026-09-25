use wc3_mdx::{Chunk, Model};

#[test]
fn synthetic_versions_and_unknown_chunks_round_trip() {
    for version in [800, 900, 1000, 1100, 1200, 1800] {
        let mut model = Model::new(version);
        model.push(Chunk::new(*b"MODL", vec![0; 372]));
        model.push(Chunk::new(*b"FUTR", vec![0, 1, 2, 255]));
        let bytes = model.to_bytes().unwrap();
        let parsed = Model::from_bytes(&bytes).unwrap();
        assert_eq!(parsed.version(), Some(version));
        assert_eq!(parsed.to_bytes().unwrap(), bytes);
    }
}

#[test]
fn preserves_repeated_chunks_and_order() {
    let mut model = Model::new(800);
    model.push(Chunk::new(*b"ABCD", vec![1]));
    model.push(Chunk::new(*b"ABCD", vec![2]));
    let parsed = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    assert_eq!(parsed.chunks()[1].data, vec![1]);
    assert_eq!(parsed.chunks()[2].data, vec![2]);
}

#[test]
fn local_files_round_trip_when_available() {
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
                let model = Model::from_bytes(&bytes)
                    .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
                assert_eq!(model.to_bytes().unwrap(), bytes, "{}", path.display());
            }
        }
    }
}
