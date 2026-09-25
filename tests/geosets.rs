use wc3_mdx::{Error, Geoset, GeosetExtent, Model, RawChunk};
use wc3_mdx::{ModelChunk, Record};

fn sample_geoset() -> Geoset {
    let geoset = Geoset::new(1800, &[[1.0, 2.0, 3.0]], &[[0.0, 0.0, 1.0]], &[0, 0, 0]).unwrap();
    Geoset::decode(&geoset.encode().unwrap(), 1800).unwrap()
}

#[test]
fn geoset_mesh_edit_preserves_other_sections() {
    let mut geoset = sample_geoset();
    assert_eq!(geoset.vertices(), vec![[1.0, 2.0, 3.0]]);
    assert_eq!(geoset.normals(), vec![[0.0, 0.0, 1.0]]);
    assert_eq!(geoset.face_indices(), vec![0, 0, 0]);
    geoset.set_vertex(0, [4.0, 5.0, 6.0]).unwrap();
    let mut model = Model::new(1800);
    model.set_geosets(&[geoset]).unwrap();
    let decoded = Model::decode(&model.encode().unwrap(), 800).unwrap();
    assert_eq!(
        decoded.geosets().unwrap()[0].vertices(),
        vec![[4.0, 5.0, 6.0]]
    );
    assert_eq!(
        decoded.geosets().unwrap()[0].normals(),
        vec![[0.0, 0.0, 1.0]]
    );
}

#[test]
fn rejects_invalid_geoset_sizes() {
    let mut model = Model::new(800);
    model.push(ModelChunk::from_raw(
        RawChunk::new(*b"GEOS", 100u32.to_le_bytes().to_vec()),
        800,
    ));
    assert_eq!(
        model.geosets(),
        Err(Error::UnexpectedEnd {
            offset: 4,
            needed: 96
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
        geoset.set_material_id(7);
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
        geoset.set_selection_group(3);
        geoset.set_unselectable(true);
        geoset.set_uv(0, 1, [0.25, 0.75]).unwrap();
        let extent = GeosetExtent {
            bounds_radius: 5.0,
            minimum: [-1.0; 3],
            maximum: [1.0; 3],
        };
        geoset.set_extent(extent);
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
            assert_eq!(geoset.tangents().unwrap().len(), 2);
            assert_eq!(geoset.skin_weights(), Some(weights.as_slice()));
            if version >= 1200 {
                assert_eq!(geoset.skin_bone_indices(), Some(indices.as_slice()));
            }
            geoset.set_skin_data(None, None).unwrap();
            geoset.set_tangents(None).unwrap();
            assert!(geoset.skin_weights().is_none());
            assert!(geoset.tangents().is_none());
            assert_eq!(geoset.name().as_deref(), Some("Body"));
        }
        assert_eq!(geoset.material_id(), 7);
        assert_eq!(geoset.normals()[1], [0.0, 1.0, 0.0]);
        assert_eq!(geoset.selection_group(), 3);
        assert!(geoset.unselectable());
        assert_eq!(geoset.extent(), extent);
        assert_eq!(geoset.vertex_groups(), &[0, 1]);
        assert_eq!(geoset.matrix_group_sizes(), vec![2, 1]);
        assert_eq!(geoset.matrix_indices(), vec![1, 2, 3]);
        assert_eq!(geoset.sequence_extents().len(), 1);
        assert_eq!(
            geoset.uv_sets(),
            vec![vec![[0.0, 0.0], [0.25, 0.75]], vec![[1.0, 1.0]; 2]]
        );
        assert_eq!(
            Geoset::decode(&geoset.encode().unwrap(), version).unwrap(),
            geoset
        );
        let mut model = Model::new(version);
        model.set_geosets(&[geoset]).unwrap();
        let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
        assert_eq!(parsed.geosets().unwrap()[0].version(), version);
        assert_eq!(parsed.geosets().unwrap()[0].material_id(), 7);
    }
}

#[test]
fn preserves_float_bits_name_padding_and_extension_order() {
    let mut geoset = sample_geoset();
    geoset.set_tangents(Some(&[[1.0, 0.0, 0.0, 1.0]])).unwrap();
    geoset
        .set_skin_data(Some(&[1, 2, 3, 4]), Some(&[5, 6, 7, 8]))
        .unwrap();
    let mut bytes = geoset.encode().unwrap();
    bytes[12..16].copy_from_slice(&0x7fa1_2345u32.to_le_bytes());
    let mats = bytes.windows(4).position(|part| part == b"MATS").unwrap();
    let count = u32::from_le_bytes(bytes[mats + 4..mats + 8].try_into().unwrap()) as usize;
    let name = mats + 8 + count * 4 + 16;
    bytes[name + 5] = 0xab;
    let tang = bytes.windows(4).position(|part| part == b"TANG").unwrap();
    let skin = bytes.windows(4).position(|part| part == b"SKIN").unwrap();
    let uv = bytes.windows(4).position(|part| part == b"UVAS").unwrap();
    assert!(tang < skin && skin < uv);
    let mut reordered = bytes[..tang].to_vec();
    reordered.extend_from_slice(&bytes[skin..uv]);
    reordered.extend_from_slice(&bytes[tang..skin]);
    reordered.extend_from_slice(&bytes[uv..]);
    let decoded = Geoset::decode(&reordered, 1800).unwrap();
    assert_eq!(decoded.encode().unwrap(), reordered);
    assert_eq!(decoded.vertices()[0][0].to_bits(), 0x7fa1_2345);
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
                let model = Model::decode(&bytes, 800).unwrap();
                for geoset in model.geosets().unwrap() {
                    geoset.vertices();
                    geoset.normals();
                    geoset.face_indices();
                    geoset.vertex_groups();
                    geoset.matrix_group_sizes();
                    geoset.matrix_indices();
                    geoset.extent();
                    geoset.sequence_extents();
                    geoset.tangents();
                    geoset.skin_weights();
                    geoset.skin_bone_indices();
                    geoset.uv_sets();
                }
            }
        }
    }
}
