use wc3::model::common::FileKey;
use wc3::model::formats::m3;
use wc3::model::formats::m3::records::{
    Character, DataDrivenMaterial, StringReference, TextureLayer, V0, V1, V20,
};
use wc3::model::formats::m3::{Container, Md34, Ref, TextureOwner};

#[test]
fn only_native_texture_fields_declare_dependencies() {
    let mut document = Container::<Md34>::new();
    let name = document
        .append(&[Character { value: b'n' }, Character { value: 0 }])
        .unwrap();
    let path = document
        .append(&[Character { value: 0xff }, Character { value: 0 }])
        .unwrap();
    let layer = document
        .append(&[TextureLayer::<V20> {
            texture_path: Ref::new(2, path.index(), m3::records::ReferenceFlags(9)),
            ..Default::default()
        }])
        .unwrap();
    let strings = document
        .append(&[StringReference {
            value: Ref::from_section(2, path.index()),
        }])
        .unwrap();
    let material = document
        .append(&[DataDrivenMaterial::<V1> {
            material_name: Ref::from_section(2, name.index()),
            texture_paths: Ref::from_section(1, strings.index()),
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
    assert_eq!(dependencies[0].reference.flags.bits(), 9);
    assert_eq!(
        dependencies[0].path.as_ptr(),
        document.section(path).unwrap().bytes().as_ptr()
    );
}

#[test]
fn malformed_texture_string_is_reported() {
    let mut document = Container::<Md34>::new();
    let path = document.append(&[Character { value: b'x' }]).unwrap();
    document
        .append(&[TextureLayer::<V20> {
            texture_path: Ref::from_section(1, path.index()),
            ..Default::default()
        }])
        .unwrap();
    assert!(document.texture_dependencies().is_err());
}

#[test]
fn historically_defined_material_revisions_declare_texture_paths() {
    let mut document = Container::<Md34>::new();
    let path = document
        .append(&[Character { value: b'x' }, Character { value: 0 }])
        .unwrap();
    document
        .append(&[TextureLayer::<V0> {
            texture_path: Ref::from_section(2, path.index()),
            ..Default::default()
        }])
        .unwrap();
    let strings = document
        .append(&[StringReference {
            value: Ref::from_section(2, path.index()),
        }])
        .unwrap();
    document
        .append(&[DataDrivenMaterial::<V0> {
            texture_paths: Ref::from_section(1, strings.index()),
            ..Default::default()
        }])
        .unwrap();
    assert_eq!(document.texture_dependencies().unwrap().len(), 2);
}
