use super::*;
use wc3::model::formats::mdlx::ConversionOptions;
use wc3::model::formats::mdlx::NoExtensions;

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
    let mut model = Document::<V800>::new();
    model.set_gliders(entries);
    let bytes = model.encode_mdx().unwrap();
    let parsed = Document::<V800>::decode_mdx(&bytes).unwrap();
    assert_eq!(parsed.gliders().cloned().collect::<Vec<_>>(), entries);
    assert_eq!(parsed.encode_mdx().unwrap(), bytes);
    let newer = model
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(newer.gliders().cloned().collect::<Vec<_>>(), entries);
    let older = newer
        .convert::<V800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(older.gliders().cloned().collect::<Vec<_>>(), entries);
    let mut dynamic = DynamicDocument::<NoExtensions>::V1800(
        newer.convert(&ConversionOptions::strict()).unwrap().model,
    );
    assert_eq!(dynamic.gliders().copied().collect::<Vec<_>>(), entries);
    dynamic.set_gliders([entries[1], entries[0], entries[1]]);
    assert_eq!(
        dynamic.gliders().copied().collect::<Vec<_>>(),
        [entries[1], entries[0], entries[1]]
    );
    let chunk = ModelChunk::<V800>::from_raw(RawChunk::new(*b"DILG", vec![1, 0, 0, 0])).unwrap();
    assert!(matches!(chunk, ModelChunk::Gliders(_)));
    assert!(ModelChunk::<V800>::from_raw(RawChunk::new(*b"DILG", vec![1, 0, 0])).is_err());
    // Clearing removes every occurrence, including manually appended chunks.
    model
        .chunks
        .push(GlidersChunk::new(entries.to_vec()).into());
    model.set_gliders([]);
    assert!(model.chunk(*b"DILG").is_none());
    assert!(model.gliders().cloned().collect::<Vec<_>>().is_empty());
    // Existing gated access remains independent of ungated DILG.
    assert!(model
        .try_face_fx()
        .map(|records| records.into_iter().cloned().collect::<Vec<_>>())
        .is_err());
}
