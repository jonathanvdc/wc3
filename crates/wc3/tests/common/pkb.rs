pub(super) type Fields = Vec<(u16, Vec<u8>)>;
pub(super) fn word(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}
pub(super) fn array(values: &[u32]) -> Vec<u8> {
    let mut bytes = word(values.len() as u32);
    bytes.extend(values.iter().flat_map(|value| value.to_le_bytes()));
    bytes
}
pub(super) fn fixture(types: &[&[u8]], objects: Vec<(u32, Fields)>, strings: &[&[u8]]) -> Vec<u8> {
    let mut bodies = Vec::new();
    for (kind, fields) in &objects {
        let mut body = vec![0x20];
        body.extend(kind.to_le_bytes());
        body.extend((fields.len() as u16).to_le_bytes());
        for (field, value) in fields {
            body.extend(field.to_le_bytes());
            body.extend(value);
        }
        bodies.extend((body.len() as u32).to_le_bytes());
        bodies.extend(body);
    }
    let mut bytes = vec![0x11, 0x0b, 0x00, 0xca, 2, 5, 1, 1];
    for value in [
        0,
        objects.len() as u32,
        types.len() as u32,
        28 + types.len() as u32 * 8 + bodies.len() as u32,
        0,
    ] {
        bytes.extend(value.to_le_bytes());
    }
    for index in 0..types.len() as u32 {
        bytes.extend(index.to_le_bytes());
        bytes.extend(0u32.to_le_bytes());
    }
    bytes.extend(bodies);
    bytes.extend(((types.len() + strings.len()) as u32).to_le_bytes());
    for string in types.iter().chain(strings) {
        assert!(string.len() < 128);
        bytes.push(string.len() as u8);
        bytes.extend(*string);
    }
    bytes
}
