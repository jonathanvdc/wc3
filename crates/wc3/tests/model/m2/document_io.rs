//! Full companion documents participate in the format-qualified codec traits.
use wc3::model::m2;
use wc3::model::m2::codec::{Decoder, Encoder};
use wc3::model::m2::companions::{BoneFile, GeometryFile, PhysicsFile};

fn frame(tag: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = tag.to_vec();
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes
}
fn check<'data, T: m2::Read<'data> + m2::Write>(bytes: &'data [u8]) {
    let mut input = Decoder::new(bytes, 274, 0);
    input.bytes(3).unwrap();
    let document = T::read(&mut input).unwrap();
    assert_eq!(input.position(), bytes.len());
    let mut output = Encoder::new(274, 0);
    output.bytes(&bytes[..3]).unwrap();
    document.write(&mut output).unwrap();
    assert_eq!(output.finish(), bytes);
}
#[test]
fn full_documents_use_the_remaining_bounded_address_space() {
    let mut bone = vec![9, 8, 7];
    bone.extend_from_slice(&1u32.to_le_bytes());
    bone.extend(frame(*b"BIDA", &[]));
    bone.extend(frame(*b"BOMT", &[]));
    check::<BoneFile>(&bone);
    let mut physics = vec![9, 8, 7];
    // PHYS chunk tags have their native reversed wire spelling.
    physics.extend(frame(*b"SYHP", &0u16.to_le_bytes()));
    check::<PhysicsFile>(&physics);
    let mut geometry = vec![9, 8, 7];
    let mut stream = b"3F32".to_vec();
    for value in [1f32, 2.0, 3.0] {
        stream.extend_from_slice(&value.to_le_bytes());
    }
    geometry.extend(frame(*b"VPOS", &stream));
    check::<GeometryFile>(&geometry);
}
#[test]
fn full_document_traits_reject_truncated_framing() {
    let bytes = b"SYHP\x02\0\0\0\0";
    assert!(<PhysicsFile as m2::Read>::read(&mut Decoder::new(bytes, 274, 0)).is_err());
    let bytes = b"VPOS\x08\0\0\0";
    assert!(<GeometryFile as m2::Read>::read(&mut Decoder::new(bytes, 274, 0)).is_err());
}

#[test]
fn document_errors_retain_the_enclosing_source_offset() {
    let mut bytes = vec![9, 8, 7];
    bytes.extend_from_slice(&2u32.to_le_bytes());
    let mut decoder = Decoder::new(&bytes, 274, 0).with_origin(20);
    decoder.bytes(3).unwrap();
    let error = <BoneFile as m2::Read>::read(&mut decoder).unwrap_err();
    assert_eq!(error.offset, 23);
}
