use super::{DecodeRecord, EncodeRecord, EncodedArray, RecordCodec};
use crate::model::common::binary::Output;

#[derive(Clone, Debug, PartialEq)]
struct Native<const STRIDE: usize>;
impl<const STRIDE: usize> RecordCodec<u16> for Native<STRIDE> {
    type ReadError = (usize, &'static str);
    type WriteError = &'static str;
    fn size() -> usize {
        STRIDE
    }
    fn read_error(offset: usize, field: &'static str) -> Self::ReadError {
        (offset, field)
    }
    fn write_error(field: &'static str) -> Self::WriteError {
        field
    }
}
impl<const STRIDE: usize> DecodeRecord<u16> for Native<STRIDE> {
    fn decode(bytes: &[u8], origin: usize) -> Result<u16, Self::ReadError> {
        if bytes == [255, 255] {
            return Err((origin, "invalid record"));
        }
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }
}
impl<const STRIDE: usize> EncodeRecord<u16> for Native<STRIDE> {
    fn encode(value: &u16, output: &mut Output) -> Result<(), Self::WriteError> {
        if *value == u16::MAX - 1 {
            *output = Output::new();
            return Ok(());
        }
        let bytes = value.to_le_bytes();
        // Simulates a native record whose encoder violated its advertised layout.
        output
            .bytes(if *value == u16::MAX {
                &bytes[..1]
            } else {
                &bytes
            })
            .map_err(|_| "allocation")
    }
}
type Array<'data> = EncodedArray<'data, u16, Native<2>>;

#[test]
fn borrows_until_first_edit_and_preserves_source() {
    let source = [1, 0, 2, 0];
    let mut records = Array::decode(&source).unwrap();
    assert_eq!(records.as_bytes().as_ptr(), source.as_ptr());
    records.set(0, &3).unwrap();
    let owned = records.as_bytes().as_ptr();
    assert_ne!(owned, source.as_ptr());
    records.set(1, &4).unwrap();
    assert_eq!(records.as_bytes().as_ptr(), owned);
    assert_eq!(
        records.iter().collect::<Result<Vec<_>, _>>().unwrap(),
        [3, 4]
    );
    assert_eq!(source, [1, 0, 2, 0]);
    let view = records.as_borrowed();
    assert_eq!(view.as_bytes().as_ptr(), owned);
    let detached = view.into_owned();
    assert_eq!(detached.as_bytes(), records.as_bytes());
    assert_ne!(detached.as_bytes().as_ptr(), owned);
}

#[test]
fn failed_edits_preserve_existing_storage() {
    let mut records = Array::from_records(&[1, 2]).unwrap();
    let before = records.clone();
    assert_eq!(records.set(0, &u16::MAX), Err("fixed-record width"));
    assert_eq!(records, before);
    assert_eq!(records.replace(&[3, u16::MAX]), Err("fixed-record width"));
    assert_eq!(records, before);
    assert_eq!(
        records.replace(&[3, u16::MAX - 1]),
        Err("fixed-record width")
    );
    assert_eq!(records, before);
    assert_eq!(records.set(2, &0), Err("array record index"));
    assert_eq!(records, before);
}

#[test]
fn checks_framing_zero_width_and_record_origins() {
    assert_eq!(
        Array::decode(&[1]).unwrap_err(),
        (0, "fixed-record array width")
    );
    type Zero<'data> = EncodedArray<'data, u16, Native<0>>;
    assert!(Zero::decode(&[]).is_err());
    assert_eq!(Zero::from_records(&[]), Err("zero-width array record"));
    let records = Array::decode(&[1, 0, 255, 255]).unwrap();
    assert_eq!(records.get(1), Some(Err((2, "invalid record"))));
    assert_eq!(records.get(2), None);
}
