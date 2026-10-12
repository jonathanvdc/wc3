use std::borrow::Cow;
use wc3::model::wmo::{
    self, Classic, DynamicFile, File, Group, GroupFraming, Profile, RawDocument, Root,
};
use wmo::root::RootChunk;

fn frame(tag: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut tag = tag;
    tag.reverse();
    let mut bytes = tag.to_vec();
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}
fn version() -> Vec<u8> {
    frame(*b"MVER", &17u32.to_le_bytes())
}
#[test]
fn root_preserves_header_position_and_unknown_bytes() {
    let mut bytes = frame(*b"XNEW", &[9, 4, 2]);
    bytes.extend(frame(*b"MOHD", &[0; 64]));
    bytes.extend(version());
    let mut root = Root::<Classic>::decode(&bytes).unwrap();
    assert_eq!(root.encode().unwrap(), bytes);
    root.header_mut().wmo_id = 123;
    let owned = root.into_owned();
    drop(bytes);
    assert_eq!(owned.header().wmo_id, 123);
    assert_eq!(
        Root::<Classic>::decode(&owned.encode().unwrap())
            .unwrap()
            .header()
            .wmo_id,
        123
    );
}
#[test]
fn required_headers_are_checked_and_cannot_be_removed() {
    assert!(Root::<Classic>::decode(&version()).is_err());
    let mut bytes = version();
    bytes.extend(frame(*b"MOHD", &[0; 64]));
    bytes.extend(frame(*b"MOHD", &[0; 64]));
    assert!(Root::<Classic>::decode(&bytes).is_err());
    assert_eq!(RawDocument::decode(&bytes).unwrap().chunks.len(), 3);
    let mut root = Root::<Classic>::new();
    assert!(root
        .insert_chunk(
            0,
            RootChunk::Unknown {
                tag: *b"MOHD",
                data: Cow::Borrowed(&[])
            }
        )
        .is_err());
    root.insert_chunk(
        0,
        RootChunk::Unknown {
            tag: *b"TEST",
            data: Cow::Borrowed(&[1]),
        },
    )
    .unwrap();
    *root.chunks_mut().next().unwrap() = RootChunk::Unknown {
        tag: *b"MOHD",
        data: Cow::Borrowed(&[0; 64]),
    };
    assert!(root.encode().is_err());
}
#[test]
fn group_nested_and_header_only_framing_roundtrip() {
    let tail = frame(*b"XYZW", &[8, 7, 6]);
    for nested in [true, false] {
        let mut bytes = version();
        let mut payload = vec![0; 68];
        if nested {
            payload.extend(&tail);
        }
        bytes.extend(frame(*b"MOGP", &payload));
        if !nested {
            bytes.extend(&tail);
        }
        let group = Group::<Classic>::decode(&bytes).unwrap();
        assert_eq!(group.encode().unwrap(), bytes);
        assert_eq!(
            group.framing(),
            if nested {
                GroupFraming::Nested
            } else {
                GroupFraming::HeaderOnly
            }
        );
        assert_eq!(group.chunks().len(), 1);
    }
}
#[test]
fn profile_is_explicit_and_not_guessed_from_mver() {
    let bytes = Root::<Classic>::new().encode().unwrap();
    assert!(DynamicFile::candidates(&bytes).len() > 1);
    assert!(DynamicFile::decode_detected(&bytes).is_err());
    let file = DynamicFile::decode(&bytes, Profile::Classic).unwrap();
    assert!(file.as_typed::<Classic>().is_some());
    assert!(file.as_typed::<wmo::Legion>().is_none());
    assert!(matches!(
        File::<Classic>::decode(&bytes).unwrap(),
        File::Root(_)
    ));
    let mut both = bytes;
    both.extend(frame(*b"MOGP", &[0; 68]));
    assert!(File::<Classic>::decode(&both).is_err());
}
#[test]
fn malformed_framing_and_nested_errors_have_absolute_positions() {
    let mut bytes = version();
    bytes.extend(frame(*b"MOHD", &[0; 63]));
    let error = Root::<Classic>::decode(&bytes).unwrap_err();
    assert_eq!(error.offset, 20);
    let mut payload = vec![0; 68];
    payload.extend([0u8; 3]);
    let mut group = version();
    group.extend(frame(*b"MOGP", &payload));
    assert_eq!(Group::<Classic>::decode(&group).unwrap_err().offset, 88);
    let mut wrong = frame(*b"MVER", &18u32.to_le_bytes());
    wrong.extend(frame(*b"MOHD", &[0; 64]));
    assert!(Root::<Classic>::decode(&wrong).is_err());
}

#[test]
fn every_truncated_prefix_is_rejected_without_panicking() {
    let root = Root::<Classic>::new().encode().unwrap();
    for end in 0..root.len() {
        assert!(Root::<Classic>::decode(&root[..end]).is_err());
    }
    let group = Group::<Classic>::new().encode().unwrap();
    for end in 0..group.len() {
        assert!(Group::<Classic>::decode(&group[..end]).is_err());
    }
    let mut payload = vec![0; 68];
    payload.extend(version());
    let mut illegal = version();
    illegal.extend(frame(*b"MOGP", &payload));
    assert!(Group::<Classic>::decode(&illegal).is_err());
}
