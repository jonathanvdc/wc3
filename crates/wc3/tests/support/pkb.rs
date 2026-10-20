//! Small original baked-effect documents for public behavior tests.
#[path = "pkb_encoding.rs"]
mod encoding;
pub use encoding::{array, word, words};
use encoding::{encode, NativeObject};

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
        encode(&self.types, &self.objects, &self.strings)
    }
}
