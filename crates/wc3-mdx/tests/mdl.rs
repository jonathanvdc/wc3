use std::io::{self, Write};
use wc3_mdx::animation::{GlobalSequence, Sequence, SequenceFlags};
use wc3_mdx::geometry::PivotPoint;
use wc3_mdx::materials::{Texture, TextureFlags};
use wc3_mdx::mdl::{
    ReadError, ReadErrorKind, Lexer, MdlRead, MdlWrite, MdlWriter, Parser, Span, TokenKind, WriteError,
};
use wc3_mdx::{FixedText, Readable, Writable};

fn print<T: MdlWrite>(value: &T) -> String {
    let mut writer = MdlWriter::new(Vec::new());
    writer.write(value).unwrap();
    String::from_utf8(writer.finish().unwrap()).unwrap()
}

#[test]
fn tokens_borrow_source_and_cover_all_punctuation() {
    let source = "\u{feff}// comment\r\nstatic TextureID -12 <= 1, { 3.5e-8 }: \"a\\b\r\nc\"";
    let tokens = Lexer::new(source).collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(
        tokens.iter().map(|t| t.kind).collect::<Vec<_>>(),
        vec![
            TokenKind::Ident("static"),
            TokenKind::Ident("TextureID"),
            TokenKind::Number("-12"),
            TokenKind::Slot,
            TokenKind::Number("1"),
            TokenKind::Comma,
            TokenKind::OpenBrace,
            TokenKind::Number("3.5e-8"),
            TokenKind::CloseBrace,
            TokenKind::Colon,
            TokenKind::Quoted("a\\b\r\nc"),
        ]
    );
    for token in tokens {
        match token.kind {
            TokenKind::Ident(raw) | TokenKind::Number(raw) => {
                assert_eq!(raw.as_ptr(), source[token.span.start..].as_ptr());
                assert_eq!(&source[token.span.start..token.span.end], raw);
            }
            TokenKind::Quoted(raw) => {
                assert_eq!(raw.as_ptr(), source[token.span.start + 1..].as_ptr())
            }
            _ => {}
        }
    }
}

#[test]
fn lexical_errors_are_precise_and_terminal() {
    for (source, kind, span) in [
        (
            "\"unterminated",
            ReadErrorKind::UnterminatedString,
            Span::new(0, 13),
        ),
        ("é", ReadErrorKind::InvalidCharacter, Span::new(0, 2)),
        (
            "/* comment */",
            ReadErrorKind::InvalidCharacter,
            Span::new(0, 1),
        ),
        ("<", ReadErrorKind::InvalidCharacter, Span::new(0, 1)),
    ] {
        let mut lexer = Lexer::new(source);
        assert_eq!(lexer.next().unwrap().unwrap_err(), ReadError::new(span, kind));
        assert!(lexer.next().is_none());
    }
    assert_eq!(
        ReadError::new(Span::new(6, 6), ReadErrorKind::UnknownField).line_column("é\r\nx\ry"),
        (3, 1)
    );
    let source = "// é\r\nBitmap {\n Image \"ok\",\n Bogus 3,\n}";
    let error = Texture::parse_mdl(source).unwrap_err();
    assert_eq!(error.kind, ReadErrorKind::UnknownField);
    assert_eq!(error.line_column(source), (4, 2));
    assert_eq!(&source[error.span.start..error.span.end], "Bogus");
    assert!(error
        .diagnostic(source)
        .to_string()
        .contains("4:2 near \"Bogus\""));
}

#[test]
fn strings_are_literal_and_fixed_text_checks_byte_capacity() {
    let source = "\"a\\n\\\\\r\n雪\\\"";
    let mut parser = Parser::new(source);
    assert_eq!(parser.read_string().unwrap(), "a\\n\\\\\r\n雪\\");
    parser.finish().unwrap();
    assert_eq!(print(&FixedText::<32>::parse_mdl(source).unwrap()), source);
    assert_eq!(FixedText::<4>::parse_mdl("\"雪\"").unwrap().text(), "雪");
    assert!(matches!(
        FixedText::<3>::parse_mdl("\"雪\"").unwrap_err().kind,
        ReadErrorKind::InvalidString { max_bytes: 2 }
    ));
    assert!(FixedText::<8>::parse_mdl("\"a\0b\"").is_err());
    assert!(matches!(
        print_result("a\"b"),
        Err(WriteError::InvalidString)
    ));
    assert!(matches!(
        print_result("a\0b"),
        Err(WriteError::InvalidString)
    ));
}
fn print_result(value: &str) -> Result<(), WriteError> {
    MdlWriter::new(io::sink()).write(value)
}

