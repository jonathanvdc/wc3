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

#[test]
fn typed_accessors_preserve_local_files_when_available() {
    let Ok(directory) = std::env::var("WC3_MDX_FIXTURES") else {
        return;
    };
    let mut pending = vec![std::path::PathBuf::from(directory)];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if !path.extension().is_some_and(|extension| extension == "mdx") {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            let mut model = Model::from_bytes(&bytes).unwrap();
            if let Some(info) = model.model_info().unwrap() {
                model.set_model_info(&info);
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"SEQS")
                .count()
                == 1
            {
                let records = model.sequences().unwrap();
                model.set_sequences(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"TEXS")
                .count()
                == 1
            {
                let records = model.textures().unwrap();
                model.set_textures(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"MTLS")
                .count()
                == 1
            {
                let records = model.materials().unwrap();
                model.set_materials(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"GEOS")
                .count()
                == 1
            {
                let records = model.geosets().unwrap();
                model.set_geosets(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"BONE")
                .count()
                == 1
            {
                let records = model.bones().unwrap();
                model.set_bones(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"PIVT")
                .count()
                == 1
            {
                let points = model.pivot_points().unwrap();
                model.set_pivot_points(&points).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"EVTS")
                .count()
                == 1
            {
                let records = model.event_objects().unwrap();
                model.set_event_objects(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"CLID")
                .count()
                == 1
            {
                let records = model.collision_shapes().unwrap();
                model.set_collision_shapes(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"ATCH")
                .count()
                == 1
            {
                let records = model.attachments().unwrap();
                model.set_attachments(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"FAFX")
                .count()
                == 1
            {
                let records = model.face_fx().unwrap();
                model.set_face_fx(&records).unwrap();
            }
            if model
                .chunks()
                .iter()
                .filter(|chunk| chunk.tag == *b"GEOA")
                .count()
                == 1
            {
                let records = model.geoset_animations().unwrap();
                model.set_geoset_animations(&records).unwrap();
            }
            assert_eq!(model.to_bytes().unwrap(), bytes, "{}", path.display());
        }
    }
}
