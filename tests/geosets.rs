use wc3_mdx::{Chunk, Error, Geoset, GeosetExtent, Model};

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
fn builds_complete_synthetic_geosets() {
    for version in [800, 900, 1100, 1200, 1800] {
        let mut geoset = Geoset::new(
            version,
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            &[[0.0, 0.0, 1.0]; 2],
            &[0, 1, 0],
        )
        .unwrap();
        geoset.set_material_id(version, 7).unwrap();
        geoset
            .set_matrix_groups(version, &[vec![1, 2], vec![3]])
            .unwrap();
        geoset
            .set_sequence_extents(
                version,
                &[GeosetExtent {
                    bounds_radius: 2.0,
                    minimum: [-2.0; 3],
                    maximum: [2.0; 3],
                }],
            )
            .unwrap();
        geoset
            .set_uv_sets(version, &[vec![[0.0, 0.0]; 2], vec![[1.0, 1.0]; 2]])
            .unwrap();
        geoset.set_normal(1, [0.0, 1.0, 0.0]).unwrap();
        geoset.set_selection_group(version, 3).unwrap();
        geoset.set_unselectable(version, true).unwrap();
        geoset.set_uv(version, 0, 1, [0.25, 0.75]).unwrap();
        let extent = GeosetExtent {
            bounds_radius: 5.0,
            minimum: [-1.0; 3],
            maximum: [1.0; 3],
        };
        geoset.set_extent(version, extent).unwrap();
        if version >= 900 {
            geoset.set_level_of_detail(version, 2).unwrap();
            geoset.set_name(version, "Body").unwrap();
            assert_eq!(geoset.name(version).unwrap().as_deref(), Some("Body"));
        }
        assert_eq!(geoset.material_id(version).unwrap(), 7);
        assert_eq!(geoset.normals().unwrap()[1], [0.0, 1.0, 0.0]);
        assert_eq!(geoset.selection_group(version).unwrap(), 3);
        assert!(geoset.unselectable(version).unwrap());
        assert_eq!(geoset.extent(version).unwrap(), extent);
        assert_eq!(geoset.vertex_groups(version).unwrap(), &[0, 0]);
        assert_eq!(geoset.matrix_group_sizes(version).unwrap(), vec![2, 1]);
        assert_eq!(geoset.matrix_indices(version).unwrap(), vec![1, 2, 3]);
        assert_eq!(geoset.sequence_extents(version).unwrap().len(), 1);
        assert_eq!(
            geoset.uv_sets(version).unwrap(),
            vec![vec![[0.0, 0.0], [0.25, 0.75]], vec![[1.0, 1.0]; 2]]
        );
        let mut model = Model::new(version);
        model.set_geosets(&[geoset]).unwrap();
        let parsed = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
        assert_eq!(
            parsed.geosets().unwrap()[0].material_id(version).unwrap(),
            7
        );
    }
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
                    let version = model.version().unwrap();
                    geoset.vertex_groups(version).unwrap();
                    geoset.matrix_group_sizes(version).unwrap();
                    geoset.matrix_indices(version).unwrap();
                    geoset.extent(version).unwrap();
                    geoset.sequence_extents(version).unwrap();
                    geoset.tangents(version).unwrap();
                    geoset.skin_weights(version).unwrap();
                    geoset.skin_bone_indices(version).unwrap();
                    geoset.uv_sets(version).unwrap();
                }
            }
        }
    }
}
