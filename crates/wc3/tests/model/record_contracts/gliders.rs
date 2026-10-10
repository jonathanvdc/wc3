use super::*;
use wc3::model::ConversionOptions;
use wc3::model::NoExtensions;

#[test]
fn gliders_keep_every_entry_in_all_versions_and_reject_partial_words() {
    let entries = [Glider { geoset_id: 3 }, Glider { geoset_id: 7 }];
    let expected = [
        b"DILG".as_slice(),
        &8u32.to_le_bytes(),
        &3u32.to_le_bytes(),
        &7u32.to_le_bytes(),
    ]
    .concat();
    assert_eq!(
        GlidersChunk::new(entries.to_vec()).encode_mdx().unwrap(),
        expected
    );
    let mut model = Model::<V800>::new();
    model.set_gliders(&entries);
    let bytes = model.encode_mdx().unwrap();
    let parsed = Model::<V800>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.gliders(), entries);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    let newer = model
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(newer.gliders(), entries);
    let older = newer
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(older.gliders(), entries);
    let mut dynamic = DynamicModel::<NoExtensions>::V1800(
        newer.convert(&ConversionOptions::strict()).unwrap().model,
    );
    assert_eq!(dynamic.gliders(), entries);
    dynamic.set_gliders(&[entries[1], entries[0], entries[1]]);
    assert_eq!(dynamic.gliders(), [entries[1], entries[0], entries[1]]);
    let chunk = ModelChunk::<V800>::from_raw(RawChunk::new(*b"DILG", vec![1, 0, 0, 0])).unwrap();
    assert!(matches!(chunk, ModelChunk::Gliders(_)));
    assert!(ModelChunk::<V800>::from_raw(RawChunk::new(*b"DILG", vec![1, 0, 0])).is_err());
    // Clearing removes every occurrence, including manually appended chunks.
    model
        .chunks
        .push(GlidersChunk::new(entries.to_vec()).into());
    model.set_gliders(&[]);
    assert!(model.chunk(*b"DILG").is_none());
    assert!(model.gliders().is_empty());
    // Existing gated access remains independent of ungated DILG.
    assert!(model.try_face_fx().is_err());
}
