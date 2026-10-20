#[path = "support/pkb.rs"]
mod pkb_fixture;
use pkb_fixture::{Builder, array, word, words};
use wc3::model::formats::pkb::Document;

fn cache_fixture(
    field_flags: u32,
    constant_slot: Option<i32>,
    extra: Option<(u16, Vec<u8>)>,
) -> Vec<u8> {
    let mut builder = Builder::default();
    let name = builder.string(b"self.metaData");
    let mut fields = vec![
        (0, word(name)),
        (1, word(5)),
        (2, word(1)),
        (3, word(field_flags)),
    ];
    if let Some(slot) = constant_slot {
        fields.push((4, word(slot as u32)));
    }
    builder.object(b"CLayerCompileCacheField", fields);
    builder.object(b"CLayerCompileCacheEvent", vec![]);
    builder.object(b"CCompilerBlobCache", vec![]);
    let mut constant = word(1);
    constant.extend(words(&[u32::MAX, 2, 3, 4]));
    let mut range = word(1);
    range.extend(words(&[0, 5, 6, 7]));
    let mut fields = vec![
        (0, constant),
        (1, range),
        (2, array(&[1])),
        (10, array(&[3])),
        (12, word(2)),
        (14, vec![1]),
        (22, word((-1.0f32).to_bits())),
    ];
    if let Some(extra) = extra {
        fields.push(extra);
    }
    builder.object(b"CLayerCompileCache", fields);
    builder.encode()
}
#[test]
fn layer_storage_borrows_unaligned_words_and_preserves_metadata_layout_and_sparse_zero() {
    let bytes = cache_fixture(5, None, None);
    let document = Document::read(&bytes).unwrap();
    let cache = document.layer_cache(4).unwrap();
    assert!(!cache.constants.is_owned());
    assert!(!cache.ranges.is_owned());
    assert_eq!(cache.constants.get(0), Some([u32::MAX, 2, 3, 4]));
    assert_eq!(cache.ranges.get(0), Some([0, 5, 6, 7]));
    assert_eq!(cache.fields[0].name, b"self.metaData");
    assert_eq!(cache.fields[0].native_type.0, 5);
    assert_eq!(cache.fields[0].native_type.value_type(), None);
    assert_eq!(cache.fields[0].constant_slot, 0);
    assert_eq!(cache.fields[0].range_slot, 0);
    assert!(cache.evolution_side_effects);
    assert!(!cache.evolve_on_death);
    assert_eq!(cache.lod, [Some(-1.0), None]);
    assert_eq!(cache.fixed, [3]);
    assert!(cache.varying.is_none());
    assert!(cache.root_event.payload.is_empty());
    let bytes = cache_fixture(1, Some(-1), None);
    assert_eq!(
        Document::read(&bytes)
            .unwrap()
            .layer_cache(4)
            .unwrap()
            .fields[0]
            .constant_slot,
        -1
    );
}
#[test]
fn unknown_layer_fields_invalid_booleans_and_storage_sentinels_fail_without_losing_source() {
    for bytes in [
        cache_fixture(1, Some(-2), None),
        cache_fixture(1, None, Some((24, word(0)))),
        cache_fixture(1, None, Some((15, vec![2]))),
        cache_fixture(1, None, Some((9, word(0)))),
    ] {
        let document = Document::read(&bytes).unwrap();
        assert!(document.layer_cache(4).is_err());
        assert_eq!(document.as_bytes(), bytes);
    }
}
#[test]
fn event_payloads_check_exact_declared_footprints_and_preserve_unsigned_max_range() {
    for declared_bytes in [4, 8, u32::MAX] {
        let mut builder = Builder::default();
        let name = builder.string(b"Position");
        builder.object(
            b"CLayerCompileCacheEventPayload",
            vec![
                (0, word(name)),
                (1, word(31)),
                (2, word(4)),
                (5, word(u32::MAX)),
            ],
        );
        builder.object(
            b"CLayerCompileCacheEvent",
            vec![(2, array(&[1])), (3, word(declared_bytes))],
        );
        let bytes = builder.encode();
        let document = Document::read(&bytes).unwrap();
        if declared_bytes == 4 {
            let event = document.layer_event(2).unwrap();
            assert_eq!(event.payload[0].range_slot, u32::MAX);
            assert_eq!(event.payload[0].kind, 0);
            assert_eq!(event.payload[0].flags, 0);
        } else {
            assert!(document.layer_event(2).is_err());
        }
    }
}
