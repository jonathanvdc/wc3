use wc3::model::chunks::ModelChunk;
use wc3::model::chunks::RawChunk;
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;

use wc3::model::scene::ModelInfo;
use wc3::model::Model;

#[test]
fn model_info_edit_round_trip() {
    let mut model = Model::<wc3::model::V1800>::new();
    let mut info = ModelInfo::new("Footman").unwrap();
    info.bounds_radius = 42.5;
    info.minimum_extent = [-1.0, -2.0, -3.0];
    info.maximum_extent = [1.0, 2.0, 3.0];
    info.blend_time = 150;
    model.set_model_info(&info);

    let parsed = Model::<wc3::model::V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let actual = parsed.model_info().unwrap();
    assert_eq!(actual.name.text(), "Footman");
    assert_eq!(actual.bounds_radius, 42.5);
    assert_eq!(actual.minimum_extent, [-1.0, -2.0, -3.0]);
    assert_eq!(actual.maximum_extent, [1.0, 2.0, 3.0]);
    assert_eq!(actual.blend_time, 150);
}

#[test]
fn renaming_preserves_the_separate_animation_file_field() {
    let mut model = Model::<wc3::model::V800>::new();
    let mut original = ModelInfo::new("Old").unwrap();
    original
        .animation_file_name
        .set_text("Animations\\Walk.mdx")
        .unwrap();
    let mut data = original.encode_mdx().unwrap();
    data[336..340].copy_from_slice(&[1, 2, 3, 4]);
    data.extend_from_slice(&[5, 6]);
    model
        .chunks
        .push(ModelChunk::from_raw(RawChunk::new(*b"MODL", data)).unwrap());
    let mut info = model.model_info().unwrap();
    assert_eq!(info.animation_file_name.text(), "Animations\\Walk.mdx");
    info.name.set_text("New").unwrap();
    assert_eq!(info.animation_file_name.text(), "Animations\\Walk.mdx");
    model.set_model_info(&info);
    let bytes = model.encode_mdx().unwrap();
    let payload = &bytes[24..];
    assert_eq!(&payload[..4], b"New\0");
    assert_eq!(&payload[80..100], b"Animations\\Walk.mdx\0");
    assert_eq!(&payload[336..340], &[1, 2, 3, 4]);
    assert_eq!(payload.len(), 372);
}

#[test]
fn model_info_rejects_malformed_chunks() {
    let mut model = Model::<wc3::model::V800>::new();
    assert!(ModelChunk::<wc3::model::V800>::from_raw(RawChunk::new(*b"MODL", vec![1])).is_err());
    assert!(model.model_info().is_none());

    let expected = ModelInfo::new("Decoded").unwrap();
    model
        .chunks
        .push(ModelChunk::from(wc3::model::chunks::ModelInfoChunk::new(
            ModelInfo::new("Duplicate").unwrap(),
            Vec::new(),
        )));
    model.set_model_info(&expected);
    assert!(matches!(model.chunks[1], ModelChunk::ModelInfo(_)));
    assert_eq!(model.chunks.len(), 2);
    assert_eq!(model.model_info(), Some(expected.clone()));

    let updated = ModelInfo::new("Updated").unwrap();
    model.set_model_info(&updated);
    assert!(matches!(model.chunks[1], ModelChunk::ModelInfo(_)));
    assert_eq!(model.chunks.len(), 2);
    assert_eq!(model.model_info(), Some(updated));
}

#[test]
fn model_name_and_animation_file_have_independent_capacities() {
    let mut info = ModelInfo::new(&"n".repeat(79)).unwrap();
    assert!(info.name.set_text(&"n".repeat(80)).is_err());
    let path = "a".repeat(259);
    info.animation_file_name.set_text(&path).unwrap();
    assert!(info.animation_file_name.set_text(&"a".repeat(260)).is_err());
    let bytes = info.encode_mdx().unwrap();
    assert_eq!(bytes.len(), 372);
    assert_eq!(&bytes[80..339], path.as_bytes());
    let parsed = ModelInfo::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.name.text(), "n".repeat(79));
    assert_eq!(parsed.animation_file_name.text(), path);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
}