#[test]
fn typed_numbers_check_full_spelling_ranges_and_nonfinite_literals() {
    assert_eq!(u32::parse_mdl("4294967295").unwrap(), u32::MAX);
    assert_eq!(i32::parse_mdl("-2147483648").unwrap(), i32::MIN);
    assert_eq!(i32::parse_mdl("-3600").unwrap(), -3600);
    assert_eq!(f32::parse_mdl("1.5e-08").unwrap(), 1.5e-8);
    for raw in ["4294967296", "-1", "1.5", "1e2", "1foo", "++1"] {
        assert!(u32::parse_mdl(raw).is_err(), "{raw}");
    }
    for raw in ["1e", "1e1000", "Infinity", "1..2", "+"] {
        assert!(f32::parse_mdl(raw).is_err(), "{raw}");
    }
    for raw in ["nan", "NaN", "NAN"] {
        assert!(f32::parse_mdl(raw).unwrap().is_nan());
    }
    for raw in ["inf", "INF", "+Inf"] {
        assert_eq!(f32::parse_mdl(raw).unwrap(), f32::INFINITY);
    }
    assert_eq!(f32::parse_mdl("-INF").unwrap(), f32::NEG_INFINITY);
    assert_eq!(print(&f32::NAN), "nan");
    assert_eq!(print(&f32::INFINITY), "inf");
    assert_eq!(print(&f32::NEG_INFINITY), "-inf");
}

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
            assert_eq!(f32::parse_mdl(&text).unwrap().to_bits(), bits, "{text}");
        }
    }
    assert_eq!(print(&50.0f32), "50.0");
    assert_eq!(print(&1e20f32), "1.0e20");
    assert_eq!(print(&-0.0f32), "-0.0");
}

#[test]
fn vectors_require_exact_arity_and_separators() {
    assert_eq!(
        <[f32; 3]>::parse_mdl("{ 1.0, -0.0, 2e-2 }").unwrap(),
        [1.0, -0.0, 0.02]
    );
    assert_eq!(<[u32; 0]>::parse_mdl("{}").unwrap(), []);
    for raw in [
        "{ 1, 2 }",
        "{ 1, 2, 3, 4 }",
        "{ 1 2 3 }",
        "{ 1, 2, 3",
        "{ 1, 2, 3 },",
    ] {
        assert!(<[f32; 3]>::parse_mdl(raw).is_err(), "{raw}");
    }
}

#[test]
fn parser_checkpoints_nested_blocks_and_finish_are_explicit() {
    let mut parser = Parser::new("{ Outer { Value 7, } Flag, } 123");
    let mut body = parser.begin_block().unwrap();
    assert_eq!(body.next_field().unwrap().unwrap().name, "Outer");
    let mut child = body.begin_block().unwrap();
    assert_eq!(child.next_field().unwrap().unwrap().name, "Value");
    assert_eq!(child.read_property::<u32>().unwrap(), 7);
    child.finish().unwrap();
    assert_eq!(body.next_field().unwrap().unwrap().name, "Flag");
    body.expect(TokenKind::Comma).unwrap();
    assert!(body.next_field().unwrap().is_none());
    body.finish().unwrap();
    parser.peek().unwrap();
    let checkpoint = parser;
    assert_eq!(parser.read::<u32>().unwrap(), 123);
    parser.finish().unwrap();
    parser = checkpoint;
    assert_eq!(parser.finish().unwrap_err().kind, ReadErrorKind::TrailingInput);
    assert_eq!(parser.read::<u32>().unwrap(), 123);
    let mut parser = Parser::new("{ Unread 1, }");
    assert!(parser.begin_block().unwrap().finish().is_err());
    let mut parser = Parser::new("@");
    let first = parser.peek().unwrap_err();
    assert_eq!(parser.next_token().unwrap_err(), first);
}

