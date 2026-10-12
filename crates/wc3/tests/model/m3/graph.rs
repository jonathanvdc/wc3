//! Reference graph checks and independent opaque-section wire fixtures.
use m3::records::{
    Character, ModelRecord, StandardMaterial, StringReference, TextureLayer, V15, V20, V21, V23,
    V24,
};
use wc3::model::m3::{self, Document, Md34, Model, Ref};

#[test]
fn typed_resolution_checks_family_revision_and_count() {
    let mut document = Document::<Md34>::new();
    let characters = document.append(&[Character { value: 0 }]).unwrap();
    let layers = document.append(&[TextureLayer::<V20>::default()]).unwrap();
    assert!(document
        .resolve(Ref::<TextureLayer<V20>>::new(1, characters.index(), 0))
        .is_err());
    assert!(document
        .resolve(Ref::<TextureLayer<V21>>::new(1, layers.index(), 0))
        .is_err());
    assert!(document
        .resolve(Ref::<TextureLayer<V20>>::new(2, layers.index(), 0))
        .is_err());
    assert!(document
        .resolve(Ref::<TextureLayer<V21>>::new(0, u32::MAX, 99))
        .unwrap()
        .is_empty());
}

#[test]
fn dangling_and_wrong_family_references_in_native_records_fail_validation() {
    for index in [999, 1] {
        let mut document = Document::<Md34>::new();
        document.append(&[TextureLayer::<V20>::default()]).unwrap();
        document
            .append(&[StringReference {
                value: Ref::new(1, index, 0),
            }])
            .unwrap();
        assert!(document.validate().is_err());
        assert!(document.write().is_err());
    }
}

#[test]
fn referenced_arrays_cannot_be_cleared_until_the_reference_is_removed() {
    let mut document = Document::<Md34>::new();
    let target = document.append(&[Character { value: 0 }]).unwrap();
    let incoming = document
        .append(&[StringReference {
            value: Ref::new(1, target.index(), 0),
        }])
        .unwrap();
    assert!(document.clear(target).is_err());
    document
        .replace(incoming, &[StringReference::default()])
        .unwrap();
    document.clear(target).unwrap();
    assert_eq!(document.section(target).unwrap().count(), 0);
    document.validate().unwrap();
}

fn opaque_fixture(tag: [u8; 4], revision: u32) -> Vec<u8> {
    let mut bytes = vec![0xaa; 96];
    bytes[..4].copy_from_slice(b"43DM");
    bytes[4..8].copy_from_slice(&64u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
    bytes[12..24].fill(0);
    bytes[32..64].fill(0x5b);
    for (slot, tag, offset, count, revision) in [
        (0, *b"MD34", 0u32, 1u32, 11u32),
        (1, tag, 32, 123, revision),
    ] {
        let start = 64 + slot * 16;
        let mut wire = tag;
        wire.reverse();
        bytes[start..start + 4].copy_from_slice(&wire);
        bytes[start + 4..start + 8].copy_from_slice(&offset.to_le_bytes());
        bytes[start + 8..start + 12].copy_from_slice(&count.to_le_bytes());
        bytes[start + 12..start + 16].copy_from_slice(&revision.to_le_bytes());
    }
    bytes
}

#[test]
fn unknown_tags_and_unknown_revisions_keep_original_placement() {
    for tag in [*b"????", *b"LAYR"] {
        let source = opaque_fixture(tag, 999);
        let mut document = Document::<Md34>::read(&source).unwrap();
        assert_eq!(document.write().unwrap(), source);
        let opaque = document.sections().nth(1).unwrap().0;
        assert!(document.section(opaque).unwrap().is_borrowed());
        assert!(document.clear(opaque).is_err());
        document.replace_bytes(opaque, vec![7; 32]).unwrap();
        let edited = document.write().unwrap();
        assert_eq!(&edited[..32], &source[..32]);
        assert_eq!(&edited[64..], &source[64..]);
        assert_eq!(&edited[32..64], &[7; 32]);
        document.replace_bytes(opaque, vec![7; 31]).unwrap();
        assert!(document.write().is_err());
    }
}

#[test]
fn root_and_material_revisions_remain_independent_after_editing() {
    let mut document = Document::<Md34>::new();
    let material = document
        .append(&[StandardMaterial::<V15>::default()])
        .unwrap();
    let root = document
        .append(&[ModelRecord::<V23> {
            standard_materials: Ref::new(1, material.index(), 0),
            ..Default::default()
        }])
        .unwrap();
    document.set_root(Ref::<ModelRecord<V23>>::new(1, root.index(), 17));
    let model = Model::from_document(document).unwrap();
    assert!(model.root::<ModelRecord<V24>>().is_err());
    let bytes = model.write().unwrap();
    let parsed = Model::<Md34>::read(&bytes).unwrap();
    let root = parsed.root::<ModelRecord<V23>>().unwrap();
    assert_eq!(
        parsed
            .document()
            .resolve::<StandardMaterial<V15>>(root.standard_materials.cast())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(parsed.document().root_reference().flags, 17);
    assert_eq!(
        parsed
            .document()
            .referenced_section(root.standard_materials)
            .unwrap()
            .unwrap()
            .revision(),
        15
    );
}

#[test]
fn malformed_index_mutations_return_errors_without_panicking() {
    let source = opaque_fixture(*b"LAYR", 999);
    for (offset, values) in [
        (4, vec![0, 23, 95, u32::MAX]),
        (8, vec![3, u32::MAX]),
        (68, vec![1, 24, 96]),
        (72, vec![0, 2]),
        (76, vec![0, 12]),
        (84, vec![0, 1, 23, 64, 65, 97, u32::MAX]),
    ] {
        for value in values {
            let mut bytes = source.clone();
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            assert!(
                Document::<Md34>::read(&bytes).is_err(),
                "offset {offset}, value {value}"
            );
        }
    }
}
