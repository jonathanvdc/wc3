//! Independently assembled native WEM framing, without a format writer fixture.
use wc3::model::wem;
fn word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

/// One WDOC record with a declared Generic profile and no linked sections.
/// Header and index fields deliberately use literal offsets rather than codec helpers.
fn fixture() -> Vec<u8> {
    let mut bytes = vec![0; 176];
    bytes[..4].copy_from_slice(b"WOEM");
    word(&mut bytes, 4, 3);
    word(&mut bytes, 8, 144);
    word(&mut bytes, 12, 2);
    word(&mut bytes, 16, 1);
    word(&mut bytes, 20, 1);
    word(&mut bytes, 28, 1);
    word(&mut bytes, 32, 1); // Inline profile count.
    word(&mut bytes, 36, 0); // Generic profile.
    word(&mut bytes, 40, 0); // Default Generic profile.
    bytes[44] = 0; // CoordSpace has a one-byte representation.
    word(&mut bytes, 45, 1.0f32.to_bits());
    // Name reference, bounds and four collection references remain empty.
    bytes[144..148].copy_from_slice(b"WOEM");
    word(&mut bytes, 152, 1);
    word(&mut bytes, 156, 0);
    bytes[160..164].copy_from_slice(b"WDOC");
    word(&mut bytes, 164, 32);
    word(&mut bytes, 168, 1);
    word(&mut bytes, 172, 1);
    bytes
}

#[test]
fn independent_v3_header_and_unaligned_document_root_are_readable() {
    use wc3::model::wem::records::{DocumentRecord, V1};
    use wc3::model::wem::{Document, V3};
    let bytes = fixture();
    let document = Document::<V3>::read(&bytes).unwrap();
    let root = document.root::<DocumentRecord<V1>>().unwrap();
    assert_eq!(root.default_profile.raw(), 0);
    assert_eq!(root.space.raw(), 0);
    assert_eq!(root.unit_scale, 1.0);
    assert_eq!(root.bounds.minimum, [0.0; 3]);
    assert_eq!(root.models.entries, 0);
    let output = document.write().unwrap();
    assert_eq!(&output[..4], b"WOEM");
    assert_eq!(u32::from_le_bytes(output[4..8].try_into().unwrap()), 3);
    let reread = Document::<V3>::read(&output).unwrap();
    assert_eq!(reread.root::<DocumentRecord<V1>>().unwrap().unit_scale, 1.0);
}

#[test]
fn malformed_container_locations_and_root_reference_are_rejected() {
    use wc3::model::wem::{Document, V3};
    for (offset, value) in [
        (8, 175),
        (12, u32::MAX),
        (16, 2),
        (20, 999),
        (164, 177),
        (168, u32::MAX),
    ] {
        let mut bytes = fixture();
        word(&mut bytes, offset, value);
        assert!(
            Document::<V3>::read(&bytes).is_err(),
            "offset {offset}, value {value}"
        );
    }
    let mut bytes = fixture();
    bytes[..4].copy_from_slice(b"MEOW");
    assert!(Document::<V3>::read(&bytes).is_err());
    for end in [0, 3, 31, 143, 175] {
        assert!(Document::<V3>::read(&fixture()[..end]).is_err());
    }
}

#[test]
fn known_root_revision_is_checked_independently_of_file_version() {
    use wc3::model::wem::{Document, RawDocument, V3};
    let mut bytes = fixture();
    word(&mut bytes, 172, 999);
    assert!(RawDocument::read(&bytes).is_ok());
    assert!(Document::<V3>::read(&bytes).is_err());
}

#[test]
fn foreign_section_identity_cannot_select_a_same_numbered_section() {
    use wc3::model::wem::RawDocument;
    let bytes = fixture();
    let first = RawDocument::read(&bytes).unwrap();
    let second = RawDocument::read(&bytes).unwrap();
    let id = first.section_id(1).unwrap();
    assert!(second.section(id).is_none());
    assert!(second.section(second.section_id(1).unwrap()).is_some());
}

