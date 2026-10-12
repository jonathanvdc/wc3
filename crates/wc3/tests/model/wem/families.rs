use std::fmt::Debug;
use wc3::model::wem::records::*;
use wc3::model::wem::{
    self, Decoder, Encoder, InlineList, NativeAbi, ReadErrorKind, Record, RecordLayout, Validate,
    VisitReferences,
};
use wc3::model::wem::{Read as _, Write as _};
fn check<T>()
where
    T: Default
        + Debug
        + PartialEq
        + Record
        + RecordLayout
        + Validate
        + VisitReferences
        + wem::Write
        + for<'a> wem::Read<'a>,
{
    let value = T::default();
    check_value(&value);
}
fn check_value<T>(value: &T)
where
    T: Debug
        + PartialEq
        + Record
        + RecordLayout
        + Validate
        + VisitReferences
        + wem::Write
        + for<'a> wem::Read<'a>,
{
    let mut output = Encoder::new(T::VERSION).with_native_abi(T::NATIVE_ABI);
    value.write(&mut output).unwrap();
    let bytes = output.finish();
    let mut input = Decoder::new(&bytes, T::VERSION).with_native_abi(T::NATIVE_ABI);
    let decoded = T::read(&mut input).unwrap();
    assert_eq!(input.remaining(), 0);
    assert_eq!(&decoded, value);
    decoded.validate().unwrap();
    let mut input = Decoder::new(&bytes, T::VERSION).with_native_abi(T::NATIVE_ABI);
    T::skip(&mut input).unwrap();
    assert_eq!(input.remaining(), 0);
    assert_eq!(
        known_record_span(T::TAG, T::VERSION, &bytes, 1, T::NATIVE_ABI).unwrap(),
        Some(bytes.len())
    );
    let repeated = [bytes.as_slice(), bytes.as_slice()].concat();
    assert_eq!(
        known_record_span(T::TAG, T::VERSION, &repeated, 2, T::NATIVE_ABI).unwrap(),
        Some(repeated.len())
    );
    validate_known_section(
        T::TAG,
        T::VERSION,
        &bytes,
        1,
        T::NATIVE_ABI,
        &mut |_| Ok(()),
    )
    .unwrap();
    if !bytes.is_empty() {
        assert!(known_record_span(
            T::TAG,
            T::VERSION,
            &bytes[..bytes.len() - 1],
            1,
            T::NATIVE_ABI
        )
        .is_err());
    }
}
#[test]
fn all_supported_native_families_read_write_and_scan_their_rows() {
    check::<Character>();
    check::<Byte>();
    check::<U16>();
    check::<U32>();
    check::<U64>();
    check::<I16>();
    check::<I32>();
    check::<Real>();
    check::<Vector2>();
    check::<Vector3>();
    check::<Vector4>();
    check::<QuaternionRecord>();
    check::<Extent>();
    check::<Byte4>();
    check::<DocumentRecord<V1>>();
    check::<ModelRecord<V1>>();
    check::<ModelRecord<V2>>();
    check::<ModelRecord<V3>>();
    check::<ModelRecord<V4>>();
    check::<ModelRecord<V5>>();
    check::<ModelRecord<V6>>();
    check::<ModelRecord<V7>>();
    check::<ModelRecord<V8>>();
    check::<ModelRecord<V9>>();
    check::<ProfileMaterialSet>();
    check::<SlotBinding>();
    check::<Mesh<V1>>();
    check::<Mesh<V2>>();
    check::<MeshSection>();
    check::<AttrLayer>();
    check::<Influence>();
    check::<VertexSplit>();
    check::<FaceRecord>();
    check::<Transform>();
    check::<Matrix44>();
    check::<Node<V1>>();
    check::<Node<V2>>();
    check::<Node<V3>>();
    check::<Node<V4>>();
    check::<Node<V5>>();
    check::<Node<V6>>();
    check::<Node<V7>>();
    check::<Node<V8>>();
    check::<Node<V9>>();
    check::<Node<V10>>();
    check::<Node<V11>>();
    check::<Node<V12>>();
    check::<Node<V13>>();
    check::<Node<V14>>();
    check::<Node<V15>>();
    check::<Node<V16>>();
    check::<Node<V17>>();
    check::<Node<V18>>();
    check::<Node<V19>>();
    check::<TPoseSourceRecord>();
    check::<PoseDelta>();
    check::<TestPose>();
    check::<AssetKey>();
    check::<Sc2Property<[f32; 3]>>();
    check::<Sc2RibbonSplinePoint>();
    check::<NativeEntry<V1>>();
    check::<NativeEntry<V2>>();
    check::<PoseSchema<V1>>();
    check::<PoseSchema<V2>>();
    check::<MaterialRecord>();
    check::<MaterialFeature>();
    check::<FresnelFeature>();
    check::<UvAnimationFeature>();
    check::<LayerShadingFeature>();
    check::<CompositeBody<V1>>();
    check::<CompositeBody<V2>>();
    check::<CompositeLayer>();
    check::<CombinersBody>();
    check::<CombinerStage>();
    check::<LegacyDeferredBody>();
    check::<PbrDeferredBody>();
    check::<TextureRecord>();
    check::<Look>();
    check::<AnimChannel>();
    check::<SubTrack<V1>>();
    check::<SubTrack<V2>>();
    check::<SubTrackContainer>();
    check::<ClipEvent>();
    check::<Clip<V1>>();
    check::<Clip<V2>>();
    check::<Clip<V3>>();
    check::<Clip<V4>>();
    check::<Clip<V5>>();
    check::<Clip<V6>>();
    check::<Clip<V7>>();
    check::<BakeWorldForce>();
    check::<TrackSet>();
    check::<PoseStage<V1>>();
    check::<PoseStage<V2>>();
    check::<StageSource>();
    check::<PhysicsBody<V1>>();
    check::<PhysicsBody<V2>>();
    check::<PhysicsBody<V3>>();
    check::<PhysicsShape<V1>>();
    check::<PhysicsShape<V2>>();
    check::<PhysicsJoint<V1>>();
    check::<PhysicsJoint<V2>>();
    check::<PhysicsJoint<V3>>();
    check::<ClothCollider<V1>>();
    check::<ClothCollider<V2>>();
    check::<Cloth<V1>>();
    check::<Cloth<V2>>();
    check::<ClothBinding>();
    check::<PhysicsRig<V1>>();
    check::<PhysicsRig<V2>>();
    check::<PhysicsRig<V3>>();
    check::<PhysicsRig<V4>>();
    check::<PhysicsRig<V5>>();
    check::<PhysicsRig<V6>>();
    check::<PhysicsRig<V7>>();
    check::<PhysicsRig<V8>>();
    check::<SoftBody>();
    check::<SoftLump>();
    check::<SoftEffect>();
    check::<SoftPart>();
    check::<ClipTrackSet>();
    check::<AnimTag>();
    check::<AnimSet>();
    check::<MdxMaterial>();
    check::<M2Material>();
    check::<D3Material<V1>>();
    check::<D3Material<V2>>();
    check::<M3Material<V1, Abi32>>();
    check::<M3Material<V1, Abi64>>();
}

