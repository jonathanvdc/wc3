use wc3_mdx::{Chunk, Error, Geoset, Model};

fn sample_geoset() -> Geoset {
    let mut data = Vec::new();
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(b"VRTX");
    data.extend_from_slice(&1u32.to_le_bytes());
    for value in [1.0f32, 2.0, 3.0] {
        data.extend_from_slice(&value.to_le_bytes());
    }
    data.extend_from_slice(b"NRMS");
    data.extend_from_slice(&1u32.to_le_bytes());
    for value in [0.0f32, 0.0, 1.0] {
        data.extend_from_slice(&value.to_le_bytes());
    }
    data.extend_from_slice(b"PTYP");
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&4u32.to_le_bytes());
    data.extend_from_slice(b"PCNT");
    data.extend_from_slice(&1u32.to_le_bytes());
    data.extend_from_slice(&3u32.to_le_bytes());
    data.extend_from_slice(b"PVTX");
    data.extend_from_slice(&3u32.to_le_bytes());
    for index in [0u16, 0, 0] {
        data.extend_from_slice(&index.to_le_bytes());
    }
    let size = data.len() as u32;
    data[..4].copy_from_slice(&size.to_le_bytes());
    Geoset::from_bytes(&data).unwrap()
}

#[test]
fn geoset_mesh_edit_preserves_other_sections() {
    let mut geoset = sample_geoset();
    assert_eq!(geoset.vertices().unwrap(), vec![[1.0, 2.0, 3.0]]);
    assert_eq!(geoset.normals().unwrap(), vec![[0.0, 0.0, 1.0]]);
    assert_eq!(geoset.face_indices().unwrap(), vec![0, 0, 0]);
    geoset.set_vertex(0, [4.0, 5.0, 6.0]).unwrap();
    let mut model = Model::new(1800);
    model.set_geosets(&[geoset]).unwrap();
    let decoded = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
    assert_eq!(
        decoded.geosets().unwrap()[0].vertices().unwrap(),
        vec![[4.0, 5.0, 6.0]]
    );
    assert_eq!(
        decoded.geosets().unwrap()[0].normals().unwrap(),
        vec![[0.0, 0.0, 1.0]]
    );
}

#[test]
fn rejects_invalid_geoset_sizes() {
    let mut model = Model::new(800);
    model.push(Chunk::new(*b"GEOS", 100u32.to_le_bytes().to_vec()));
    assert_eq!(
        model.geosets(),
        Err(Error::MalformedRecord {
            tag: *b"GEOS",
            offset: 0
        })
    );
}

#[test]
fn local_geosets_have_bounded_mesh_sections_when_available() {
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
                for geoset in model.geosets().unwrap() {
                    geoset.vertices().unwrap();
                    geoset.normals().unwrap();
                    geoset.face_indices().unwrap();
                }
            }
        }
    }
}
