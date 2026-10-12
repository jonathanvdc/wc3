//! Document trait adapters preserve the address-space origin of bounded inputs.
use m2::{Read as _, Write as _};
use wc3::model::m2::{
    self,
    codec::{Decoder, Encoder},
    DynamicModel, Flat, Model, V264,
};

fn native_header(version: u32) -> Vec<u8> {
    let mut bytes = vec![0; 304];
    bytes[..4].copy_from_slice(b"MD20");
    bytes[4..8].copy_from_slice(&version.to_le_bytes());
    bytes
}
#[test]
fn native_document_read_after_a_prefix_uses_a_new_local_offset_base() {
    let mut bytes = b"prefix".to_vec();
    bytes.extend_from_slice(&native_header(264));
    let mut input = Decoder::new(&bytes, 0, 0).with_origin(100);
    input.bytes(6).unwrap();
    let model = Model::<Flat<V264>>::read(&mut input).unwrap();
    assert_eq!(model.version(), 264);
    assert_eq!(input.position(), bytes.len());
}
#[test]
fn native_document_read_retains_absolute_error_locations() {
    let mut bytes = b"prefix".to_vec();
    bytes.extend_from_slice(&native_header(999));
    let mut input = Decoder::new(&bytes, 0, 0).with_origin(100);
    input.bytes(6).unwrap();
    assert_eq!(
        Model::<Flat<V264>>::read(&mut input).unwrap_err().offset,
        110
    );
    let mut input = Decoder::new(&bytes, 0, 0).with_origin(100);
    input.bytes(6).unwrap();
    assert_eq!(DynamicModel::read(&mut input).unwrap_err().offset, 110);
}

#[test]
fn document_write_retains_native_version_under_an_unrelated_encoder_context() {
    let header = native_header(264);
    let model = Model::<Flat<V264>>::decode_m2(&header).unwrap();
    let mut output = Encoder::new(999, 0);
    output.bytes(b"prefix").unwrap();
    model.write(&mut output).unwrap();
    let bytes = output.finish();
    assert_eq!(&bytes[..6], b"prefix");
    assert_eq!(&bytes[6..], header);
}