fn named_fixture() -> Vec<u8> {
    let base = fixture();
    let mut bytes = vec![0; 208];
    bytes[..144].copy_from_slice(&base[..144]);
    bytes[144..148].copy_from_slice(b"name");
    word(&mut bytes, 8, 160);
    word(&mut bytes, 12, 3);
    word(&mut bytes, 49, 4);
    word(&mut bytes, 53, 2);
    bytes[160..192].copy_from_slice(&base[144..176]);
    bytes[192..196].copy_from_slice(b"CHAR");
    word(&mut bytes, 196, 144);
    word(&mut bytes, 200, 4);
    bytes
}

#[test]
fn string_sections_are_borrowed_and_counted_without_requiring_a_terminator() {
    use wc3::model::wem::records::{DocumentRecord, V1};
    use wc3::model::wem::{Document, V3};
    let bytes = named_fixture();
    let document = Document::<V3>::read(&bytes).unwrap();
    let root = document.root::<DocumentRecord<V1>>().unwrap();
    // The format's CHAR reference length defines the string, rather than a trailing NUL.
    let name = document.string(root.name).unwrap().unwrap();
    assert_eq!(name, "name");
    assert_eq!(name.as_ptr(), bytes[144..].as_ptr());
}

#[test]
fn named_section_reference_cannot_overrun_its_declared_count() {
    use wc3::model::wem::{Document, V3};
    let mut bytes = named_fixture();
    word(&mut bytes, 49, 5);
    assert!(Document::<V3>::read(&bytes).is_err());
}

#[test]
fn unknown_payload_and_index_hole_keep_their_original_slots_and_bytes() {
    use wc3::model::wem::RawDocument;
    let bytes = opaque_fixture();
    let document = RawDocument::read(&bytes).unwrap();
    let section = document.section(document.section_id(3).unwrap()).unwrap();
    assert_eq!(section.tag(), *b"ZZZZ");
    assert_eq!(section.revision(), 27);
    assert_eq!(section.count(), 5);
    assert_eq!(section.bytes(), &bytes[144..176]);
    let output = document.write().unwrap();
    let reread = RawDocument::read(&output).unwrap();
    let opaque = reread.section(reread.section_id(3).unwrap()).unwrap();
    assert_eq!(opaque.bytes(), &bytes[144..176]);
    assert_eq!(
        reread.section(reread.section_id(2).unwrap()).unwrap().tag(),
        *b"HOLE"
    );
}

#[test]
fn checked_document_requires_one_root_record_even_when_a_prefix_reference_is_valid() {
    use wc3::model::wem::{Document, RawDocument, V3};
    let base = fixture();
    let mut bytes = vec![0; 288];
    bytes[..32].copy_from_slice(&base[..32]);
    bytes[32..137].copy_from_slice(&base[32..137]);
    bytes[137..242].copy_from_slice(&base[32..137]);
    word(&mut bytes, 8, 256);
    bytes[256..288].copy_from_slice(&base[144..176]);
    word(&mut bytes, 280, 2);
    assert!(RawDocument::read(&bytes).is_ok());
    assert!(Document::<V3>::read(&bytes).is_err());
}

