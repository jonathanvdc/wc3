//! Independent group payload widths and contextual liquid/tangent decoding.
use wc3::model::formats::wmo::{self, Classic, Legion, Shadowlands};
use wmo::codec::{Decoder, Encoder};
use wmo::group::{GroupChunk, GroupContext, LiquidKind, LiquidVertices};
use wmo::records::{Batch, GroupHeader};
use wmo::{Read as _, RecordLayout as _, Write as _};

#[test]
fn header_tail_and_batch_prefix_are_selected_statically() {
    assert_eq!(GroupHeader::<Classic>::size(), 68);
    assert_eq!(GroupHeader::<Shadowlands>::size(), 68);
    assert_eq!(Batch::<Classic>::size(), 24);
    assert_eq!(Batch::<Legion>::size(), 24);
    let mut bytes = [0u8; 68];
    bytes[64..68].copy_from_slice(&[0xfe, 0xff, 3, 0]);
    let legacy = GroupHeader::<Classic>::read(&mut Decoder::new(&bytes)).unwrap();
    let modern = GroupHeader::<Shadowlands>::read(&mut Decoder::new(&bytes)).unwrap();
    assert_eq!(legacy.tail, 0x0003fffe);
    assert_eq!(modern.tail.parent_or_first_child, -2);
    assert_eq!(modern.tail.next_child, 3);
    let mut output = Encoder::new();
    modern.write(&mut output).unwrap();
    assert_eq!(output.finish(), bytes);
    let mut batch = [0u8; 24];
    batch[0] = 2;
    batch[1] = 3;
    batch[10..12].copy_from_slice(&1234u16.to_le_bytes());
    batch[18..20].copy_from_slice(&7u16.to_le_bytes());
    batch[20..22].copy_from_slice(&9u16.to_le_bytes());
    batch[22] = 6;
    batch[23] = 4;
    let legacy = Batch::<Classic>::read(&mut Decoder::new(&batch)).unwrap();
    let modern = Batch::<Legion>::read(&mut Decoder::new(&batch)).unwrap();
    assert_eq!(legacy.material_index(), 4);
    assert_eq!(legacy.vertex_range(), (7, 9));
    assert_eq!(modern.material_index(), 1234);
    assert_eq!(modern.vertex_range(), (0x20007, 0x30009));
}
#[test]
fn liquids_borrow_unaligned_grids_and_require_explicit_union_context() {
    let mut payload = vec![0; 30];
    payload[0..4].copy_from_slice(&1i32.to_le_bytes());
    payload[4..8].copy_from_slice(&1i32.to_le_bytes());
    payload.extend_from_slice(&[1, 2, 3, 4]);
    payload.extend_from_slice(&5.5f32.to_le_bytes());
    let GroupChunk::<Classic>::Liquid(raw) = GroupChunk::decode(*b"MLIQ", &payload, 50).unwrap()
    else {
        panic!()
    };
    let LiquidVertices::Unresolved(vertices) = raw.vertices else {
        panic!()
    };
    assert_eq!(vertices.as_bytes().as_ptr(), payload[30..].as_ptr());
    assert_eq!(vertices.get(0).unwrap().unwrap().data, [1, 2, 3, 4]);
    let context = GroupContext {
        liquid_kind: Some(LiquidKind::Magma),
        ..Default::default()
    };
    let chunk =
        GroupChunk::<Classic>::decode_with_context(*b"MLIQ", &payload, 50, &context).unwrap();
    let GroupChunk::Liquid(liquid) = &chunk else {
        panic!()
    };
    let LiquidVertices::Magma(vertices) = &liquid.vertices else {
        panic!()
    };
    assert_eq!(
        vertices.get(0).unwrap().unwrap().texture_coordinates,
        [0x0201, 0x0403]
    );
    assert_eq!(chunk.encode().unwrap(), payload);
    payload[0..4].copy_from_slice(&(-1i32).to_le_bytes());
    assert!(GroupChunk::<Classic>::decode(*b"MLIQ", &payload, 50).is_err());
}
#[test]
fn tangent_prefix_uses_batch_context_and_preserves_unresolved_payloads() {
    let mut payload = vec![0xff, 0xff, 0, 0];
    for value in [1f32, 2.0, 3.0, 4.0] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    assert!(matches!(
        GroupChunk::<Legion>::decode(*b"MOTA", &payload, 10).unwrap(),
        GroupChunk::TangentsUnresolved(_)
    ));
    let context = GroupContext {
        batch_count: Some(2),
        ..Default::default()
    };
    let chunk =
        GroupChunk::<Legion>::decode_with_context(*b"MOTA", &payload, 10, &context).unwrap();
    let GroupChunk::Tangents(tangents) = &chunk else {
        panic!()
    };
    assert_eq!(tangents.first_tangent.get(0).unwrap().unwrap(), u16::MAX);
    assert_eq!(
        tangents.tangents.get(0).unwrap().unwrap(),
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(chunk.encode().unwrap(), payload);
    assert!(GroupChunk::<Legion>::decode_with_context(
        *b"MOTA",
        &payload,
        10,
        &GroupContext {
            batch_count: Some(100),
            ..Default::default()
        }
    )
    .is_err());
}
#[test]
fn known_fixed_streams_reject_partial_elements() {
    for (tag, width) in [
        (*b"MOPY", 2),
        (*b"MPY2", 4),
        (*b"MOVI", 2),
        (*b"MOVX", 4),
        (*b"MOVT", 12),
        (*b"MONR", 12),
        (*b"MOTV", 8),
        (*b"MOBA", 24),
        (*b"MOBS", 24),
        (*b"MOPB", 24),
        (*b"MORB", 8),
        (*b"MOBN", 16),
        (*b"MOLP", 44),
        (*b"MOLS", 56),
        (*b"MOP2", 96),
        (*b"MOS2", 108),
    ] {
        let bytes = vec![0; width];
        let chunk = GroupChunk::<Legion>::decode(tag, &bytes, 123).unwrap();
        assert_eq!(chunk.tag(), tag);
        assert_eq!(chunk.encode().unwrap(), bytes);
        assert!(GroupChunk::<Legion>::decode(tag, &bytes[..width - 1], 123).is_err());
    }
}
