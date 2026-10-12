use wc3::model::common::FileKey;
use wc3::model::m3::records::{
    Character, DataDrivenMaterial, StringReference, TextureLayer, V0, V1, V20,
};
use wc3::model::m3::{Document, Md34, Ref, TextureOwner};

#[test]
fn only_native_texture_fields_declare_dependencies() {
    let mut document = Document::<Md34>::new();
    let name = document
        .append(&[Character { value: b'n' }, Character { value: 0 }])
        .unwrap();
    let path = document
        .append(&[Character { value: 0xff }, Character { value: 0 }])
        .unwrap();
    let layer = document
        .append(&[TextureLayer::<V20> {
            texture_path: Ref::new(2, path.index(), 9),
            ..Default::default()
        }])
        .unwrap();
    let strings = document
        .append(&[StringReference {
            value: Ref::new(2, path.index(), 0),
        }])
        .unwrap();
    let material = document
        .append(&[DataDrivenMaterial::<V1> {
            material_name: Ref::new(2, name.index(), 0),
            texture_paths: Ref::new(1, strings.index(), 0),
            ..Default::default()
        }])
        .unwrap();
    let dependencies = document.texture_dependencies().unwrap();
    assert_eq!(dependencies.len(), 2);
    assert_eq!(
        dependencies[0].owner,
        TextureOwner::Layer {
            section: layer,
            record: 0
        }
    );
    assert_eq!(
        dependencies[1].owner,
        TextureOwner::DataDriven {
            section: material,
            record: 0,
            slot: 0
        }
    );
    assert_eq!(dependencies[0].key(), FileKey::RawPath(vec![0xff]));
    assert_eq!(dependencies[0].reference.flags, 9);
    assert_eq!(
        dependencies[0].path.as_ptr(),
        document.section(path).unwrap().bytes().as_ptr()
    );
}

#[test]
fn malformed_texture_string_is_reported() {
    let mut document = Document::<Md34>::new();
    let path = document.append(&[Character { value: b'x' }]).unwrap();
    document
        .append(&[TextureLayer::<V20> {
            texture_path: Ref::new(1, path.index(), 0),
            ..Default::default()
        }])
        .unwrap();
    assert!(document.texture_dependencies().is_err());
}

#[test]
fn historically_defined_material_revisions_declare_texture_paths() {
    let mut document = Document::<Md34>::new();
    let path = document
        .append(&[Character { value: b'x' }, Character { value: 0 }])
        .unwrap();
    document
        .append(&[TextureLayer::<V0> {
            texture_path: Ref::new(2, path.index(), 0),
            ..Default::default()
        }])
        .unwrap();
    let strings = document
        .append(&[StringReference {
            value: Ref::new(2, path.index(), 0),
        }])
        .unwrap();
    document
        .append(&[DataDrivenMaterial::<V0> {
            texture_paths: Ref::new(1, strings.index(), 0),
            ..Default::default()
        }])
        .unwrap();
    assert_eq!(document.texture_dependencies().unwrap().len(), 2);
}
