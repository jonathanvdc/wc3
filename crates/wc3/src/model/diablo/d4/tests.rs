use super::records::{AnimationPermutation, AppearanceLook};
use super::*;

#[test]
fn exact_format_hashes_round_trip() {
    fn check<L: Layout>() {
        let document = Document::<L>::default();
        let files = document.write_files().unwrap();
        assert!(files.payload.is_empty());
        Document::<L>::read(&files.root).unwrap();
    }
    check::<ActorC23b892a>();
    check::<Animation2088d14a>();
    check::<Appearance297828cc>();
    check::<MaterialBf40b801>();
}
#[test]
fn appearance_relocates_nested_arrays() {
    let mut document = Appearance::default();
    document.root.sno_id = 42;
    document
        .root
        .appearance_looks
        .push(AppearanceLook::default());
    let files = document.write_files().unwrap();
    let read = Appearance::read(&files.root).unwrap();
    assert_eq!(read.root.sno_id, 42);
    assert_eq!(read.root.appearance_looks.len(), 1);
    let second = read.write_files().unwrap();
    assert_eq!(files, second);
}
#[test]
fn animation_external_payload_is_explicit() {
    let mut document = Animation::default();
    let mut permutation = AnimationPermutation::default();
    permutation.payload_data.push(Default::default());
    document.root.permutations.push(permutation);
    let files = document.write_files().unwrap();
    assert!(!files.payload.is_empty());
    assert_eq!(
        Animation::read(&files.root).unwrap_err(),
        ReadError::MissingPayload
    );
    let decoded = Animation::read_with_payload(&files.root, &files.payload).unwrap();
    assert_eq!(decoded.root.permutations.len(), 1);
    assert_eq!(decoded.write_files().unwrap(), files);
}
#[test]
fn unknown_hash_and_negative_array_are_rejected() {
    let mut file = Appearance::default().write().unwrap();
    file[4..8].copy_from_slice(&0x12345678u32.to_le_bytes());
    assert!(matches!(
        Appearance::read(&file),
        Err(ReadError::UnsupportedFormat(0x12345678))
    ));
    let mut file = Appearance::default().write().unwrap();
    file[16 + 168..16 + 172].copy_from_slice(&(-1i32).to_le_bytes());
    assert!(matches!(
        Appearance::read(&file),
        Err(ReadError::Invalid(_))
    ));
}

#[test]
fn hand_authored_actor_uses_native_offsets() {
    let mut bytes = vec![0u8; 16 + 768];
    bytes[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    bytes[4..8].copy_from_slice(&ActorC23b892a::FORMAT_HASH.to_le_bytes());
    bytes[16..20].copy_from_slice(&1234i32.to_le_bytes());
    bytes[16 + 12..16 + 16].copy_from_slice(&0xa5a5u32.to_le_bytes());
    bytes[16 + 20..16 + 24].copy_from_slice(&9876i32.to_le_bytes());
    bytes[16 + 284..16 + 288].copy_from_slice(&3.5f32.to_le_bytes());
    let actor = Actor::read(&bytes).unwrap();
    assert_eq!(actor.root.sno_id, 1234);
    assert_eq!(actor.root.flags, 0xa5a5);
    assert_eq!(actor.root.appearance, 9876);
    assert_eq!(actor.root.location_power_src[0], 3.5);
    let output = actor.write().unwrap();
    assert_eq!(&output[16 + 20..16 + 24], &9876i32.to_le_bytes());
}
#[test]
fn external_leaf_buffers_borrow_then_detach() {
    use crate::model::common::IntoZeroVec as _;
    let mut document = Animation::default();
    let mut permutation = AnimationPermutation::default();
    let data = records::AnimPayloadData {
        unk_a9ead38: [1.5f32, 2.5].into_zero_vec(),
        ..Default::default()
    };
    permutation.payload_data.push(data);
    document.root.permutations.push(permutation);
    let files = document.write_files().unwrap();
    let decoded = Animation::read_with_payload(&files.root, &files.payload).unwrap();
    let leaf = &decoded.root.permutations[0].payload_data[0].unk_a9ead38;
    let pointer = leaf.as_bytes().as_ptr() as usize;
    let start = files.payload.as_ptr() as usize;
    assert!(pointer >= start && pointer < start + files.payload.len());
    let owned = decoded.into_owned();
    drop(files);
    assert_eq!(
        owned.root.permutations[0].payload_data[0]
            .unk_a9ead38
            .get(1),
        Some(2.5)
    );
}
#[test]
fn cyclic_descriptors_are_bounded() {
    struct Cycle;
    impl RecordLayout for Cycle {
        const SIZE: usize = 16;
    }
    impl<'a> Read<'a> for Cycle {
        fn read(d: &Decoder<'a>, o: usize) -> Result<Self, ReadError> {
            let _: Vec<Cycle> = d.records(o, false)?;
            Ok(Self)
        }
    }
    let mut bytes = [0u8; 16];
    bytes[12..16].copy_from_slice(&16i32.to_le_bytes());
    let err = Decoder::new(&bytes, None)
        .records::<Cycle>(0, false)
        .err()
        .unwrap();
    assert_eq!(err, ReadError::Invalid("array nesting limit"));
}
#[test]
fn polymorphic_family_and_unknown_types_are_checked() {
    let mut document = Appearance::default();
    document
        .root
        .speed_tree_wind
        .push(records::Polymorphic::TexAnim(Box::default()));
    assert!(matches!(
        document.write_files(),
        Err(WriteError::Invalid("polymorphic base family"))
    ));
    let mut bytes = vec![0u8; 32];
    bytes[12..16].copy_from_slice(&16i32.to_le_bytes());
    bytes[16..20].copy_from_slice(&1i32.to_le_bytes());
    assert!(Decoder::new(&bytes, None)
        .polymorphic(0, false, 2780951310)
        .is_err());
}

#[test]
fn unresolved_polymorphic_prefix_tables_are_rejected() {
    let mut bytes = vec![0u8; 48];
    bytes[8..12].copy_from_slice(&24i32.to_le_bytes());
    bytes[12..16].copy_from_slice(&24i32.to_le_bytes());
    bytes[16..20].copy_from_slice(&1i32.to_le_bytes());
    bytes[24] = 1;
    assert_eq!(
        Decoder::new(&bytes, None)
            .polymorphic(0, false, 2780951310)
            .unwrap_err(),
        ReadError::Invalid("unsupported nonzero polymorphic prefix table")
    );
}

#[test]
fn empty_tag_maps_still_validate_their_descriptor_prefix_and_offset() {
    let mut bytes = [0u8; 16];
    assert!(EmptyTagMap::read(&Decoder::new(&bytes, None), 0).is_ok());
    bytes[0] = 1;
    assert!(EmptyTagMap::read(&Decoder::new(&bytes, None), 0).is_err());
    bytes[0] = 0;
    bytes[8..12].copy_from_slice(&(-1i32).to_le_bytes());
    assert!(EmptyTagMap::read(&Decoder::new(&bytes, None), 0).is_err());
}
