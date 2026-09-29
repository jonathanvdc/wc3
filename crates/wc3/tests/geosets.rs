use wc3::model::geometry::{Geoset, GeosetExtent, SkinWeights};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::{
    DynamicModel, Model, ModelVersion, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};

fn sample_geoset() -> Geoset<V1800> {
    let geoset = Geoset::<V1800>::new(&[[1.0, 2.0, 3.0]], &[[0.0, 0.0, 1.0]], &[0, 0, 0]).unwrap();
    Geoset::<V1800>::decode_mdx(&geoset.encode_mdx().unwrap()).unwrap()
}

#[test]
fn geoset_mesh_edit_preserves_other_sections() {
    let mut geoset = sample_geoset();
    assert_eq!(geoset.vertices(), vec![[1.0, 2.0, 3.0]]);
    assert_eq!(geoset.normals(), vec![[0.0, 0.0, 1.0]]);
    assert_eq!(geoset.face_indices(), vec![0, 0, 0]);
    geoset.set_vertex(0, [4.0, 5.0, 6.0]).unwrap();
    let mut model = Model::<V1800>::new();
    model.set_geosets(&[geoset]);
    let decoded = Model::<V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    assert_eq!(decoded.geosets()[0].vertices(), vec![[4.0, 5.0, 6.0]]);
    assert_eq!(decoded.geosets()[0].normals(), vec![[0.0, 0.0, 1.0]]);
}

#[test]
fn builds_complete_synthetic_geosets() {
    check_version::<V800>();
    check_version::<V900>();
    check_version::<V1100>();
    check_version::<V1200>();
    check_version::<V1300>();
    check_version::<V1400>();
    check_version::<V1600>();
    check_version::<V1800>();
}

fn check_version<V: ModelVersion>() {
    let version = V::NUMBER;
    let mut geoset = Geoset::<V>::new(
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        &[[0.0, 0.0, 1.0]; 2],
        &[0, 1, 0],
    )
    .unwrap();
    geoset.material_id = 7;
    geoset.set_matrix_groups(&[vec![1, 2], vec![3]]).unwrap();
    geoset.set_vertex_groups(&[0, 1]).unwrap();
    geoset.sequence_extents = [GeosetExtent {
        bounds_radius: 2.0,
        minimum: [-2.0; 3],
        maximum: [2.0; 3],
    }]
    .to_vec();
    geoset.set_uv_sets(&[vec![[0.0, 0.0]; 2], vec![[1.0, 1.0]; 2]]);
    geoset.set_normal(1, [0.0, 1.0, 0.0]).unwrap();
    geoset.selection_group = 3;
    geoset.set_unselectable(true);
    geoset.set_uv(0, 1, [0.25, 0.75]).unwrap();
    let extent = GeosetExtent {
        bounds_radius: 5.0,
        minimum: [-1.0; 3],
        maximum: [1.0; 3],
    };
    geoset.extent = extent;
    if version >= 900 {
        geoset.try_set_level_of_detail(2).unwrap();
        geoset.try_set_name("Body").unwrap();
        geoset
            .try_set_tangents(Some(&[[1.0, 0.0, 0.0, 1.0]; 2]))
            .unwrap();
        let weights = [SkinWeights {
            bone_indices: [1, 2, 3, 4],
            weights: [255, 0, 0, 0],
        }; 2];
        geoset.try_set_skin_weights(Some(&weights)).unwrap();
        assert_eq!(geoset.try_tangents().unwrap().unwrap().len(), 2);
        assert_eq!(geoset.try_skin_weights().unwrap(), Some(weights.as_slice()));
        assert_eq!(
            Geoset::<V>::decode_mdx(&geoset.encode_mdx().unwrap()).unwrap(),
            geoset
        );
        geoset.try_set_skin_weights(None).unwrap();
        geoset.try_set_tangents(None).unwrap();
        assert!(geoset.try_skin_weights().unwrap().is_none());
        assert!(geoset.try_tangents().unwrap().is_none());
        assert_eq!(geoset.try_name().unwrap().as_ref(), "Body");
    }
    assert_eq!(geoset.material_id, 7);
    assert_eq!(geoset.normals()[1], [0.0, 1.0, 0.0]);
    assert_eq!(geoset.selection_group, 3);
    assert!(geoset.unselectable());
    assert_eq!(geoset.extent, extent);
    assert_eq!(geoset.vertex_groups(), &[0, 1]);
    assert_eq!(geoset.matrix_group_sizes(), vec![2, 1]);
    assert_eq!(geoset.matrix_indices(), vec![1, 2, 3]);
    assert_eq!(geoset.sequence_extents.len(), 1);
    assert_eq!(
        geoset.uv_sets(),
        vec![vec![[0.0, 0.0], [0.25, 0.75]], vec![[1.0, 1.0]; 2]]
    );
    assert_eq!(
        Geoset::<V>::decode_mdx(&geoset.encode_mdx().unwrap()).unwrap(),
        geoset
    );
    let mut model = Model::<V>::new();
    model.set_geosets(&[geoset]);
    let parsed = Model::<V>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    assert_eq!(parsed.geosets()[0].version(), version);
    assert_eq!(parsed.geosets()[0].material_id, 7);
}

