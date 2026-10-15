use wc3::model::io;
use wc3::model::{
    conversion::ImportOptions,
    formats::m2::{Bundle, Chunked, Document, Flat, V272},
};

#[test]
fn chunked_m2_without_a_core_returns_a_decode_error() {
    let mut bytes = b"PFID".to_vec();
    bytes.extend_from_slice(&4u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    let error = Document::<Chunked<V272>>::decode_m2(&bytes).unwrap_err();
    assert!(error.to_string().contains("missing MD21 core"));
}

#[test]
fn chunked_m2_with_multiple_cores_returns_an_import_error() {
    let core = Document::<Flat<V272>>::new().encode_m2().unwrap();
    let mut bytes = Vec::new();
    for _ in 0..2 {
        bytes.extend_from_slice(b"MD21");
        bytes.extend_from_slice(&(core.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&core);
    }
    let model = Document::<Chunked<V272>>::decode_m2(&bytes).unwrap();
    let bundle = Bundle::from_model(model);
    let error = io::m2::import(&bundle, &ImportOptions::default()).unwrap_err();
    assert!(error.to_string().contains("multiple core payloads"));
}
