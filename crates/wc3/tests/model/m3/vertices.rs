use std::borrow::Cow;
use wc3::model::m3;
use wc3::model::m3::vertices::{Color, DynamicVertices, ExtraRaw, NoColor, Packed, Raw, Vertices};

#[test]
fn packed_vertices_borrow_unaligned_data_and_edit_native_fields() {
    let mut bytes = [0; 37];
    let vertex = &mut bytes[1..];
    vertex[..4].copy_from_slice(&2.5f32.to_le_bytes());
    vertex[12..16].copy_from_slice(&[255, 0, 0, 0]);
    vertex[16..20].copy_from_slice(&[3, 4, 5, 6]);
    vertex[20..24].copy_from_slice(&[255, 128, 0, 255]);
    vertex[24..28].copy_from_slice(&[10, 20, 30, 40]);
    vertex[28..30].copy_from_slice(&2048i16.to_le_bytes());
    vertex[30..32].copy_from_slice(&(-1024i16).to_le_bytes());
    vertex[32..36].copy_from_slice(&[128, 255, 0, 255]);
    let mut buffer = Vertices::<_, Packed<1, Color>>::new(Cow::Borrowed(&bytes[1..])).unwrap();
    assert_eq!(buffer.as_bytes().as_ptr(), bytes[1..].as_ptr());
    let view = buffer.vertex(0).unwrap();
    assert_eq!(view.position(), [2.5, 0.0, 0.0]);
    assert_eq!(view.weights(), [255, 0, 0, 0]);
    assert_eq!(view.bone_indices(), [3, 4, 5, 6]);
    assert_eq!(view.color(), [10, 20, 30, 40]);
    assert_eq!(view.uv_legacy::<0>(), [1.0, -0.5]);
    assert_eq!(view.normal_unorm()[0], 1.0);
    assert_eq!(view.normal_unorm()[2], -1.0);
    buffer.vertex_mut(0).unwrap().set_position([8.0, 9.0, 10.0]);
    assert_ne!(buffer.as_bytes().as_ptr(), bytes[1..].as_ptr());
    assert_eq!(buffer.vertex(0).unwrap().position(), [8.0, 9.0, 10.0]);
    assert_eq!(&buffer.as_bytes()[12..], &bytes[13..]);
}
#[test]
fn sparse_uv_channels_preserve_type_selected_channel_identity() {
    let mut bytes = vec![0; 32];
    bytes[24..26].copy_from_slice(&123i16.to_le_bytes());
    bytes[26..28].copy_from_slice(&456i16.to_le_bytes());
    let buffer = Vertices::<_, Packed<2>>::new(bytes.as_slice()).unwrap();
    assert_eq!(buffer.vertex(0).unwrap().uv::<1>(), [123, 456]);
    assert!(matches!(
        DynamicVertices::from_flags(bytes.as_slice(), m3::records::VertexFlags(0x0184_0061))
            .unwrap(),
        DynamicVertices::NoColorUv02(_)
    ));
}
#[test]
fn fifth_uv_channel_has_its_own_type_and_wire_slot() {
    let mut bytes = vec![0; 36];
    bytes[28..30].copy_from_slice(&512i16.to_le_bytes());
    bytes[30..32].copy_from_slice(&(-512i16).to_le_bytes());
    let buffer = Vertices::<_, Packed<17>>::new(bytes.as_slice()).unwrap();
    assert_eq!(buffer.vertex(0).unwrap().uv::<4>(), [512, -512]);
    assert!(matches!(
        DynamicVertices::from_flags(bytes.as_slice(), m3::records::VertexFlags(0x4182_007d))
            .unwrap(),
        DynamicVertices::NoColorUv17(_)
    ));
}
#[test]
fn ambiguous_flags_preserve_raw_storage_without_assigning_vertex_boundaries() {
    let bytes = [9, 8, 7];
    let raw = DynamicVertices::from_flags(bytes.as_slice(), m3::records::VertexFlags(0x2182_007d))
        .unwrap();
    assert!(matches!(raw, DynamicVertices::Raw(_)));
    assert_eq!(raw.vertex_count(), None);
    assert_eq!(raw.as_bytes(), bytes);
    assert!(Vertices::<_, Packed<1>>::new(&[0; 31][..]).is_err());
    assert!(Vertices::<_, Raw<0>>::new(&[][..]).is_err());
}
#[test]
fn mutable_slice_and_owned_buffers_use_the_same_vertex_type() {
    let mut bytes = vec![0; 32];
    let mut borrowed = Vertices::<_, Packed<1, NoColor>>::new(bytes.as_mut_slice()).unwrap();
    borrowed.vertex_mut(0).unwrap().set_uv::<0>([44, 55]);
    let mut owned = borrowed.to_owned();
    owned.vertex_mut(0).unwrap().set_weights([1, 2, 3, 4]);
    assert_eq!(owned.vertex(0).unwrap().weights(), [1, 2, 3, 4]);
    assert_eq!(bytes[24..28], [44, 0, 55, 0]);
}
#[test]
fn raw_extra_slot_requires_an_explicit_layout() {
    let mut bytes = vec![0; 36];
    let mut buffer =
        Vertices::<_, Packed<1, NoColor, ExtraRaw>>::new(bytes.as_mut_slice()).unwrap();
    buffer.vertex_mut(0).unwrap().set_extra_bytes([1, 2, 3, 4]);
    assert_eq!(buffer.vertex(0).unwrap().extra_bytes(), [1, 2, 3, 4]);
}
