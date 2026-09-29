use super::*;

#[test]
fn signed_event_times_and_material_priority_preserve_wire_bits() {
    let frames = [i32::MIN, -3600, 0, i32::MAX];
    let event = EventObject::new(Node::new("Event", 0).unwrap(), u32::MAX, &frames);
    let bytes = event.encode_mdx().unwrap();
    let expected: Vec<_> = frames
        .iter()
        .flat_map(|frame| frame.to_le_bytes())
        .collect();
    assert_eq!(&bytes[bytes.len() - 16..], expected);
    assert_eq!(
        EventObject::decode_mdx(&bytes).unwrap().frames.as_slice(),
        frames
    );
    let mut material = Material::<V800>::new();
    material.priority_plane = -7;
    let bytes = material.encode_mdx().unwrap();
    assert_eq!(&bytes[4..8], &(-7i32).to_le_bytes());
    assert_eq!(
        Material::<V800>::decode_mdx(&bytes).unwrap().priority_plane,
        -7
    );
    let mut model = Model::<V800>::new();
    model.set_materials(&[material]);
    model.set_event_objects(&[event]);
    let converted = model
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert_eq!(converted.materials()[0].priority_plane, -7);
    assert_eq!(converted.event_objects()[0].frames.as_slice(), frames);
}
