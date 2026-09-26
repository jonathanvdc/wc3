use wc3_mdx::{Model, ModelInfo, RawChunk};
use wc3_mdx::{ModelChunk, Record};

#[test]
fn model_info_edit_round_trip() {
    let mut model = Model::new(1800);
    let mut info = ModelInfo::new("Footman").unwrap();
    info.set_bounds_radius(42.5);
    info.set_minimum_extent([-1.0, -2.0, -3.0]);
    info.set_maximum_extent([1.0, 2.0, 3.0]);
    info.set_blend_time(150);
    model.set_model_info(&info);

    let parsed = Model::decode(&model.encode().unwrap(), 800).unwrap();
    let actual = parsed.model_info().unwrap();
    assert_eq!(actual.name(), "Footman");
    assert_eq!(actual.bounds_radius(), 42.5);
    assert_eq!(actual.minimum_extent(), [-1.0, -2.0, -3.0]);
    assert_eq!(actual.maximum_extent(), [1.0, 2.0, 3.0]);
    assert_eq!(actual.blend_time(), 150);
}

#[test]
fn preserves_reserved_bytes_when_replacing_model_info() {
    let mut model = Model::new(800);
    let mut data = ModelInfo::new("Old").unwrap().encode().unwrap();
    data[336..340].copy_from_slice(&[1, 2, 3, 4]);
    data.extend_from_slice(&[5, 6]);
    model.push(ModelChunk::from_raw(RawChunk::new(*b"MODL", data), 800));
    let mut info = model.model_info().unwrap();
    info.set_name("New").unwrap();
    model.set_model_info(&info);
    let bytes = model.encode().unwrap();
    let payload = &bytes[24..];
    assert_eq!(&payload[336..340], &[1, 2, 3, 4]);
    assert_eq!(payload.len(), 372);
}

#[test]
fn model_info_skips_malformed_chunks() {
    let mut model = Model::new(800);
    model.push(ModelChunk::from_raw(RawChunk::new(*b"MODL", vec![1]), 800));
    assert!(model.model_info().is_none());

    let expected = ModelInfo::new("Decoded").unwrap();
    model.push(ModelChunk::ModelInfo(wc3_mdx::ModelInfoChunk::new(
        ModelInfo::new("Duplicate").unwrap(),
        Vec::new(),
    )));
    model.set_model_info(&expected);
    assert!(matches!(model.chunks()[1], ModelChunk::ModelInfo(_)));
    assert_eq!(model.chunks().len(), 2);
    assert_eq!(model.model_info(), Some(expected.clone()));

    let updated = ModelInfo::new("Updated").unwrap();
    model.set_model_info(&updated);
    assert!(matches!(model.chunks()[1], ModelChunk::ModelInfo(_)));
    assert_eq!(model.chunks().len(), 2);
    assert_eq!(model.model_info(), Some(updated));
}
