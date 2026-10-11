use wc3::model::geometry::{Geoset, GeosetExtent, SkinWeight, SkinWeights};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;
use wc3::model::AsULE;
use wc3::model::ZeroVec;

use wc3::model::{Model, ModelVersion, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900};

fn sample_geoset() -> Geoset<'static, V1800> {
    let geoset = Geoset::<V1800>::new(&[[1.0, 2.0, 3.0]], &[[0.0, 0.0, 1.0]], &[0, 0, 0]).unwrap();
    Geoset::<V1800>::decode_mdx(&geoset.encode_mdx().unwrap())
        .unwrap()
        .into_owned()
}

#[test]
fn geoset_mesh_edit_preserves_other_sections() {
    let mut geoset = sample_geoset();
    assert_eq!(geoset.vertices().to_vec(), vec![[1.0, 2.0, 3.0]]);
    assert_eq!(geoset.normals().to_vec(), vec![[0.0, 0.0, 1.0]]);
    assert_eq!(geoset.face_indices().to_vec(), vec![0, 0, 0]);
    geoset.set_vertex(0, [4.0, 5.0, 6.0]).unwrap();
    let mut model = Model::<V1800>::new();
    model.set_geosets(&[geoset]);
    let decoded = Model::<V1800>::decode_mdx(&model.encode_mdx().unwrap())
        .unwrap()
        .into_owned();
    assert_eq!(
        decoded.geosets().next().unwrap().vertices().to_vec(),
        vec![[4.0, 5.0, 6.0]]
    );
    assert_eq!(
        decoded.geosets().next().unwrap().normals().to_vec(),
        vec![[0.0, 0.0, 1.0]]
    );
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
    .into_iter()
    .collect();
    geoset.set_uv_sets(
        ([vec![[0.0, 0.0]; 2], vec![[1.0, 1.0]; 2]])
            .iter()
            .map(|set| ZeroVec::alloc_from_slice(set))
            .collect(),
    );
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
            .try_set_tangents(Some(ZeroVec::alloc_from_slice(&[[1.0, 0.0, 0.0, 1.0]; 2])))
            .unwrap();
        let weights = [V::SkinWeights::from_influences([1, 2, 3, 4], [255, 0, 0, 0]).unwrap(); 2];
        geoset
            .try_set_skin_weights(Some(ZeroVec::alloc_from_slice(&weights)))
            .unwrap();
        assert_eq!(geoset.try_tangents().unwrap().unwrap().len(), 2);
        assert_eq!(
            geoset
                .try_skin_weights()
                .unwrap()
                .map(|weights| weights.to_vec()),
            Some(weights.to_vec())
        );
        assert_eq!(
            Geoset::<V>::decode_mdx(&geoset.encode_mdx().unwrap())
                .unwrap()
                .into_owned(),
            geoset
        );
        geoset.try_set_skin_weights(None).unwrap();
        geoset.try_set_tangents(None).unwrap();
        assert!(geoset.try_skin_weights().unwrap().is_none());
        assert!(geoset.try_tangents().unwrap().is_none());
        assert_eq!(geoset.try_name().unwrap().as_ref(), "Body");
    }
    assert_eq!(geoset.material_id, 7);
    assert_eq!(geoset.normals().get(1).unwrap(), [0.0, 1.0, 0.0]);
    assert_eq!(geoset.selection_group, 3);
    assert!(geoset.unselectable());
    assert_eq!(geoset.extent, extent);
    assert_eq!(geoset.vertex_groups(), &[0, 1]);
    assert_eq!(geoset.matrix_group_sizes().to_vec(), vec![2, 1]);
    assert_eq!(geoset.matrix_indices().to_vec(), vec![1, 2, 3]);
    assert_eq!(geoset.sequence_extents.len(), 1);
    assert_eq!(
        geoset
            .uv_sets()
            .iter()
            .map(|set| set.to_vec())
            .collect::<Vec<_>>(),
        vec![vec![[0.0, 0.0], [0.25, 0.75]], vec![[1.0, 1.0]; 2]]
    );
    assert_eq!(
        Geoset::<V>::decode_mdx(&geoset.encode_mdx().unwrap())
            .unwrap()
            .into_owned(),
        geoset
    );
    let mut model = Model::<V>::new();
    model.set_geosets(&[geoset]);
    let parsed = Model::<V>::decode_mdx(&model.encode_mdx().unwrap())
        .unwrap()
        .into_owned();
    assert_eq!(parsed.geosets().next().unwrap().version(), version);
    assert_eq!(parsed.geosets().next().unwrap().material_id, 7);
}

#[test]
fn preserves_float_bits_name_padding_and_extension_order() {
    let mut geoset = sample_geoset();
    geoset
        .try_set_tangents(Some(ZeroVec::alloc_from_slice(&[[1.0, 0.0, 0.0, 1.0]])))
        .unwrap();
    geoset
        .try_set_skin_weights(Some(ZeroVec::alloc_from_slice(&[SkinWeights {
            bone_indices: [5, 6, 7, 8],
            weights: [1, 2, 3, 249],
        }])))
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
    assert_eq!(decoded.vertices().get(0).unwrap()[0].to_bits(), 0x7fa1_2345);
}

