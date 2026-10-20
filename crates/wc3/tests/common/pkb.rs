#[path = "../support/pkb_encoding.rs"]
mod encoding;
use encoding::encode;
pub(super) use encoding::{array, word, Fields};
pub(super) fn fixture(types: &[&[u8]], objects: Vec<(u32, Fields)>, strings: &[&[u8]]) -> Vec<u8> {
    let names: Vec<_> = (0..types.len() as u32).collect();
    let strings: Vec<_> = types.iter().chain(strings).copied().collect();
    encode(&names, &objects, &strings)
}