#[test]
fn texture_and_sequence_roundtrip_through_mdx() {
    let source = r#"Bitmap { WrapHeight, ReplaceableId 2, Image "Textures\Unit.blp", WrapWidth, }"#;
    let texture = Texture::parse_mdl(source).unwrap();
    assert_eq!(texture.path(), "Textures\\Unit.blp");
    assert_eq!(texture.flags().bits(), 3);
    assert_eq!(texture.replaceable_id(), 2);
    let canonical = print(&texture);
    assert_eq!(canonical, "Bitmap {\n\tImage \"Textures\\Unit.blp\",\n\tReplaceableId 2,\n\tWrapWidth,\n\tWrapHeight,\n}\n");
    let bytes = texture.encode().unwrap();
    assert_eq!(
        Texture::parse_mdl(&print(&Texture::decode(&bytes).unwrap()))
            .unwrap()
            .encode()
            .unwrap(),
        bytes
    );
    assert_eq!(
        Texture::parse_mdl("Bitmap {}").unwrap(),
        Texture::new("").unwrap()
    );

    let source = r#"Anim "Walk" { SyncPoint 1, BoundsRadius 85.0, MaximumExtent { 55.0, 55.0, 105.0 },
        MinimumExtent { -55.0, -55.0, 0.0 }, Rarity -0.0, MoveSpeed 270.0, NonLooping, Interval { 3334, 6667 }, }"#;
    let sequence = Sequence::parse_mdl(source).unwrap();
    assert_eq!(sequence.interval(), [3334, 6667]);
    assert_eq!(sequence.sync_point(), 1);
    assert!(sequence.flags().non_looping());
    let bytes = sequence.encode().unwrap();
    assert_eq!(
        Sequence::parse_mdl(&print(&Sequence::decode(&bytes).unwrap()))
            .unwrap()
            .encode()
            .unwrap(),
        bytes
    );
    assert!(print(&sequence).contains("Rarity -0.0,"));
    assert_eq!(
        Sequence::parse_mdl("Anim \"Stand\" { Interval { 0, 1000 }, }").unwrap(),
        Sequence::new("Stand", [0, 1000]).unwrap()
    );
}

#[test]
fn record_readers_reject_duplicates_unknown_fields_missing_values_and_trailing_input() {
    assert_eq!(
        Texture::parse_mdl("Bitmap { Image \"a\", Image \"b\", }")
            .unwrap_err()
            .kind,
        ReadErrorKind::DuplicateField
    );
    assert_eq!(
        Texture::parse_mdl("Bitmap { WrapWidth, WrapWidth, }")
            .unwrap_err()
            .kind,
        ReadErrorKind::DuplicateField
    );
    assert_eq!(
        Sequence::parse_mdl("Anim \"A\" {}").unwrap_err().kind,
        ReadErrorKind::MissingField("Interval")
    );
    assert_eq!(
        Sequence::parse_mdl("Anim \"A\" { Interval { 0, 1 }, SyncPoint 1, SyncPoint 2, }")
            .unwrap_err()
            .kind,
        ReadErrorKind::DuplicateField
    );
    for source in [
        "Bitmap { Image \"a\" }",
        "Bitmap { WrapWidth 1, }",
        "Bitmap {",
        "Bitmap {} Bitmap {}",
        "Bitmap { BlendColors, }",
    ] {
        assert!(Texture::parse_mdl(source).is_err(), "{source}");
    }
}

