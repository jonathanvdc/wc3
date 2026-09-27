use wc3_mdx::io::{Cursor, Encoder, FixedText, ValueError};

#[test]
fn fixed_text_preserves_raw_bytes_until_edited() {
    let bytes = [b'A', 0, 0xff, b'Z'];
    let mut cursor = Cursor::new(&bytes);
    let mut field: FixedText<4> = cursor.read().unwrap();
    cursor.finish().unwrap();
    assert_eq!(field.text(), "A");
    assert_eq!(field.as_bytes(), &bytes);

    let mut encoded = Vec::new();
    Encoder::new(&mut encoded).write(&field).unwrap();
    assert_eq!(encoded, bytes);

    field.set_text("Hi").unwrap();
    assert_eq!(field.as_bytes(), b"Hi\0\0");
    assert_eq!(field.text(), "Hi");
}

#[test]
fn fixed_text_checks_capacity_and_nuls() {
    let mut field = FixedText::<4>::default();
    assert_eq!(
        field.set_text("four"),
        Err(ValueError::InvalidString { max_bytes: 3 })
    );
    assert_eq!(
        field.set_text("a\0b"),
        Err(ValueError::InvalidString { max_bytes: 3 })
    );
    assert_eq!(field.as_bytes(), &[0; 4]);
}
