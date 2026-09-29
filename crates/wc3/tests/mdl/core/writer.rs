use super::*;

#[test]
fn finite_float_output_preserves_bits_including_signed_zero() {
    let special = [
        0, 0x80000000, 1, 0x007fffff, 0x00800000, 0x7f7fffff, 0xff7fffff, 0x3f800000,
    ];
    let mut state = 42u32;
    for bits in special.into_iter().chain((0..10_000).map(|_| {
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        state
    })) {
        let value = f32::from_bits(bits);
        if value.is_finite() {
            let text = print(&value);
            assert_eq!(f32::decode_mdl(&text).unwrap().to_bits(), bits, "{text}");
        }
    }
    assert_eq!(print(&50.0f32), "50.0");
    assert_eq!(print(&1e20f32), "1.0e20");
    assert_eq!(print(&-0.0f32), "-0.0");
}

#[test]
fn encode_mdl_preserves_unicode_and_checks_block_balance() {
    let texture = Texture::new("雪.blp").unwrap();
    let text = texture.encode_mdl().unwrap();
    assert!(text.contains("雪.blp"));
    assert_eq!(Texture::decode_mdl(&text).unwrap(), texture);

    struct Unbalanced;
    impl mdl::Write for Unbalanced {
        fn write_mdl<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), WriteError> {
            writer.begin_block("Unbalanced")
        }
    }
    assert!(matches!(
        Unbalanced.encode_mdl(),
        Err(WriteError::UnbalancedBlocks)
    ));
}

#[test]
fn printing_rejects_unrepresentable_binary_fields() {
    let mut texture = Texture::new("a").unwrap();
    texture.flags = TextureFlags(4);
    assert!(matches!(
        Writer::new(io::sink()).write(&texture),
        Err(WriteError::Unsupported(_))
    ));
    let mut sequence = Sequence::new("a", [0, 1]).unwrap();
    sequence.flags = SequenceFlags(2);
    assert!(matches!(
        Writer::new(io::sink()).write(&sequence),
        Err(WriteError::Unsupported(_))
    ));
    for bytes in [[b'a', 0, 1, 0], [0xff, 0, 0, 0], [b'a'; 4]] {
        assert!(matches!(
            Writer::new(io::sink()).write(&FixedText::from_bytes(bytes)),
            Err(WriteError::Unsupported(_))
        ));
    }
    let mut bytes = Texture::new("a").unwrap().encode_mdx().unwrap();
    bytes[260] = 1;
    assert!(matches!(
        Writer::new(io::sink()).write(&Texture::decode_mdx(&bytes).unwrap()),
        Err(WriteError::Unsupported(_))
    ));
}

#[test]
fn writer_checks_balance_identifiers_and_io_errors() {
    let mut writer = Writer::new(io::sink());
    assert!(matches!(
        writer.end_block(),
        Err(WriteError::UnbalancedBlocks)
    ));
    writer.begin_block("Outer").unwrap();
    assert!(matches!(writer.finish(), Err(WriteError::UnbalancedBlocks)));
    assert!(matches!(
        Writer::new(io::sink()).flag("Bad Name"),
        Err(WriteError::InvalidIdentifier)
    ));
    struct Failing;
    impl Write for Failing {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    assert!(
        matches!(Writer::new(Failing).write(&1u32), Err(WriteError::Io(error)) if error.kind() == io::ErrorKind::BrokenPipe)
    );
}