#[test]
fn preserves_float_bits_name_padding_and_extension_order() {
    let mut geoset = sample_geoset();
    geoset
        .try_set_tangents(Some(&[[1.0, 0.0, 0.0, 1.0]]))
        .unwrap();
    geoset
        .try_set_skin_weights(Some(&[SkinWeights {
            bone_indices: [5, 6, 7, 8],
            weights: [1, 2, 3, 249],
        }]))
        .unwrap();
    let mut bytes = geoset.encode_mdx().unwrap();
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
    let decoded = Geoset::<V1800>::decode_mdx(&reordered).unwrap();
    assert_eq!(decoded.encode_mdx().unwrap(), reordered);
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
                let model = DynamicModel::decode_mdx(&bytes, 800).unwrap();
                fn check<V: ModelVersion>(model: Model<V>) {
                    for geoset in model.geosets() {
                        geoset.vertices();
                        geoset.normals();
                        geoset.face_indices();
                        geoset.vertex_groups();
                        geoset.matrix_group_sizes();
                        geoset.matrix_indices();
                        geoset.sequence_extents.as_slice();
                        let _ = geoset.try_tangents();
                        let _ = geoset.try_skin_weights();
                        geoset.uv_sets();
                    }
                }
                match model {
                    DynamicModel::V800(model) => check(model),
                    DynamicModel::V900(model) => check(model),
                    DynamicModel::V1000(model) => check(model),
                    DynamicModel::V1100(model) => check(model),
                    DynamicModel::V1200(model) => check(model),
                    DynamicModel::V1300(model) => check(model),
                    DynamicModel::V1400(model) => check(model),
                    DynamicModel::V1600(model) => check(model),
                    DynamicModel::V1800(model) => check(model),
                }
            }
        }
    }
}

#[test]
fn skin_elements_widen_at_v1400() {
    let weights = [SkinWeights {
        bone_indices: [1, 2, 3, 255],
        weights: [128, 64, 32, 31],
    }];
    let mut narrow = Geoset::<V1300>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
    narrow.set_skin_weights(Some(&weights)).unwrap();
    let mut wide = Geoset::<V1400>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
    wide.set_skin_weights(Some(&weights)).unwrap();
    let narrow_bytes = narrow.encode_mdx().unwrap();
    let wide_bytes = wide.encode_mdx().unwrap();
    for (bytes, payload) in [
        (&narrow_bytes, vec![1, 2, 3, 255, 128, 64, 32, 31]),
        (
            &wide_bytes,
            vec![1, 0, 2, 0, 3, 0, 255, 0, 128, 0, 64, 0, 32, 0, 31, 0],
        ),
    ] {
        let skin = bytes.windows(4).position(|part| part == b"SKIN").unwrap();
        assert_eq!(&bytes[skin + 4..skin + 8], &8u32.to_le_bytes());
        assert_eq!(&bytes[skin + 8..skin + 8 + payload.len()], payload);
        assert_eq!(
            &bytes[skin + 8 + payload.len()..skin + 12 + payload.len()],
            b"UVAS"
        );
    }
    assert_eq!(Geoset::<V1300>::decode_mdx(&narrow_bytes).unwrap(), narrow);
    assert_eq!(Geoset::<V1400>::decode_mdx(&wide_bytes).unwrap(), wide);
}

#[test]
fn skin_version_gates_and_wide_indices() {
    let weights = [SkinWeights {
        bone_indices: [256, 1000, 65535, 0],
        weights: [255, 0, 0, 0],
    }];
    let mut classic = Geoset::<V800>::new(&[], &[], &[]).unwrap();
    assert!(classic.try_set_skin_weights(Some(&weights)).is_err());
    assert!(classic.try_skin_weights().is_err());
    let mut narrow = Geoset::<V1300>::new(&[], &[], &[]).unwrap();
    assert!(narrow.set_skin_weights(Some(&weights)).is_err());
    assert!(narrow.skin_weights().is_none());
    let mut wide = Geoset::<V1400>::new(&[], &[], &[]).unwrap();
    wide.set_skin_weights(Some(&weights)).unwrap();
    assert_eq!(
        Geoset::<V1400>::decode_mdx(&wide.encode_mdx().unwrap()).unwrap(),
        wide
    );
}

#[test]
fn rejects_incomplete_skin_vertices_and_out_of_range_wide_weights() {
    let mut geoset = sample_geoset();
    geoset
        .set_skin_weights(Some(&[SkinWeights {
            bone_indices: [1; 4],
            weights: [255, 0, 0, 0],
        }]))
        .unwrap();
    let bytes = geoset.encode_mdx().unwrap();
    let skin = bytes.windows(4).position(|part| part == b"SKIN").unwrap();
    let mut invalid_weight = bytes.clone();
    invalid_weight[skin + 16..skin + 18].copy_from_slice(&256u16.to_le_bytes());
    assert!(Geoset::<V1800>::decode_mdx(&invalid_weight).is_err());
    let mut invalid_count = bytes;
    invalid_count[skin + 4..skin + 8].copy_from_slice(&7u32.to_le_bytes());
    assert!(Geoset::<V1800>::decode_mdx(&invalid_count).is_err());
}
