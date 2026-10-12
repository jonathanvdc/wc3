use wc3::model::wmo::{
    self,
    codec::{Array, Decoder, Encoder},
};
use wmo::{Read as _, RecordLayout as _, Validate as _, Write as _};
#[derive(Clone, Debug, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
struct Entry {
    id: u16,
    coordinates: [f32; 2],
    flags: u8,
    padding: u8,
}
#[derive(Clone, Debug, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
struct Pair<T>(T, T);
#[derive(Clone, Debug, PartialEq, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
struct Empty;
#[derive(wmo::Validate)]
enum Validated<T> {
    Named { value: T },
    Tuple(T),
    Empty,
}
const ENTRY: [u8; 12] = [7, 0, 0, 0, 96, 64, 0, 0, 0, 192, 3, 0];
#[test]
fn derives_follow_field_order_without_alignment_padding() {
    let value = Entry::read(&mut Decoder::new(&ENTRY)).unwrap();
    assert_eq!(value.id, 7);
    assert_eq!(value.coordinates, [3.5, -2.0]);
    assert_eq!(value.flags, 3);
    assert_eq!(Entry::size(), 12);
    value.validate().unwrap();
    let mut output = Encoder::new();
    value.write(&mut output).unwrap();
    assert_eq!(output.finish(), ENTRY);
    let pair = Pair::<u16>::read(&mut Decoder::new(&[1, 0, 2, 0])).unwrap();
    assert_eq!(pair, Pair(1, 2));
    assert_eq!(Pair::<u16>::size(), 4);
    assert_eq!(Empty::size(), 0);
    Empty::read(&mut Decoder::new(&[]))
        .unwrap()
        .validate()
        .unwrap();
    for value in [
        Validated::Named { value: 1u16 },
        Validated::Tuple(2),
        Validated::Empty,
    ] {
        value.validate().unwrap();
    }
}
#[test]
fn fixed_arrays_borrow_unaligned_bytes_and_copy_only_for_edits() {
    let mut bytes = vec![99];
    bytes.extend_from_slice(&ENTRY);
    bytes.extend_from_slice(&ENTRY);
    let mut values = Array::<Entry>::decode(&bytes[1..]).unwrap();
    assert_eq!(values.as_bytes().as_ptr(), bytes[1..].as_ptr());
    assert_eq!(values.len(), 2);
    assert_eq!(values.get(1).unwrap().unwrap().coordinates, [3.5, -2.0]);
    assert!(values.get(2).is_none());
    let replacement = Entry {
        id: 42,
        coordinates: [1.0, 2.0],
        flags: 0,
        padding: 0,
    };
    values.set(1, &replacement).unwrap();
    assert_eq!(values.get(1).unwrap().unwrap(), replacement);
    assert_eq!(values.get(0).unwrap().unwrap().id, 7);
    assert_eq!(&bytes[13..], ENTRY);
    assert_eq!(values.iter().count(), 2);
    assert!(Array::<Entry>::decode(&ENTRY[..11]).is_err());
    assert!(Array::<Empty>::decode(&[]).is_err());
    assert!(values.set(2, &replacement).is_err());
    values.validate().unwrap();
    let rebuilt = Array::from_records(&[replacement]).unwrap();
    assert_eq!(rebuilt.len(), 1);
}
#[test]
fn bounded_reads_report_absolute_origins() {
    let error = Entry::read(&mut Decoder::new(&[7, 0, 0]).with_origin(50)).unwrap_err();
    assert_eq!(error.offset, 52);
    let error = Array::<Entry>::read(&mut Decoder::new(&ENTRY[..11]).with_origin(50)).unwrap_err();
    assert_eq!(error.offset, 50);
}

#[derive(wmo::Write)]
struct VariablePayload<'data> {
    records: Array<'data, Entry>,
}
#[test]
fn write_derive_composes_variable_payloads_without_a_fixed_layout() {
    let payload = VariablePayload {
        records: Array::decode(&ENTRY).unwrap(),
    };
    let mut output = Encoder::new();
    payload.write(&mut output).unwrap();
    assert_eq!(output.finish(), ENTRY);
}