#[test]
fn shared_string_links_keep_identity_when_the_target_section_is_resized() {
    use wc3::model::wem::records::{AssetKey, Character, DocumentRecord, V1};
    use wc3::model::wem::{Document, Ref, V3};
    let bytes = named_fixture();
    let document = Document::<V3>::read(&bytes).unwrap();
    let mut root = document.root::<DocumentRecord<V1>>().unwrap();
    let root_id = document.root_id();
    let mut raw = document.into_raw();
    let name_id = raw.section_id(root.name.index).unwrap();
    let key_id = raw
        .append(&[AssetKey {
            group: 0,
            id: 42,
            path: root.name,
        }])
        .unwrap();
    let linked = raw
        .array::<AssetKey>(key_id)
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(
        raw.string(linked.path).unwrap().unwrap().as_ptr(),
        raw.string(root.name).unwrap().unwrap().as_ptr()
    );
    let replacement: Vec<_> = b"longer name"
        .iter()
        .map(|value| Character { value: *value })
        .collect();
    raw.replace(name_id, &replacement).unwrap();
    root.name = Ref::new(replacement.len() as u32, name_id.index(), root.name.flags);
    raw.replace(root_id, &[root]).unwrap();
    // The other reference deliberately keeps a prefix count while sharing the same target.
    let output = Document::<V3>::from_raw(raw).unwrap().write().unwrap();
    let decoded = Document::<V3>::read(&output).unwrap();
    let root = decoded.root::<DocumentRecord<V1>>().unwrap();
    let key = decoded
        .raw()
        .array::<AssetKey>(decoded.raw().section_id(key_id.index()).unwrap())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(root.name.index, key.path.index);
    assert_eq!(decoded.string(root.name).unwrap(), Some("longer name"));
    assert_eq!(decoded.string(key.path).unwrap(), Some("long"));
}

#[test]
fn authored_empty_document_and_owned_io_roundtrip() {
    use wc3::model::wem::{self, Document, V3};
    let document = Document::<V3>::new().unwrap();
    let bytes = document.write().unwrap();
    let owned = wem::from_reader::<V3>(bytes.as_slice()).unwrap();
    let mut output = Vec::new();
    wem::to_writer(&mut output, &owned).unwrap();
    Document::<V3>::read(&output).unwrap();
}

#[test]
fn unknown_global_generation_remains_opaque_and_preserves_native_bytes() {
    use wc3::model::wem::{Document, RawDocument, V3};
    let mut bytes = fixture();
    word(&mut bytes, 4, 999);
    let mut raw = RawDocument::read(&bytes).unwrap();
    assert_eq!(raw.write().unwrap(), bytes);
    assert!(Document::<V3>::read(&bytes).is_err());
    raw.set_profile_mask(0x80000000u32.into());
    let output = raw.write().unwrap();
    word(&mut bytes, 28, 0x80000000);
    assert_eq!(output, bytes);
}

#[test]
fn deleting_referenced_sections_or_using_foreign_edit_ids_is_rejected() {
    use wc3::model::wem::RawDocument;
    let bytes = named_fixture();
    let mut raw = RawDocument::read(&bytes).unwrap();
    let other = RawDocument::read(&bytes).unwrap();
    assert!(raw.clear(other.section_id(2).unwrap()).is_err());
    assert!(raw.clear(raw.section_id(2).unwrap()).is_err());
    raw.validate().unwrap();
    assert!(raw.write().is_ok());
}

#[test]
fn known_reference_families_reject_incorrect_target_tags() {
    use wc3::model::wem::{Document, V3};
    let mut bytes = named_fixture();
    bytes[192..196].copy_from_slice(b"U8__");
    assert!(Document::<V3>::read(&bytes).is_err());
}

fn opaque_fixture() -> Vec<u8> {
    let base = fixture();
    let mut bytes = vec![0; 240];
    bytes[..144].copy_from_slice(&base[..144]);
    word(&mut bytes, 8, 176);
    word(&mut bytes, 12, 4);
    for (index, byte) in bytes[144..176].iter_mut().enumerate() {
        *byte = 0x80 + index as u8;
    }
    bytes[176..208].copy_from_slice(&base[144..176]);
    bytes[208..212].copy_from_slice(b"HOLE");
    bytes[224..228].copy_from_slice(b"ZZZZ");
    word(&mut bytes, 228, 144);
    word(&mut bytes, 232, 5);
    word(&mut bytes, 236, 27);
    bytes
}