#[test]
fn skin_elements_widen_at_v1400() {
    let weights = [SkinWeights {
        bone_indices: [1, 2, 3, 255],
        weights: [128, 64, 32, 31],
    }];
    let mut narrow = Geoset::<V1300>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
    narrow
        .set_skin_weights(Some(ZeroVec::alloc_from_slice(&weights)))
        .unwrap();
    let mut wide = Geoset::<V1400>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
    wide.set_skin_weights(Some(ZeroVec::alloc_from_slice(&weights.map(|row| {
        SkinWeights::<u16>::from_influences(row.bone_indices(), row.weights()).unwrap()
    }))))
    .unwrap();
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
    let weights = [SkinWeights::<u16> {
        bone_indices: [256, 1000, 65535, 0],
        weights: [255, 0, 0, 0],
    }];
    let mut classic = Geoset::<V800>::new(&[], &[], &[]).unwrap();
    assert!(classic
        .try_set_skin_weights(Some(ZeroVec::alloc_from_slice(&[])))
        .is_err());
    assert!(classic.try_skin_weights().is_err());
    let narrow = Geoset::<V1300>::new(&[], &[], &[]).unwrap();
    assert!(
        SkinWeights::<u8>::from_influences(weights[0].bone_indices, weights[0].weights).is_none()
    );
    assert!(narrow.skin_weights().is_none());
    let mut wide = Geoset::<V1400>::new(&[], &[], &[]).unwrap();
    wide.set_skin_weights(Some(ZeroVec::alloc_from_slice(&weights)))
        .unwrap();
    assert_eq!(
        Geoset::<V1400>::decode_mdx(&wide.encode_mdx().unwrap())
            .unwrap()
            .into_owned(),
        wide
    );
}

#[test]
fn rejects_incomplete_skin_vertices_and_out_of_range_wide_weights() {
    let mut geoset = sample_geoset();
    geoset
        .set_skin_weights(Some(ZeroVec::alloc_from_slice(&[SkinWeights {
            bone_indices: [1; 4],
            weights: [255, 0, 0, 0],
        }])))
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

#[test]
fn skin_arrays_borrow_their_version_selected_wire_layout() {
    fn check<V: ModelVersion>(width: usize) {
        let mut geoset = Geoset::<V>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
        geoset
            .try_set_skin_weights(Some(ZeroVec::alloc_from_slice(&[
                V::SkinWeights::from_influences([1, 2, 3, 4], [128, 64, 32, 31]).unwrap(),
            ])))
            .unwrap();
        let mut bytes = vec![0];
        bytes.extend(geoset.encode_mdx().unwrap());
        let input = &bytes[1..];
        let decoded = Geoset::<V>::decode_mdx(input).unwrap();
        let skin = decoded.try_skin_weights().unwrap().unwrap();
        assert!(!skin.is_owned());

        let encoded = skin.as_bytes();
        assert_eq!(encoded.len(), width);
        let offset = input.windows(4).position(|tag| tag == b"SKIN").unwrap() + 8;
        assert_eq!(encoded.as_ptr(), input[offset..].as_ptr());
        assert_eq!(skin.get(0).unwrap().bone_indices(), [1, 2, 3, 4]);
        assert_eq!(skin.get(0).unwrap().weights(), [128, 64, 32, 31]);
        assert_eq!(decoded.encode_mdx().unwrap(), input);
        let mut edited = skin.clone();
        edited.to_mut_slice()[0] = V::SkinWeights::from_influences([4, 3, 2, 1], [255, 0, 0, 0])
            .unwrap()
            .to_unaligned();
        assert!(edited.is_owned());
        assert_eq!(skin.get(0).unwrap().weights(), [128, 64, 32, 31]);
    }
    check::<V900>(8);
    check::<V1300>(8);
    check::<V1400>(16);
    check::<V1800>(16);
}

#[test]
fn wide_skin_weights_validate_the_normalized_weight_range() {
    let mut geoset = Geoset::<V1400>::new(&[[0.0; 3]], &[[0.0; 3]], &[]).unwrap();
    let invalid = SkinWeights::<u16> {
        bone_indices: [1, 0, 0, 0],
        weights: [256, 0, 0, 0],
    };
    assert!(geoset
        .set_skin_weights(Some(ZeroVec::alloc_from_slice(&[invalid])))
        .is_err());
    assert!(geoset.skin_weights().is_none());
    geoset
        .set_skin_weights(Some(ZeroVec::alloc_from_slice(&[SkinWeights {
            bone_indices: [1, 0, 0, 0],
            weights: [255, 0, 0, 0],
        }])))
        .unwrap();
    let mut bytes = geoset.encode_mdx().unwrap();
    let offset = bytes.windows(4).position(|tag| tag == b"SKIN").unwrap() + 16;
    bytes[offset..offset + 2].copy_from_slice(&256u16.to_le_bytes());
    assert!(Geoset::<V1400>::decode_mdx(&bytes).is_err());
}
