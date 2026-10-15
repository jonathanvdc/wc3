//! Regression checks from independent assembly and framing review.
use wc3::model::formats::wmo::codec::Array;
use wc3::model::formats::wmo::records::GroupInfo;
use wc3::model::formats::wmo::root::RootChunk;
use wc3::model::formats::wmo::{Bundle, Classic, FileKey, Group, GroupKey, Legion, Root};

fn frame(mut tag: [u8; 4], payload: &[u8]) -> Vec<u8> {
    tag.reverse();
    let mut result = tag.to_vec();
    result.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    result.extend_from_slice(payload);
    result
}
fn modern_root() -> Root<'static, Legion> {
    let mut root = Root::new();
    root.header_mut().group_count = 1;
    let info = GroupInfo {
        name_offset: -1,
        ..GroupInfo::default()
    };
    root.insert_chunk(0, RootChunk::Groups(Array::from_records(&[info]).unwrap()))
        .unwrap();
    root.insert_chunk(
        1,
        RootChunk::GroupFileIds(Array::from_records(&[100u32]).unwrap()),
    )
    .unwrap();
    root
}

#[test]
fn bundle_cannot_emit_a_group_under_stale_identity_after_gfid_edit() {
    let mut bundle = Bundle::from_root(modern_root());
    bundle.insert_group(
        GroupKey { index: 0, lod: 0 },
        FileKey::FileId(100),
        Group::new(),
    );
    bundle.validate_complete().unwrap();
    for chunk in bundle.root.chunks_mut() {
        if let RootChunk::GroupFileIds(ids) = chunk {
            ids.set(0, &200).unwrap();
        }
    }
    assert!(bundle.validate_complete().is_err());
    assert!(bundle.encode().is_err());
}

#[test]
fn bundle_rejects_registered_lod_not_declared_by_root() {
    let mut bundle = Bundle::from_root(modern_root());
    bundle.insert_group(
        GroupKey { index: 0, lod: 0 },
        FileKey::FileId(100),
        Group::new(),
    );
    bundle.insert_group(
        GroupKey { index: 0, lod: 2 },
        FileKey::FileId(300),
        Group::new(),
    );
    assert!(bundle.validate_complete().is_err());
    assert!(bundle.encode().is_err());
}

#[test]
fn header_only_body_preserves_intervening_version_frame_position() {
    let mut bytes = frame(*b"MOGP", &[0; 68]);
    bytes.extend(frame(*b"XAAA", &[1]));
    bytes.extend(frame(*b"MVER", &17u32.to_le_bytes()));
    bytes.extend(frame(*b"XBBB", &[2]));
    let group = Group::<Classic>::decode(&bytes).unwrap();
    assert_eq!(group.encode().unwrap(), bytes);
}

#[test]
fn path_rewrite_preserves_absent_texture_references() {
    use wc3::model::formats::wmo::{records::Material, TextureReference};
    let mut root = Root::<Classic>::new();
    root.insert_chunk(
        0,
        RootChunk::TextureNames(Array::from_records(b"\0old.blp\0").unwrap()),
    )
    .unwrap();
    root.insert_chunk(
        1,
        RootChunk::Materials(
            Array::from_records(&[Material {
                texture0: 1,
                ..Material::default()
            }])
            .unwrap(),
        ),
    )
    .unwrap();
    root.rewrite_texture_paths(|path| [b"new/".as_slice(), path].concat())
        .unwrap();
    assert_eq!(
        root.texture_reference(0, 0).unwrap(),
        Some(TextureReference::Path(b"new/old.blp"))
    );
    assert_eq!(root.texture_reference(0, 1).unwrap(), None);
    assert_eq!(root.texture_reference(0, 2).unwrap(), None);
}

#[test]
fn inconsistent_gfid_stride_does_not_silently_change_group_mapping() {
    let mut root = Root::<Legion>::new();
    root.header_mut().group_count = 3;
    root.insert_chunk(
        0,
        RootChunk::Groups(
            Array::from_records(&[GroupInfo::default(), GroupInfo::default()]).unwrap(),
        ),
    )
    .unwrap();
    root.insert_chunk(
        1,
        RootChunk::GroupFileIds(Array::from_records(&[100u32, 101, 102, 200, 201, 202]).unwrap()),
    )
    .unwrap();
    assert!(root.dependencies(&FileKey::Root).is_err());
}
