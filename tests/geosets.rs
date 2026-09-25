use wc3_mdx::{Chunk, Error, Geoset, GeosetExtent, Model};

fn sample_geoset() -> Geoset {
    let geoset = Geoset::new(1800, &[[1.0, 2.0, 3.0]], &[[0.0, 0.0, 1.0]], &[0, 0, 0]).unwrap();
    Geoset::from_bytes(1800, geoset.as_bytes()).unwrap()
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
        geoset.set_material_id(7).unwrap();
        geoset.set_matrix_groups(&[vec![1, 2], vec![3]]).unwrap();
        geoset.set_vertex_groups(&[0, 1]).unwrap();
        geoset
            .set_sequence_extents(&[GeosetExtent {
                bounds_radius: 2.0,
                minimum: [-2.0; 3],
                maximum: [2.0; 3],
            }])
            .unwrap();
        geoset
            .set_uv_sets(&[vec![[0.0, 0.0]; 2], vec![[1.0, 1.0]; 2]])
            .unwrap();
        geoset.set_normal(1, [0.0, 1.0, 0.0]).unwrap();
        geoset.set_selection_group(3).unwrap();
        geoset.set_unselectable(true).unwrap();
        geoset.set_uv(0, 1, [0.25, 0.75]).unwrap();
        let extent = GeosetExtent {
            bounds_radius: 5.0,
            minimum: [-1.0; 3],
            maximum: [1.0; 3],
        };
        geoset.set_extent(extent).unwrap();
        if version >= 900 {
            geoset.set_level_of_detail(2).unwrap();
            geoset.set_name("Body").unwrap();
            geoset
                .set_tangents(Some(&[[1.0, 0.0, 0.0, 1.0]; 2]))
                .unwrap();
            let weights = [0u8; 16];
            let indices = [1u8; 16];
            geoset
                .set_skin_data(
                    Some(&weights),
                    (version >= 1200).then_some(indices.as_slice()),
                )
                .unwrap();
            assert_eq!(geoset.tangents().unwrap().unwrap().len(), 2);
            assert_eq!(geoset.skin_weights().unwrap(), Some(weights.as_slice()));
            if version >= 1200 {
                assert_eq!(
                    geoset.skin_bone_indices().unwrap(),
                    Some(indices.as_slice())
                );
            }
            geoset.set_skin_data(None, None).unwrap();
            geoset.set_tangents(None).unwrap();
            assert!(geoset.skin_weights().unwrap().is_none());
            assert!(geoset.tangents().unwrap().is_none());
            assert_eq!(geoset.name().unwrap().as_deref(), Some("Body"));
        }
        assert_eq!(geoset.material_id().unwrap(), 7);
        assert_eq!(geoset.normals().unwrap()[1], [0.0, 1.0, 0.0]);
        assert_eq!(geoset.selection_group().unwrap(), 3);
        assert!(geoset.unselectable().unwrap());
        assert_eq!(geoset.extent().unwrap(), extent);
        assert_eq!(geoset.vertex_groups().unwrap(), &[0, 1]);
        assert_eq!(geoset.matrix_group_sizes().unwrap(), vec![2, 1]);
        assert_eq!(geoset.matrix_indices().unwrap(), vec![1, 2, 3]);
        assert_eq!(geoset.sequence_extents().unwrap().len(), 1);
        assert_eq!(
            geoset.uv_sets().unwrap(),
            vec![vec![[0.0, 0.0], [0.25, 0.75]], vec![[1.0, 1.0]; 2]]
        );
        let mut model = Model::new(version);
        model.set_geosets(&[geoset]).unwrap();
        let parsed = Model::from_bytes(&model.to_bytes().unwrap()).unwrap();
        assert_eq!(parsed.geosets().unwrap()[0].version(), version);
        assert_eq!(parsed.geosets().unwrap()[0].material_id().unwrap(), 7);
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
                    geoset.vertex_groups().unwrap();
                    geoset.matrix_group_sizes().unwrap();
                    geoset.matrix_indices().unwrap();
                    geoset.extent().unwrap();
                    geoset.sequence_extents().unwrap();
                    geoset.tangents().unwrap();
                    geoset.skin_weights().unwrap();
                    geoset.skin_bone_indices().unwrap();
                    geoset.uv_sets().unwrap();
                }
            }
        }
    }
}