#[test]
fn opaque_address_spaces_reject_resizing_and_appending_index_slots() {
    use wc3::model::wem::records::Character;
    use wc3::model::wem::RawDocument;
    let bytes = opaque_fixture();
    let mut resized = RawDocument::read(&bytes).unwrap();
    resized
        .replace_bytes(resized.section_id(3).unwrap(), vec![0; 33])
        .unwrap();
    assert!(resized.write().is_err());
    let mut appended = RawDocument::read(&bytes).unwrap();
    appended.append(&[Character { value: b'x' }]).unwrap();
    assert!(appended.write().is_err());
}

fn material_fixture(native_kind: u8, payload_size: usize) -> Vec<u8> {
    let base = fixture();
    let table = 144 + payload_size;
    let mut bytes = vec![0; table + 48];
    bytes[..144].copy_from_slice(&base[..144]);
    word(&mut bytes, 8, table as u32);
    word(&mut bytes, 12, 3);
    bytes[144 + 17] = 1;
    bytes[144 + 18] = 1;
    bytes[144 + 61] = native_kind;
    bytes[table..table + 32].copy_from_slice(&base[144..176]);
    bytes[table + 32..table + 36].copy_from_slice(b"MATL");
    word(&mut bytes, table + 36, 144);
    word(&mut bytes, table + 40, 1);
    word(&mut bytes, table + 44, 1);
    bytes
}

#[test]
fn unknown_variant_preserves_the_bounded_section_without_guessing_its_inline_width() {
    use wc3::model::wem::RawDocument;
    let bytes = material_fixture(99, 80);
    let mut raw = RawDocument::read(&bytes).unwrap();
    let section = raw.section(raw.section_id(2).unwrap()).unwrap();
    assert_eq!(section.bytes(), &bytes[144..224]);
    assert_eq!(raw.write().unwrap(), bytes);
    raw.set_profile_mask(0u32.into());
    let mut expected = bytes.clone();
    word(&mut expected, 28, 0);
    assert_eq!(raw.write().unwrap(), expected);
    let malformed = material_fixture(2, 62); // Known M2 branch lacks its required twelve-byte link.
    let error = RawDocument::read(&malformed).unwrap_err();
    assert_eq!(error.offset, 206);
}

#[test]
fn zero_count_unknown_sections_still_retain_their_uninterpreted_payload_extent() {
    use wc3::model::wem::RawDocument;
    let mut bytes = opaque_fixture();
    word(&mut bytes, 232, 0);
    let raw = RawDocument::read(&bytes).unwrap();
    assert_eq!(
        raw.section(raw.section_id(3).unwrap()).unwrap().bytes(),
        &bytes[144..176]
    );
}

struct HeaderReplacement;
impl wem::Record for HeaderReplacement {
    const TAG: [u8; 4] = *b"WOEM";
    const VERSION: u32 = 0;
}
impl wem::Write for HeaderReplacement {
    fn write(&self, output: &mut wem::Encoder) -> Result<(), wem::WriteError> {
        output.bytes(&[0; 32])
    }
}

#[test]
fn custom_typed_records_cannot_replace_container_header_metadata() {
    use wc3::model::wem::RawDocument;
    let bytes = fixture();
    let mut raw = RawDocument::read(&bytes).unwrap();
    assert!(raw
        .replace(raw.section_id(0).unwrap(), &[HeaderReplacement])
        .is_err());
    assert_eq!(raw.write().unwrap(), bytes);
}

#[test]
fn empty_indexed_arrays_still_require_the_selected_family_and_revision() {
    use wc3::model::wem::records::{Character, NativeEntry, U32, V1, V2};
    use wc3::model::wem::RawDocument;
    let mut raw = RawDocument::new();
    let scalar = raw.append(&[U32 { value: 42 }]).unwrap();
    raw.clear(scalar).unwrap();
    assert!(raw.array::<U32>(scalar).unwrap().is_empty());
    assert!(raw.array::<Character>(scalar).is_err());
    let native = raw.append(&[NativeEntry::<V1>::default()]).unwrap();
    raw.clear(native).unwrap();
    assert!(raw.array::<NativeEntry<V1>>(native).unwrap().is_empty());
    assert!(raw.array::<NativeEntry<V2>>(native).is_err());
}

