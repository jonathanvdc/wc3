use wc3::model::wem::{self, Array, Decoder, Encoder, InlineList, InlineOption};

#[test]
fn mixed_optional_widths_preserve_presence_and_neighbor_boundaries() {
    let mut bytes = vec![0, 7];
    bytes.extend_from_slice(&0x12345678u32.to_le_bytes());
    bytes.push(0);
    let mut array = Array::<InlineOption<u32>>::read(&bytes, 3, 1).unwrap();
    assert_eq!(array.as_bytes().as_ptr(), bytes.as_ptr());
    assert_eq!(array.get(0).unwrap().unwrap().value, None);
    assert_eq!(array.get(1).unwrap().unwrap().presence, 7);
    assert_eq!(array.get(1).unwrap().unwrap().value, Some(0x12345678));
    array.set(0, &InlineOption::from(Some(99))).unwrap();
    assert_ne!(array.as_bytes().as_ptr(), bytes.as_ptr());
    assert_eq!(array.get(0).unwrap().unwrap().value, Some(99));
    assert_eq!(array.get(1).unwrap().unwrap().presence, 7);
    assert_eq!(array.get(1).unwrap().unwrap().value, Some(0x12345678));
    assert_eq!(array.get(2).unwrap().unwrap().value, None);
}

#[test]
fn inline_lists_are_counted_inside_the_record_without_section_references() {
    let bytes = [2, 0, 0, 0, 0x34, 0x12, 0x78, 0x56];
    let mut decoder = Decoder::new(&bytes, 1);
    let values: InlineList<u16> = decoder.read().unwrap();
    assert_eq!(values.0, [0x1234, 0x5678]);
    assert_eq!(decoder.remaining(), 0);
    let mut encoder = Encoder::new(1);
    encoder.write(&values).unwrap();
    assert_eq!(encoder.finish(), bytes);
}

#[test]
fn variable_record_bounds_reject_impossible_counts_and_truncated_values() {
    assert!(Array::<InlineOption<u32>>::read(&[1, 0, 0], 1, 1).is_err());
    assert!(Array::<InlineOption<u32>>::read(&[0], u32::MAX, 1).is_err());
    assert!(Decoder::new(&[255, 255, 255, 255], 1)
        .read::<InlineList<u16>>()
        .is_err());
    let mut encoder = Encoder::new(1);
    assert!(encoder
        .write(&InlineOption {
            presence: 0,
            value: Some(1u32)
        })
        .is_err());
}

#[derive(Clone, Debug)]
struct SkipOnlyLeaf;
impl<'data> wem::Read<'data> for SkipOnlyLeaf {
    fn read(_: &mut Decoder<'data>) -> Result<Self, wem::ReadError> {
        Err(wem::ReadError::invalid(0, "leaf decoded during indexing"))
    }
    fn skip(input: &mut Decoder<'data>) -> Result<(), wem::ReadError> {
        input.bytes(1)?;
        Ok(())
    }
}
impl wem::RecordLayout for SkipOnlyLeaf {
    fn minimum_size() -> usize {
        1
    }
}
#[derive(wem::Read, wem::RecordLayout)]
struct LazyList {
    values: InlineList<SkipOnlyLeaf>,
}

#[test]
fn variable_row_indexing_uses_compositional_skip_without_decoding_inline_elements() {
    let bytes = [2, 0, 0, 0, 13, 17, 1, 0, 0, 0, 19];
    let array = Array::<LazyList>::read(&bytes, 2, 1).unwrap();
    assert_eq!(array.len(), 2);
    assert_eq!(array.as_bytes().as_ptr(), bytes.as_ptr());
    assert!(array.get(0).unwrap().is_err());
    assert!(array.get(1).unwrap().is_err());
    let empty = Array::<LazyList>::read(&[0; 4], 1, 1).unwrap();
    assert!(empty.get(0).unwrap().unwrap().values.0.is_empty());
}
