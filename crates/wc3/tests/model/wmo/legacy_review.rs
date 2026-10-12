//! Historical regressions from independent framing and embedded-address review.
use std::borrow::Cow;
use wc3::model::wmo::{self, Read as _};
use wmo::codec::Decoder;
use wmo::group::GroupChunk;
use wmo::legacy::{Group14, Group16, LegacyGroupChunk, Root14, Root16, RootChunk14};
use wmo::root::RootChunk;

fn frame(mut tag: [u8; 4], bytes: &[u8]) -> Vec<u8> {
    tag.reverse();
    let mut out = tag.to_vec();
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
    out
}
fn alpha_reversed_rows() -> Vec<u8> {
    // MVER=12 bytes, MOMO=(MOHD72 + MOGI88)+8 =168 bytes.
    // Each following MOGP frame occupies 136 bytes.
    let mut header = vec![0; 64];
    header[4..8].copy_from_slice(&2u32.to_le_bytes());
    let mut info = vec![0; 80];
    for (row, offset) in [316u32, 180].into_iter().enumerate() {
        let start = row * 40;
        info[start..start + 4].copy_from_slice(&offset.to_le_bytes());
        info[start + 4..start + 8].copy_from_slice(&136u32.to_le_bytes());
        info[start + 36..start + 40].copy_from_slice(&(-1i32).to_le_bytes());
    }
    let mut momo = frame(*b"MOHD", &header);
    momo.extend(frame(*b"MOGI", &info));
    let mut out = frame(*b"MVER", &14u32.to_le_bytes());
    out.extend(frame(*b"MOMO", &momo));
    for id in [111i32, 222] {
        let mut group = vec![0; 128];
        group[0..8].fill(0xff);
        group[116..120].copy_from_slice(&id.to_le_bytes());
        out.extend(frame(*b"MOGP", &group));
    }
    out
}

#[test]
fn alpha_logical_mogi_order_is_preserved_when_physical_groups_differ() {
    let bytes = alpha_reversed_rows();
    let root = Root14::decode(&bytes).unwrap();
    assert_eq!(root.group(0).unwrap().header.unique_id, 222);
    assert_eq!(root.group(1).unwrap().header.unique_id, 111);
    let encoded = root.encode().unwrap();
    assert_eq!(encoded, bytes);
    let decoded = Root14::decode(&encoded).unwrap();
    assert_eq!(decoded.group(0).unwrap().header.unique_id, 222);
}

#[test]
fn alpha_offsets_relocate_without_changing_logical_group_identity() {
    let bytes = alpha_reversed_rows();
    let mut root = Root14::decode(&bytes).unwrap();
    root.push_chunk(RootChunk14::Common(RootChunk::Unknown {
        tag: *b"XTRA",
        data: Cow::Borrowed(&[1, 2, 3]),
    }))
    .unwrap();
    let encoded = root.encode().unwrap();
    let decoded = Root14::decode(&encoded).unwrap();
    assert_eq!(decoded.group(0).unwrap().header.unique_id, 222);
    assert_eq!(decoded.group(1).unwrap().header.unique_id, 111);
}

#[test]
fn legacy_header_only_documents_preserve_intervening_version_position() {
    for (version, width) in [(14u32, 128usize), (16, 68)] {
        let mut bytes = frame(*b"MOGP", &vec![0; width]);
        bytes.extend(frame(*b"XAAA", &[1]));
        bytes.extend(frame(*b"MVER", &version.to_le_bytes()));
        bytes.extend(frame(*b"XBBB", &[2]));
        let encoded = if version == 14 {
            Group14::decode(&bytes).unwrap().encode().unwrap()
        } else {
            Group16::decode(&bytes).unwrap().encode().unwrap()
        };
        assert_eq!(encoded, bytes);
    }
}

#[test]
fn legacy_group_read_errors_include_enclosing_cursor_origin() {
    let mut input = Decoder::new(&[0; 3]).with_origin(500);
    assert_eq!(Group14::read(&mut input).unwrap_err().offset, 500);
    let mut input = Decoder::new(&[0; 3]).with_origin(700);
    assert_eq!(Group16::read(&mut input).unwrap_err().offset, 700);
}

#[test]
fn mutable_legacy_root_payloads_cannot_inject_protected_framing() {
    for tag in [*b"MVER", *b"MOGP", *b"MOMO", *b"MOHD"] {
        let mut alpha = Root14::new();
        alpha
            .push_chunk(RootChunk14::Common(RootChunk::Unknown {
                tag: *b"XAAA",
                data: Cow::Borrowed(&[]),
            }))
            .unwrap();
        *alpha.chunks_mut().next().unwrap() = RootChunk14::Common(RootChunk::Unknown {
            tag,
            data: Cow::Borrowed(&[]),
        });
        assert!(alpha.encode().is_err());
        let mut beta = Root16::new();
        beta.push_chunk(RootChunk::Unknown {
            tag: *b"XAAA",
            data: Cow::Borrowed(&[]),
        })
        .unwrap();
        *beta.chunks_mut().next().unwrap() = RootChunk::Unknown {
            tag,
            data: Cow::Borrowed(&[]),
        };
        assert!(beta.encode().is_err());
    }
}

#[test]
fn legacy_group_common_wrapper_rejects_overridden_modern_payload_types() {
    let mut group = Group16::new();
    group
        .chunks
        .push(LegacyGroupChunk::Common(GroupChunk::Triangles(
            wmo::codec::Array::from_records(&[wmo::records::Triangle::default()]).unwrap(),
        )));
    assert!(group.encode().is_err());
}

#[test]
fn beta_payload_factory_establishes_required_standalone_version() {
    let group = Group16::decode_payload(&[0; 68]).unwrap();
    let bytes = group.encode().unwrap();
    let decoded = Group16::decode(&bytes).unwrap();
    assert_eq!(decoded.header, group.header);
    assert_eq!(decoded.entries().len(), 2);
}

#[test]
fn alpha_common_wrapper_rejects_modern_material_record_width() {
    let mut root = Root14::new();
    let material = wmo::records::Material::<wmo::Classic>::default();
    root.push_chunk(RootChunk14::Common(RootChunk::Materials(
        wmo::codec::Array::from_records(&[material]).unwrap(),
    )))
    .unwrap();
    assert!(root.encode().is_err());
}