#[test]
fn counted_lists_stream_and_validate_on_exhaustion_or_finish() {
    let mut parser = Parser::new("GlobalSequences 2 { Duration 1000, Duration 2500, } 99");
    parser.expect_ident("GlobalSequences").unwrap();
    let mut list = parser.counted::<GlobalSequence>().unwrap();
    assert_eq!(list.declared_count(), 2);
    assert_eq!(list.next().unwrap().unwrap(), GlobalSequence(1000));
    list.finish().unwrap();
    assert_eq!(parser.read::<u32>().unwrap(), 99);
    let mut parser = Parser::new("2 { { 0.0, 1.0, 2.0 }, { 3.0, 4.0, 5.0 }, }");
    let values = parser
        .counted::<PivotPoint>()
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        values,
        vec![PivotPoint([0.0, 1.0, 2.0]), PivotPoint([3.0, 4.0, 5.0])]
    );
    parser.finish().unwrap();
    assert_eq!(print(&GlobalSequence(1000)), "Duration 1000,\n");
    assert_eq!(print(&values[0]), "{ 0.0, 1.0, 2.0 },\n");
    let mut writer = MdlWriter::new(Vec::new());
    writer.counted("PivotPoints", values.iter()).unwrap();
    let text = String::from_utf8(writer.finish().unwrap()).unwrap();
    let mut parser = Parser::new(&text);
    parser.expect_ident("PivotPoints").unwrap();
    assert_eq!(
        parser
            .counted::<PivotPoint>()
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        values
    );
    for (source, expected, actual) in [("2 { Duration 1, }", 2, 1), ("0 { Duration 1, }", 0, 1)] {
        let mut parser = Parser::new(source);
        let mut list = parser.counted::<GlobalSequence>().unwrap();
        let error = loop {
            if let Some(Err(error)) = list.next() {
                break error;
            }
        };
        assert_eq!(error.kind, ReadErrorKind::CountMismatch { expected, actual });
        assert!(list.next().is_none());
        assert_eq!(list.finish().unwrap_err(), error);
    }
    let mut parser = Parser::new("0 {}");
    parser.counted::<PivotPoint>().unwrap().finish().unwrap();
    parser.finish().unwrap();
    let mut parser = Parser::new("1 { Duration 1,");
    assert!(parser
        .counted::<GlobalSequence>()
        .unwrap()
        .finish()
        .is_err());
}

#[test]
fn counted_lists_reject_item_readers_that_make_no_progress() {
    struct Empty;
    impl MdlRead for Empty {
        fn read_mdl(_: &mut Parser<'_>) -> Result<Self, ReadError> {
            Ok(Self)
        }
    }
    let mut parser = Parser::new("1 { Thing }");
    assert_eq!(
        parser
            .counted::<Empty>()
            .unwrap()
            .finish()
            .unwrap_err()
            .kind,
        ReadErrorKind::NoProgress
    );
}

#[test]
fn printing_rejects_unrepresentable_binary_fields() {
    let mut texture = Texture::new("a").unwrap();
    texture.set_flags(TextureFlags(4));
    assert!(matches!(
        MdlWriter::new(io::sink()).write(&texture),
        Err(WriteError::Unsupported(_))
    ));
    let mut sequence = Sequence::new("a", [0, 1]).unwrap();
    sequence.set_flags(SequenceFlags(2));
    assert!(matches!(
        MdlWriter::new(io::sink()).write(&sequence),
        Err(WriteError::Unsupported(_))
    ));
    for bytes in [[b'a', 0, 1, 0], [0xff, 0, 0, 0], [b'a'; 4]] {
        assert!(matches!(
            MdlWriter::new(io::sink()).write(&FixedText::from_bytes(bytes)),
            Err(WriteError::Unsupported(_))
        ));
    }
    let mut bytes = Texture::new("a").unwrap().encode().unwrap();
    bytes[260] = 1;
    assert!(matches!(
        MdlWriter::new(io::sink()).write(&Texture::decode(&bytes).unwrap()),
        Err(WriteError::Unsupported(_))
    ));
}

#[test]
fn writer_checks_balance_identifiers_and_io_errors() {
    let mut writer = MdlWriter::new(io::sink());
    assert!(matches!(
        writer.end_block(),
        Err(WriteError::UnbalancedBlocks)
    ));
    writer.begin_block("Outer").unwrap();
    assert!(matches!(writer.finish(), Err(WriteError::UnbalancedBlocks)));
    assert!(matches!(
        MdlWriter::new(io::sink()).flag("Bad Name"),
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
        matches!(MdlWriter::new(Failing).write(&1u32), Err(WriteError::Io(error)) if error.kind() == io::ErrorKind::BrokenPipe)
    );
}
