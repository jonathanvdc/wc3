//! Alpha and early-beta group records retain their historical widths and framing.
use wc3::model::formats::wmo::{self, Read as _, RecordLayout as _, Validate as _};
use wmo::codec::Decoder;
use wmo::legacy::{
    Batch14, Batch16, Group14, Group16, GroupChunk14, GroupHeader14, GroupHeader16,
    LegacyGroupChunk,
};

fn frame(mut tag: [u8; 4], bytes: &[u8]) -> Vec<u8> {
    tag.reverse();
    let mut output = tag.to_vec();
    output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    output.extend_from_slice(bytes);
    output
}
#[test]
fn alpha_header_and_lightmapped_batch_have_independent_native_offsets() {
    assert_eq!(GroupHeader14::size(), 128);
    assert_eq!(Batch14::size(), 24);
    let mut bytes = [0u8; 128];
    bytes[36..40].copy_from_slice(&(-2i32).to_le_bytes());
    bytes[44..48].copy_from_slice(&[1, 2, 3, 4]);
    bytes[52..54].copy_from_slice(&11u16.to_le_bytes());
    bytes[116..120].copy_from_slice(&(-7i32).to_le_bytes());
    bytes[120..128].copy_from_slice(b"RESERVED");
    let header = GroupHeader14::read(&mut Decoder::new(&bytes)).unwrap();
    assert_eq!(header.portal_start, -2);
    assert_eq!(header.fog_indices, [1, 2, 3, 4]);
    assert_eq!(header.interior_batches[0].vertex_start, 11);
    assert_eq!(header.unique_id, -7);
    assert_eq!(&header.reserved_tail, b"RESERVED");
    let mut batch = [0u8; 24];
    batch[0] = 5;
    batch[1] = 9;
    batch[14..16].copy_from_slice(&123u16.to_le_bytes());
    batch[16..18].copy_from_slice(&3u16.to_le_bytes());
    batch[23] = 77;
    let value = Batch14::read(&mut Decoder::new(&batch)).unwrap();
    assert_eq!(value.lightmap, 5);
    assert_eq!(value.material, 9);
    assert_eq!(value.start_index, 123);
    assert_eq!(value.index_count, 3);
    assert_eq!(value.reserved, 77);
}
#[test]
fn beta_batches_keep_the_eight_bytes_removed_in_version17() {
    assert_eq!(GroupHeader16::size(), 68);
    assert_eq!(Batch16::size(), 32);
    let mut bytes = [0u8; 32];
    bytes[12..16].copy_from_slice(&70_000u32.to_le_bytes());
    bytes[23] = 9;
    bytes[24..].copy_from_slice(b"EXTRA123");
    let value = Batch16::read(&mut Decoder::new(&bytes)).unwrap();
    assert_eq!(value.start_index, 70_000);
    assert_eq!(value.material, 9);
    assert_eq!(&value.extension, b"EXTRA123");
    let chunk = wmo::legacy::GroupChunk16::decode(*b"MOBA", &bytes, 0).unwrap();
    assert_eq!(chunk.encode().unwrap(), bytes);
    assert!(wmo::legacy::GroupChunk16::decode(*b"MOBA", &bytes[..24], 0).is_err());
}
#[test]
fn alpha_embedded_payloads_preserve_typed_lightmaps_and_unknown_chunks() {
    let mut payload = vec![0u8; 128];
    payload.extend(frame(*b"MOPY", &[0x20, 4, 7, 99]));
    payload.extend(frame(*b"MOLM", &[1, 2, 3, 4]));
    payload.extend(frame(*b"TEST", &[9, 8, 7]));
    let group = Group14::decode_payload(&payload).unwrap();
    assert_eq!(group.encode_payload().unwrap(), payload);
    let LegacyGroupChunk::Triangles(values) = &group.chunks[0] else {
        panic!()
    };
    let triangle = values.get(0).unwrap().unwrap();
    assert_eq!(triangle.lightmap, 4);
    assert_eq!(triangle.material, 7);
    assert_eq!(triangle.reserved, 99);
    let framed = frame(*b"MOGP", &payload);
    assert_eq!(Group14::decode(&framed).unwrap().encode().unwrap(), framed);
    let data = vec![0u8; 32772];
    let chunk = GroupChunk14::decode(*b"MOLD", &data, 0).unwrap();
    let LegacyGroupChunk::LightMapData(values) = &chunk else {
        panic!()
    };
    assert_eq!(values.len(), 1);
    assert_eq!(values.as_bytes().as_ptr(), data.as_ptr());
    assert_eq!(chunk.encode().unwrap(), data);
}
#[test]
fn standalone_beta_requires_the_exact_version_and_both_nesting_conventions_roundtrip() {
    for version in [14u32, 16] {
        for nested in [true, false] {
            let width = if version == 14 { 128 } else { 68 };
            let mut body = vec![0u8; width];
            let unknown = frame(*b"TEST", &[1, 2]);
            if nested {
                body.extend_from_slice(&unknown);
            }
            let mut bytes = frame(*b"MVER", &version.to_le_bytes());
            bytes.extend(frame(*b"MOGP", &body));
            if !nested {
                bytes.extend_from_slice(&unknown);
            }
            let encoded = if version == 14 {
                Group14::decode(&bytes).unwrap().encode().unwrap()
            } else {
                Group16::decode(&bytes).unwrap().encode().unwrap()
            };
            assert_eq!(encoded, bytes);
        }
    }
    assert!(Group16::decode(&frame(*b"MOGP", &[0u8; 68])).is_err());
    let group = Group14::new();
    group.validate().unwrap();
    let group = Group16::new();
    group.validate().unwrap();
}

