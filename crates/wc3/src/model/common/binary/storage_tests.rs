use super::{AddressSpace, Cursor, Output, OutputError, ValueError};

#[test]
fn cursors_check_random_access_and_preserve_failed_reads() {
    let mut cursor = Cursor::new(&[1, 2, 3]).with_origin(40);
    assert_eq!(cursor.value::<u16>().unwrap(), 513);
    assert_eq!(cursor.position(), 2);
    assert!(matches!(cursor.value::<u16>(), Err(ValueError::Bounds(_))));
    assert_eq!(cursor.position(), 2);
    assert_eq!(cursor.absolute_position(), 42);
    assert_eq!(cursor.remaining(), 1);
    assert!(cursor.seek(4).is_err());
    assert_eq!(cursor.position(), 2);
    assert_eq!(cursor.at(3).unwrap().remaining(), 0);
    let error = AddressSpace::new(&[1])
        .with_origin(8)
        .range(usize::MAX, 2)
        .unwrap_err();
    assert_eq!(error.offset, usize::MAX);
    assert_eq!(error.remaining, 0);
}

#[test]
fn output_reserves_aligned_blocks_and_patches_only_existing_bytes() {
    let mut output = Output::from_bytes(vec![7]);
    let at = output.reserve(4, 3).unwrap();
    assert_eq!(at, 3);
    assert_eq!(output.data(), &[7, 0, 0, 0, 0, 0, 0]);
    output.value(at, &0x12345678u32).unwrap();
    assert_eq!(
        AddressSpace::new(output.data()).value::<u32>(at).unwrap(),
        0x12345678
    );
    let previous = output.data().to_vec();
    assert_eq!(output.reserve(1, 0), Err(OutputError::Alignment));
    assert_eq!(output.reserve(usize::MAX, 8), Err(OutputError::Overflow));
    assert!(matches!(
        output.patch(6, &[1, 2]),
        Err(OutputError::Bounds(_))
    ));
    assert_eq!(output.data(), previous);
    output.append(&[9]).unwrap();
    assert_eq!(output.finish().last(), Some(&9));
}

#[test]
fn zero_width_scalars_fail_without_consuming_or_changing_storage() {
    let mut cursor = Cursor::new(&[1, 2]);
    assert_eq!(cursor.value::<[u8; 0]>(), Err(ValueError::Invalid));
    assert_eq!(cursor.position(), 0);
    let mut output = Output::from_bytes(vec![1, 2]);
    assert_eq!(output.value(0, &[0u8; 0]), Err(OutputError::InvalidValue));
    assert_eq!(output.data(), [1, 2]);
}
