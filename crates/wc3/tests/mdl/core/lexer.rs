use super::*;

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
        assert_eq!(
            lexer.next().unwrap().unwrap_err(),
            ReadError::new(span, kind)
        );
        assert!(lexer.next().is_none());
    }
    assert_eq!(
        ReadError::new(Span::new(6, 6), ReadErrorKind::UnknownField).line_column("é\r\nx\ry"),
        (3, 1)
    );
    let source = "// é\r\nBitmap {\n Image \"ok\",\n Bogus 3,\n}";
    let error = Texture::decode_mdl(source).unwrap_err();
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
    assert_eq!(print(&FixedText::<32>::decode_mdl(source).unwrap()), source);
    assert_eq!(FixedText::<4>::decode_mdl("\"雪\"").unwrap().text(), "雪");
    assert!(matches!(
        FixedText::<3>::decode_mdl("\"雪\"").unwrap_err().kind,
        ReadErrorKind::InvalidString { max_bytes: 2 }
    ));
    assert!(FixedText::<8>::decode_mdl("\"a\0b\"").is_err());
    assert!(matches!(
        print_result("a\"b"),
        Err(WriteError::InvalidString)
    ));
    assert!(matches!(
        print_result("a\0b"),
        Err(WriteError::InvalidString)
    ));
}
