//! Shared original 2.5.1 fixture encoding, preserving caller table order.
pub type Fields = Vec<(u16, Vec<u8>)>;
pub type NativeObject = (u32, Fields);
pub fn word(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}
pub fn words(values: &[u32]) -> Vec<u8> {
    values.iter().copied().flat_map(u32::to_le_bytes).collect()
}
pub fn array(values: &[u32]) -> Vec<u8> {
    let mut bytes = word(values.len() as u32);
    bytes.extend(words(values));
    bytes
}
pub fn encode<S: AsRef<[u8]>>(types: &[u32], objects: &[NativeObject], strings: &[S]) -> Vec<u8> {
    let mut bytes = vec![0x11, 0x0b, 0, 0xca, 2, 5, 1, 1];
    for value in [0, objects.len() as u32, types.len() as u32, 0, 0] {
        bytes.extend(word(value));
    }
    for name in types {
        bytes.extend(word(*name));
        bytes.extend(word(0));
    }
    for (kind, fields) in objects {
        let mut body = vec![0x20];
        body.extend(word(*kind));
        body.extend((fields.len() as u16).to_le_bytes());
        for (field, value) in fields {
            body.extend(field.to_le_bytes());
            body.extend(value);
        }
        bytes.extend(word(body.len() as u32));
        bytes.extend(body);
    }
    let offset = bytes.len() as u32;
    bytes[20..24].copy_from_slice(&offset.to_le_bytes());
    bytes.extend(word(strings.len() as u32));
    for string in strings {
        let string = string.as_ref();
        assert!(string.len() < 128);
        bytes.push(string.len() as u8);
        bytes.extend(string);
    }
    bytes
}
#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::formats::pkb::Document;
    #[test]
    fn raw_encoder_preserves_interleaved_type_and_value_string_indices() {
        let strings: [&[u8]; 3] = [b"first value", b"NativeType", b"second value"];
        let bytes = encode(&[1], &[(0, vec![(0, word(2))])], &strings);
        let document = Document::read(&bytes).unwrap();
        assert_eq!(document.types()[0].name, 1);
        assert_eq!(document.strings()[1], b"NativeType");
        assert_eq!(document.objects()[0].fields, &[0, 0, 2, 0, 0, 0]);
        assert_eq!(document.strings()[2], b"second value");
    }
}