#[test]
fn default_profile_must_belong_to_the_roots_declared_profile_set() {
    use wc3::model::wem::{Document, RawDocument, V3};
    let mut bytes = fixture();
    word(&mut bytes, 40, 1); // Default WC3 Classic; the declared list contains only Generic.
    assert!(RawDocument::read(&bytes).is_ok());
    assert!(Document::<V3>::read(&bytes).is_err());
}

#[test]
fn borrowed_views_share_payloads_and_owned_documents_keep_existing_ids() {
    use wc3::model::wem::records::{DocumentRecord, V1};
    use wc3::model::wem::{Borrowed as _, Document, V3};
    let owned = {
        let bytes = named_fixture();
        let document = Document::<V3>::read(&bytes).unwrap();
        let id = document.root_id();
        let view = document.as_borrowed();
        assert_eq!(
            view.raw().section(id).unwrap().bytes().as_ptr(),
            document.raw().section(id).unwrap().bytes().as_ptr()
        );
        let owned = view.into_owned();
        assert!(owned.raw().section(id).is_some());
        owned
    };
    let root = owned.root::<DocumentRecord<V1>>().unwrap();
    assert_eq!(owned.string(root.name).unwrap(), Some("name"));
    assert_eq!(
        Document::<V3>::read(&owned.write().unwrap())
            .unwrap()
            .string(root.name)
            .unwrap(),
        Some("name")
    );
}

#[test]
fn material_profile_sets_must_use_a_declared_profile() {
    use wc3::model::wem::records::{ProfileId, ProfileMaterialSet};
    use wc3::model::wem::{Document, V3};
    let document = Document::<V3>::new().unwrap();
    let mut raw = document.into_raw();
    raw.append(&[ProfileMaterialSet {
        profile: ProfileId::Wc3Classic,
        ..Default::default()
    }])
    .unwrap();
    assert!(Document::<V3>::from_raw(raw).is_err());
}

#[test]
fn explicitly_declared_unknown_profiles_remain_valid_open_codes() {
    use wc3::model::wem::records::{DocumentRecord, ProfileId, ProfileMaterialSet, V1};
    use wc3::model::wem::{Document, InlineList, V3};
    let document = Document::<V3>::new().unwrap();
    let root_id = document.root_id();
    let mut root = document.root::<DocumentRecord<V1>>().unwrap();
    root.profiles = InlineList(vec![ProfileId::from_raw(999)]);
    root.default_profile = ProfileId::from_raw(999);
    let mut raw = document.into_raw();
    raw.replace(root_id, &[root]).unwrap();
    raw.append(&[ProfileMaterialSet {
        profile: ProfileId::from_raw(999),
        ..Default::default()
    }])
    .unwrap();
    Document::<V3>::from_raw(raw).unwrap();
}

#[test]
fn unsupported_profile_set_revisions_are_not_given_current_layout_semantics() {
    use wc3::model::wem::{Document, V3};
    let document = Document::<V3>::new().unwrap();
    let mut raw = document.into_raw();
    // This word may resemble a known profile selector; revision 999 does not establish that meaning.
    raw.append_raw(*b"PROF", 999, 1, 1u32.to_le_bytes().to_vec())
        .unwrap();
    Document::<V3>::from_raw(raw).unwrap();
}

#[test]
fn empty_known_section_offsets_do_not_truncate_overlapping_live_payloads() {
    use wc3::model::wem::{Document, V3};
    let mut bytes = fixture();
    bytes.resize(192, 0);
    word(&mut bytes, 12, 3);
    bytes[176..180].copy_from_slice(b"U32_");
    word(&mut bytes, 180, 48); // A zero-width section has no occupied span.
    let document = Document::<V3>::read(&bytes).unwrap();
    assert_eq!(document.write().unwrap(), bytes);
    assert!(document
        .raw()
        .section(document.raw().section_id(2).unwrap())
        .unwrap()
        .bytes()
        .is_empty());
}
