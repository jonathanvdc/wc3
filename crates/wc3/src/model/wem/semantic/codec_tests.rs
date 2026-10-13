use super::*;

#[test]
fn storage_preference_does_not_change_unpacked_attribute_bytes() {
    let mut doc = RawDocument::new();
    let values = [[0.25f32, -0.5, 2.0]];
    let mut layer = attribute(&mut doc, "position", &values, r::AttrType::F32x3)
        .unwrap()
        .unwrap();
    layer.storage = r::AttributeEncoding::SNorm16;
    assert_eq!(float_values::<3>(&doc, &layer).unwrap(), values);
    assert!(float_values::<2>(&doc, &layer).is_err());
}
