use super::*;

#[test]
fn typed_numbers_check_full_spelling_ranges_and_nonfinite_literals() {
    assert_eq!(u32::decode_mdl("4294967295").unwrap(), u32::MAX);
    assert_eq!(i32::decode_mdl("-2147483648").unwrap(), i32::MIN);
    assert_eq!(i32::decode_mdl("-3600").unwrap(), -3600);
    assert_eq!(f32::decode_mdl("1.5e-08").unwrap(), 1.5e-8);
    for raw in ["4294967296", "-1", "1.5", "1e2", "1foo", "++1"] {
        assert!(u32::decode_mdl(raw).is_err(), "{raw}");
    }
    for raw in ["1e", "1e1000", "Infinity", "1..2", "+"] {
        assert!(f32::decode_mdl(raw).is_err(), "{raw}");
    }
    for raw in ["nan", "NaN", "NAN"] {
        assert!(f32::decode_mdl(raw).unwrap().is_nan());
    }
    for raw in ["inf", "INF", "+Inf"] {
        assert_eq!(f32::decode_mdl(raw).unwrap(), f32::INFINITY);
    }
    assert_eq!(f32::decode_mdl("-INF").unwrap(), f32::NEG_INFINITY);
    assert_eq!(print(&f32::NAN), "nan");
    assert_eq!(print(&f32::INFINITY), "inf");
    assert_eq!(print(&f32::NEG_INFINITY), "-inf");
}

#[test]
fn vectors_require_exact_arity_and_separators() {
    assert_eq!(
        <[f32; 3]>::decode_mdl("{ 1.0, -0.0, 2e-2 }").unwrap(),
        [1.0, -0.0, 0.02]
    );
    assert_eq!(<[u32; 0]>::decode_mdl("{}").unwrap(), []);
    for raw in [
        "{ 1, 2 }",
        "{ 1, 2, 3, 4 }",
        "{ 1 2 3 }",
        "{ 1, 2, 3",
        "{ 1, 2, 3 },",
    ] {
        assert!(<[f32; 3]>::decode_mdl(raw).is_err(), "{raw}");
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
    assert_eq!(
        parser.finish().unwrap_err().kind,
        ReadErrorKind::TrailingInput
    );
    assert_eq!(parser.read::<u32>().unwrap(), 123);
    let mut parser = Parser::new("{ Unread 1, }");
    assert!(parser.begin_block().unwrap().finish().is_err());
    let mut parser = Parser::new("@");
    let first = parser.peek().unwrap_err();
    assert_eq!(parser.next_token().unwrap_err(), first);
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
    let mut writer = Writer::new(Vec::new());
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
        assert_eq!(
            error.kind,
            ReadErrorKind::CountMismatch { expected, actual }
        );
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
    impl mdl::Read for Empty {
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
