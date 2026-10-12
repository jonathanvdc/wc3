use wc3::model::wmo::{
    self, Bundle, Classic, DependencyKind, FileKey, Group, GroupKey, Root, TextureReference,
};
use wmo::codec::Array;
use wmo::records::{GroupInfo, Material};
use wmo::root::RootChunk;
fn root() -> Root<'static, Classic> {
    let mut root = Root::new();
    root.header_mut().group_count = 1;
    root.insert_chunk(
        0,
        RootChunk::Groups(Array::from_records(&[GroupInfo::default()]).unwrap()),
    )
    .unwrap();
    root
}
#[test]
fn legacy_groups_resolve_without_loading_textures() {
    let root = root().encode().unwrap();
    let group = Group::<Classic>::new().encode().unwrap();
    let owner = FileKey::Path("World\\house.wmo".into());
    let mut requests = Vec::new();
    let bundle = Bundle::<Classic>::decode(&root, owner, |dependency| {
        requests.push(dependency.key.clone());
        Ok(Some(group.as_slice()))
    })
    .unwrap();
    assert_eq!(requests, [FileKey::Path("World\\house_000.wmo".into())]);
    bundle.validate_complete().unwrap();
    assert_eq!(
        bundle
            .group(GroupKey { index: 0, lod: 0 })
            .unwrap()
            .encode()
            .unwrap(),
        group
    );
    assert_eq!(bundle.encode().unwrap().groups.len(), 1);
}
#[test]
fn group_ids_cover_lod_rows_and_missing_files_are_reported() {
    let mut root = root();
    root.insert_chunk(
        1,
        RootChunk::GroupFileIds(Array::from_records(&[100u32, 200]).unwrap()),
    )
    .unwrap();
    let mut bundle = Bundle::from_root(root).with_key(FileKey::Root);
    let dependencies = bundle.dependencies().unwrap();
    assert!(matches!(
        dependencies[1].kind,
        DependencyKind::Group { index: 0, lod: 1 }
    ));
    assert!(bundle.validate_complete().is_err());
    bundle.insert_group(
        GroupKey { index: 0, lod: 0 },
        FileKey::FileId(100),
        Group::new(),
    );
    assert!(bundle.validate_complete().is_err());
    bundle.insert_group(
        GroupKey { index: 0, lod: 1 },
        FileKey::FileId(200),
        Group::new(),
    );
    bundle.validate_complete().unwrap();
}
#[test]
fn texture_paths_rebuild_offsets_without_changing_ids_or_color_words() {
    let mut root = Root::<Classic>::new();
    root.insert_chunk(
        0,
        RootChunk::TextureNames(Array::from_records(b"a.blp\0").unwrap()),
    )
    .unwrap();
    root.insert_chunk(
        1,
        RootChunk::Materials(Array::from_records(&[Material::default()]).unwrap()),
    )
    .unwrap();
    assert_eq!(
        root.texture_reference(0, 0).unwrap(),
        Some(TextureReference::Path(b"a.blp"))
    );
    root.rewrite_texture_paths(|_| b"longer.blp".to_vec())
        .unwrap();
    assert_eq!(
        root.texture_reference(0, 2).unwrap(),
        Some(TextureReference::Path(b"longer.blp"))
    );
    let bytes = root.encode().unwrap();
    let decoded = Root::<Classic>::decode(&bytes).unwrap();
    assert_eq!(
        decoded.texture_reference(0, 0).unwrap(),
        Some(TextureReference::Path(b"longer.blp"))
    );
    let mut ids = Root::<wmo::Legion>::new();
    let material = wmo::records::Material {
        texture0: 42,
        ..Default::default()
    };
    ids.insert_chunk(
        0,
        RootChunk::Materials(Array::from_records(&[material]).unwrap()),
    )
    .unwrap();
    assert_eq!(ids.dependencies(&FileKey::Root).unwrap().len(), 1);
}

#[test]
fn beta_bundle_keeps_native_group_version_and_checks_identities() {
    use wmo::legacy::{Bundle16, Group16, Root16};
    let mut root = Root16::new();
    root.header_mut().group_count = 1;
    root.push_chunk(RootChunk::Groups(
        Array::from_records(&[GroupInfo::default()]).unwrap(),
    ))
    .unwrap();
    let root = root.encode().unwrap();
    let group = Group16::new().encode().unwrap();
    let bundle = Bundle16::decode(&root, FileKey::Path("beta.wmo".into()), |_| {
        Ok(Some(group.as_slice()))
    })
    .unwrap();
    bundle.validate_complete().unwrap();
    assert_eq!(
        bundle.encode().unwrap().groups.values().next().unwrap(),
        &group
    );
    let mut bad = Bundle16::from_root(Root16::decode(&root).unwrap())
        .with_key(FileKey::Path("beta.wmo".into()));
    bad.insert_group(0, FileKey::Path("wrong.wmo".into()), Group16::new());
    assert!(bad.validate_complete().is_err());
    assert!(bad.encode().is_err());
}
