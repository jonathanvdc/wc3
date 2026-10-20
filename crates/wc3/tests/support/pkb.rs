//! Small original baked-effect documents for public behavior tests.
type NativeObject = (u32, Vec<(u16, Vec<u8>)>);

#[derive(Default)]
pub struct Builder {
    strings: Vec<Vec<u8>>,
    types: Vec<u32>,
    objects: Vec<NativeObject>,
}
impl Builder {
    pub fn string(&mut self, value: &[u8]) -> u32 {
        if let Some(index) = self.strings.iter().position(|string| string == value) {
            return index as u32;
        }
        let index = self.strings.len() as u32;
        self.strings.push(value.to_vec());
        index
    }
    pub fn object(&mut self, name: &[u8], fields: Vec<(u16, Vec<u8>)>) -> u32 {
        let name = self.string(name);
        let kind = if let Some(index) = self.types.iter().position(|entry| *entry == name) {
            index as u32
        } else {
            let index = self.types.len() as u32;
            self.types.push(name);
            index
        };
        self.objects.push((kind, fields));
        self.objects.len() as u32
    }
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = vec![0x11, 0x0b, 0, 0xca, 2, 5, 1, 1];
        for value in [0, self.objects.len() as u32, self.types.len() as u32, 0, 0] {
            bytes.extend(word(value));
        }
        for kind in &self.types {
            bytes.extend(word(*kind));
            bytes.extend(word(0));
        }
        for (kind, fields) in &self.objects {
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
        bytes.extend(word(self.strings.len() as u32));
        for string in &self.strings {
            assert!(string.len() < 128);
            bytes.push(string.len() as u8);
            bytes.extend(string);
        }
        bytes
    }
}
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
