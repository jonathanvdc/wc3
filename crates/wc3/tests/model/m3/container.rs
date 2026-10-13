//! Independently authored index-table wire fixtures.
use m3::codec::{Decoder, Encoder};
use wc3::model::m3::{
    self, Document, Envelope, Md33, Md34, ReadError, Record, RecordLayout, Ref, WriteError,
};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Word(u32);
impl Record for Word {
    const TAG: [u8; 4] = *b"U32_";
    const VERSION: u32 = 0;
}
impl<E: Envelope> RecordLayout<E> for Word {
    fn size() -> usize {
        4
    }
}
impl<'data, E: Envelope> m3::Read<'data, E> for Word {
    fn read(input: &mut Decoder<'data, E>) -> Result<Self, ReadError> {
        Ok(Self(input.read()?))
    }
}
impl<E: Envelope> m3::Write<E> for Word {
    fn write(&self, output: &mut Encoder<E>) -> Result<(), WriteError> {
        output.write(&self.0)
    }
}
fn words_fixture(magic: [u8; 4]) -> Vec<u8> {
    let mut bytes = vec![0xaa; 96];
    let mut wire = magic;
    wire.reverse();
    bytes[..4].copy_from_slice(&wire);
    bytes[4..8].copy_from_slice(&64u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
    bytes[12..20].fill(0);
    if magic == *b"MD34" {
        bytes[20..24].fill(0);
    }
    bytes[32..36].copy_from_slice(&0x1122_3344u32.to_le_bytes());
    bytes[36..40].copy_from_slice(&0x5566_7788u32.to_le_bytes());
    for (slot, tag, offset, count, revision) in
        [(0, magic, 0u32, 1u32, 11u32), (1, *b"U32_", 32, 2, 0)]
    {
        let at = 64 + slot * 16;
        let mut tag = tag;
        tag.reverse();
        bytes[at..at + 4].copy_from_slice(&tag);
        bytes[at + 4..at + 8].copy_from_slice(&offset.to_le_bytes());
        bytes[at + 8..at + 12].copy_from_slice(&count.to_le_bytes());
        bytes[at + 12..at + 16].copy_from_slice(&revision.to_le_bytes());
    }
    bytes
}
fn check_fixture<E: Envelope>() {
    let bytes = words_fixture(E::MAGIC);
    let document = Document::<E>::read(&bytes).unwrap();
    document.validate().unwrap();
    assert_eq!(document.write().unwrap(), bytes);
    let id = document.sections().nth(1).unwrap().0;
    let array = document.array::<Word>(id).unwrap();
    assert_eq!(array.as_bytes().as_ptr(), bytes[32..].as_ptr());
    assert_eq!(array.get(0).unwrap().unwrap(), Word(0x1122_3344));
    let first = document
        .resolve(Ref::<Word>::from_section(1, id.index()))
        .unwrap();
    let second = document
        .resolve(Ref::<Word>::from_section(1, id.index()))
        .unwrap();
    assert_eq!(first.as_bytes().as_ptr(), second.as_bytes().as_ptr());
    assert!(document
        .resolve(Ref::<Word>::from_section(3, id.index()))
        .is_err());
    assert!(document.resolve(Ref::<Word>::from_section(1, 999)).is_err());
    assert!(document
        .resolve(Ref::<Word>::from_section(0, 999))
        .unwrap()
        .is_empty());
    let mut document = document.into_owned();
    document.replace(id, &[Word(7), Word(8), Word(9)]).unwrap();
    let encoded = document.write().unwrap();
    let decoded = Document::<E>::read(&encoded).unwrap();
    assert!(decoded.array::<Word>(id).is_err());
    let id = decoded.section_id(id.index()).unwrap();
    assert_eq!(decoded.array::<Word>(id).unwrap().len(), 3);
    assert_eq!(
        decoded.array::<Word>(id).unwrap().get(2).unwrap().unwrap(),
        Word(9)
    );
}
#[test]
fn md34_indexed_arrays_and_shared_references() {
    check_fixture::<Md34>();
}
#[test]
fn md33_short_header_and_arrays() {
    check_fixture::<Md33>();
}
#[test]
fn malformed_index_offsets_spans_and_counts_fail() {
    for (offset, value) in [
        (4, 95u32),
        (8, u32::MAX),
        (84, 65),
        (84, 97),
        (88, u32::MAX),
    ] {
        let mut bytes = words_fixture(*b"MD34");
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert!(Document::<Md34>::read(&bytes).is_err(), "offset {offset}");
    }
}
#[test]
fn md33_rejects_unrepresentable_reference_flags() {
    let mut document = Document::<Md33>::read(&words_fixture(*b"MD33"))
        .unwrap()
        .into_owned();
    document.set_root(Ref::<m3::Any>::with_flags(
        0,
        0,
        m3::records::ReferenceFlags(1),
    ));
    assert!(document.write().is_err());
}