#[test]
fn historical_liquid_magma_coordinates_remain_unsigned() {
    let mut payload = vec![0u8; 30];
    payload[..4].copy_from_slice(&1i32.to_le_bytes());
    payload[4..8].copy_from_slice(&1i32.to_le_bytes());
    payload.extend_from_slice(&[0xfe, 0xff, 0xfd, 0xff]);
    payload.extend_from_slice(&4.5f32.to_le_bytes());
    let GroupChunk14::Liquid(liquid) = GroupChunk14::decode(*b"MLIQ", &payload, 0).unwrap() else {
        panic!()
    };
    let vertices = liquid.magma_vertices().unwrap();
    assert_eq!(
        vertices.get(0).unwrap().unwrap().texture_coordinates,
        [65534, 65533]
    );
    assert_eq!(vertices.as_bytes().as_ptr(), payload[30..].as_ptr());
    assert_eq!(liquid.encode().unwrap(), payload);
}

#[test]
fn alpha_root_references_use_native_material_and_light_tables() {
    use wmo::codec::Array;
    use wmo::group::GroupChunk;
    use wmo::legacy::{LegacyTriangle, Light14, Material14, Root14, RootChunk14};
    let mut root = Root14::new();
    let mut group = Group14::new();
    group.chunks.push(LegacyGroupChunk::Triangles(
        Array::from_records(&[LegacyTriangle {
            material: 0,
            ..Default::default()
        }])
        .unwrap(),
    ));
    assert!(root.validate_group_references(&group).is_err());
    root.push_chunk(RootChunk14::Materials(
        Array::from_records(&[Material14::default()]).unwrap(),
    ))
    .unwrap();
    root.validate_group_references(&group).unwrap();
    group
        .chunks
        .push(LegacyGroupChunk::Common(GroupChunk::LightReferences(
            Array::from_records(&[0u16]).unwrap(),
        )));
    assert!(root.validate_group_references(&group).is_err());
    root.push_chunk(RootChunk14::Lights(
        Array::from_records(&[Light14::default()]).unwrap(),
    ))
    .unwrap();
    root.validate_group_references(&group).unwrap();
    group.header.portal_start = -1;
    group.header.portal_count = 0;
    root.validate_group_references(&group).unwrap();
    group.header.portal_count = 1;
    assert!(root.validate_group_references(&group).is_err());
}