#[test]
fn inline_profile_and_slot_runs_are_nonempty_and_references_keep_their_order() {
    let document = DocumentRecord::<V1> {
        profiles: InlineList(vec![ProfileId::GENERIC, ProfileId::WOW]),
        ..Default::default()
    };
    check_value(&document);
    let model = ModelRecord::<V9> {
        material_slots: InlineList(vec![wem::Ref::new(4, 12, 93), wem::Ref::new(5, 14, 97)]),
        ..Default::default()
    };
    check_value(&model);
    let mut output = Encoder::new(9);
    model.write(&mut output).unwrap();
    let bytes = output.finish();
    let mut active = Vec::new();
    validate_known_section(*b"MODL", 9, &bytes, 1, None, &mut |reference| {
        if reference.entries != 0 {
            active.push(reference);
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(active.len(), 2);
    assert_eq!((active[0].target, active[0].index), (Some(*b"CHAR"), 12));
    assert_eq!(active[1].index, 14);
    let material = D3Material::<V2> {
        shader_params: InlineList(vec![D3TagValue {
            value_type: 17,
            tag_id: 0xa0013,
            value: 0xfedcba98,
        }]),
        ..Default::default()
    };
    check_value(&material);
}
fn word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
#[test]
fn literal_node_headers_distinguish_revision_one_and_two_matrix_slots() {
    // NODE1 ends the fixed header at86, then stores helper-kind0 and NBAG reference12.
    let mut old = vec![0u8; 99];
    old[73] = 0x7f;
    word(&mut old, 74, 5);
    word(&mut old, 78, 44);
    word(&mut old, 82, 12);
    let value = Node::<V1>::read(&mut Decoder::new(&old, 1)).unwrap();
    assert_eq!(value.uniform_scale_only.0, 0x7f);
    assert_eq!(
        (value.poses.entries, value.poses.index, value.poses.flags),
        (5, 44, 12)
    );
    assert_eq!(value.pose_matrices, ());
    assert_eq!(
        known_record_span(*b"NODE", 1, &old, 1, None).unwrap(),
        Some(99)
    );
    let mut newer = vec![0u8; 111];
    newer[..86].copy_from_slice(&old[..86]);
    word(&mut newer, 86, 2);
    word(&mut newer, 90, 53);
    word(&mut newer, 94, 90);
    let value = Node::<V2>::read(&mut Decoder::new(&newer, 2)).unwrap();
    assert_eq!(
        (
            value.pose_matrices.entries,
            value.pose_matrices.index,
            value.pose_matrices.flags
        ),
        (2, 53, 90)
    );
    assert_eq!(
        known_record_span(*b"NODE", 2, &newer, 1, None).unwrap(),
        Some(111)
    );
    assert!(known_record_span(*b"NODE", 2, &newer[..110], 1, None).is_err());
}
#[test]
fn literal_attribute_fields_keep_the_two_byte_domains_and_four_byte_storage_code() {
    let mut bytes = vec![0u8; 30];
    word(&mut bytes, 0, 3);
    word(&mut bytes, 4, 7);
    word(&mut bytes, 8, 9);
    bytes[12] = 1;
    bytes[13] = 2;
    word(&mut bytes, 14, 0x11223344);
    word(&mut bytes, 18, 12);
    word(&mut bytes, 22, 23);
    word(&mut bytes, 26, 29);
    let value = AttrLayer::read(&mut Decoder::new(&bytes, 1)).unwrap();
    assert_eq!(value.domain, Domain::HALFEDGE);
    assert_eq!(value.type_, AttrType::F32X3);
    assert_eq!(value.storage.0, 0x11223344);
    assert_eq!(
        (value.data.entries, value.data.index, value.data.flags),
        (12, 23, 29)
    );
    let mut seen = Vec::new();
    validate_known_section(*b"ATTR", 1, &bytes, 1, None, &mut |r| {
        seen.push(r);
        Ok(())
    })
    .unwrap();
    assert_eq!(seen[0].target, Some(*b"CHAR"));
    assert_eq!(seen[1].target, Some(*b"U8__"));
}
#[test]
fn explicit_native_material_abi_rejects_a_conflicting_codec_context() {
    let native = M3Material::<V1, Abi64>::default();
    let mut output = Encoder::new(1).with_native_abi(Some(NativeAbi::Abi32));
    assert!(native.write(&mut output).is_err());
    let mut output = Encoder::new(1);
    native.write(&mut output).unwrap();
    let bytes = output.finish();
    let mut input = Decoder::new(&bytes, 1).with_native_abi(Some(NativeAbi::Abi32));
    assert!(M3Material::<V1, Abi64>::read(&mut input).is_err());
    assert_eq!(
        known_record_span(*b"NM3_", 1, &bytes, 1, None).unwrap(),
        None
    );
    assert_eq!(
        known_record_span(*b"NM3_", 1, &bytes, 1, Some(NativeAbi::Abi64)).unwrap(),
        Some(bytes.len())
    );
}
#[test]
fn an_unknown_native_material_body_index_is_an_unsupported_layout() {
    let mut bytes = vec![0u8; 19];
    bytes[9..17].copy_from_slice(&99u64.to_le_bytes());
    let mut input = Decoder::new(&bytes, 1);
    let error = M3Material::<V1, Abi64>::read(&mut input).unwrap_err();
    assert_eq!(error.kind, ReadErrorKind::UnsupportedLayout);
    assert_eq!(
        known_record_span(*b"NM3_", 1, &bytes, 1, Some(NativeAbi::Abi64)).unwrap(),
        None
    );
}

#[test]
fn literal_soft_lump_has_unaligned_node_words_and_two_preserved_bool_bytes() {
    let mut bytes = vec![0u8; 49];
    bytes[0] = 0x91;
    bytes[1..5].copy_from_slice(&1.25f32.to_le_bytes());
    word(&mut bytes, 33, 7);
    word(&mut bytes, 37, 11);
    word(&mut bytes, 41, 13);
    bytes[45..47].copy_from_slice(&17u16.to_le_bytes());
    bytes[47] = 0x7f;
    bytes[48] = 0xfe;
    let lump = SoftLump::read(&mut Decoder::new(&bytes, 1)).unwrap();
    assert_eq!(lump.kind.0, 0x91);
    assert_eq!(lump.anchor[0], 1.25);
    assert_eq!((lump.carrier, lump.node, lump.effect), (7, 11, 13));
    assert_eq!(lump.rider, 17);
    assert_eq!((lump.swings.0, lump.rigid.0), (0x7f, 0xfe));
    let mut output = Encoder::new(1);
    lump.write(&mut output).unwrap();
    assert_eq!(output.finish(), bytes);
    assert_eq!(
        known_record_span(*b"SLMP", 1, &bytes, 1, None).unwrap(),
        Some(49)
    );
}

#[test]
fn literal_lod_settings_store_three_scalar_ratios_not_three_arrays() {
    let mut bytes = Vec::new();
    bytes.push(0x7f);
    for ratio in [1.0f32, 0.88, 0.71] {
        bytes.extend_from_slice(&ratio.to_le_bytes());
    }
    bytes.extend_from_slice(&[0xfe, 0x55]);
    assert_eq!(bytes.len(), 15);
    let settings = LodExport::read(&mut Decoder::new(&bytes, 4)).unwrap();
    assert_eq!(
        (settings.ratio1, settings.ratio2, settings.ratio3),
        (1.0, 0.88, 0.71)
    );
    assert_eq!(
        (
            settings.generate.0,
            settings.lock_borders.0,
            settings.source_had_levels.0
        ),
        (0x7f, 0xfe, 0x55)
    );
    let mut output = Encoder::new(4);
    settings.write(&mut output).unwrap();
    assert_eq!(output.finish(), bytes);
}
