use wc3::model::wem::records::{AssetKey, NativeEntry, V1, V2};
use wc3::model::wem::{Array, Encoder};

#[test]
fn asset_identity_is_two_words_followed_by_a_twelve_byte_path_reference() {
    let mut bytes = Vec::new();
    for word in [0x11223344u32, 0x55667788, 4, 9, 0xabcdef01] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    let array = Array::<AssetKey>::read(&bytes, 1, 1).unwrap();
    assert_eq!(array.as_bytes().as_ptr(), bytes.as_ptr());
    let key = array.get(0).unwrap().unwrap();
    assert_eq!(key.group, 0x11223344);
    assert_eq!(key.id, 0x55667788);
    assert_eq!(key.path.entries, 4);
    assert_eq!(key.path.index, 9);
    assert_eq!(key.path.flags, 0xabcdef01);
    let mut encoder = Encoder::new(1);
    encoder.write(&key).unwrap();
    assert_eq!(encoder.finish(), bytes);
}

#[test]
fn native_entry_revision_two_adds_a_text_link_after_the_signed_value() {
    let mut bytes = vec![0; 32];
    bytes[12..20].copy_from_slice(&(-123i64).to_le_bytes());
    bytes[20..24].copy_from_slice(&3u32.to_le_bytes());
    bytes[24..28].copy_from_slice(&7u32.to_le_bytes());
    bytes[28..32].copy_from_slice(&17u32.to_le_bytes());
    let first = Array::<NativeEntry<V1>>::read(&bytes[..20], 1, 1).unwrap();
    let second = Array::<NativeEntry<V2>>::read(&bytes, 1, 2).unwrap();
    assert_eq!(first.get(0).unwrap().unwrap().value, -123);
    let entry = second.get(0).unwrap().unwrap();
    assert_eq!(entry.value, -123);
    assert_eq!(entry.text.entries, 3);
    assert_eq!(entry.text.index, 7);
    assert_eq!(entry.text.flags, 17);
    assert!(Array::<NativeEntry<V2>>::read(&bytes[..20], 1, 2).is_err());
}

#[test]
fn material_native_kind_controls_presence_of_the_inline_body_reference() {
    use wc3::model::wem::records::MaterialRecord;
    // Name12 + common48 + synchronization1 + native-kind1.
    let mut without = vec![0; 62];
    without[17] = 1; // Depth test, immediately after u32 blend and u8 cull.
    without[18] = 1; // Depth write.
    without[60] = 0; // Native synchronization.
    without[61] = 0; // No native block: no reference follows.
    let base = Array::<MaterialRecord>::read(&without, 1, 1).unwrap();
    let material = base.get(0).unwrap().unwrap();
    let mut encoder = Encoder::new(1);
    encoder.write(&material).unwrap();
    assert_eq!(encoder.finish(), without);
    let mut with = without.clone();
    with[61] = 2; // M2 native material block.
    with.extend_from_slice(&1u32.to_le_bytes());
    with.extend_from_slice(&9u32.to_le_bytes());
    with.extend_from_slice(&23u32.to_le_bytes());
    let native = Array::<MaterialRecord>::read(&with, 1, 1).unwrap();
    let mut encoder = Encoder::new(1);
    encoder.write(&native.get(0).unwrap().unwrap()).unwrap();
    assert_eq!(encoder.finish(), with);
    assert!(Array::<MaterialRecord>::read(&with[..62], 1, 1).is_err());
}
