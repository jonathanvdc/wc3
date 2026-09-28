use wc3::model::animation::{GeosetTrack, Sequence};
use wc3::model::mdl;
use wc3::model::mdl::{Read as _, Write as _};

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(fields)]
struct Common<T> {
    #[mdl(header)]
    id: u32,
    #[mdl(property = "Value")]
    value: T,
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(fields)]
struct Position {
    #[mdl(property = "Position", default)]
    position: [f32; 3],
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Child")]
struct Child {
    #[mdl(property = "Id")]
    id: u32,
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(entry)]
struct Frame(i32);

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Record", write_order(target, common, children, frames))]
pub struct Record {
    #[mdl(flatten)]
    common: Common<u32>,
    #[mdl(block = "Target")]
    target: Position,
    #[mdl(repeated = "Child")]
    children: Vec<Child>,
    #[mdl(counted = "Frames")]
    frames: Vec<Frame>,
}

#[test]
fn interleaved_flattened_fields_nested_blocks_and_both_collection_forms() {
    let source = "Record 7 { Child { Id 1, } Target { Position { 1, 2, 3 }, } Frames 2 { -10, 40, } Value 9, Child { Id 2, } }";
    let value = Record::decode_mdl(source).unwrap();
    assert_eq!(value.common, Common { id: 7, value: 9 });
    assert_eq!(value.children, vec![Child { id: 1 }, Child { id: 2 }]);
    assert_eq!(value.frames, vec![Frame(-10), Frame(40)]);
    let text = value.encode_mdl().unwrap();
    assert!(text.starts_with("Record 7 {\n\tTarget {\n"));
    assert!(text.contains("\tFrames 2 {\n\t\t-10,\n\t\t40,\n\t}\n"));
    assert!(text.find("Target").unwrap() < text.find("Value").unwrap());
    assert_eq!(Record::decode_mdl(&text).unwrap(), value);
}

#[test]
fn empty_collections_and_nested_defaults() {
    let value = Record::decode_mdl("Record 3 { Value 0, Target {} Frames 0 {} }").unwrap();
    assert!(value.children.is_empty());
    assert!(value.frames.is_empty());
    assert_eq!(value.target.position, [0.0; 3]);
    assert_eq!(
        Record::decode_mdl(&value.encode_mdl().unwrap()).unwrap(),
        value
    );
}

#[test]
fn required_duplicate_unknown_and_count_errors_keep_their_spans() {
    for (source, kind, needle) in [
        (
            "Record 0 { Value 1, Value 2, }",
            mdl::ReadErrorKind::DuplicateField,
            "Value 2",
        ),
        (
            "Record 0 { Value 1, Target {} Target {} }",
            mdl::ReadErrorKind::DuplicateField,
            "Target {} }",
        ),
        (
            "Record 0 { Value 1, Target { Position { 0, 0, 0 }, Position { 1, 1, 1 }, } }",
            mdl::ReadErrorKind::DuplicateField,
            "Position { 1",
        ),
        (
            "Record 0 { Surprise 0, }",
            mdl::ReadErrorKind::UnknownField,
            "Surprise",
        ),
        (
            "Record 0 { Value 1, Target { Surprise 0, } }",
            mdl::ReadErrorKind::UnknownField,
            "Surprise",
        ),
        (
            "Record 0 { Value 1, Target {} Frames 2 { 1, } }",
            mdl::ReadErrorKind::CountMismatch {
                expected: 2,
                actual: 1,
            },
            "} }",
        ),
        (
            "Record 0 { Value 1, Target {} Frames 0 { 1, } }",
            mdl::ReadErrorKind::CountMismatch {
                expected: 0,
                actual: 1,
            },
            "1, }",
        ),
    ] {
        let error = Record::decode_mdl(source).unwrap_err();
        assert_eq!(error.kind, kind, "{source}");
        assert_eq!(error.span.start, source.find(needle).unwrap(), "{source}");
    }
    let missing = Record::decode_mdl("Record 0 { Target {} Frames 0 {} }").unwrap_err();
    assert_eq!(missing.kind, mdl::ReadErrorKind::MissingField("Value"));
    for source in [
        "Record 0 { Value 1, Target {}, Frames 0 {} }",
        "Record 0 { Value 1, Target {} Frames 1 { 1 } }",
        "Record 0 { Value 1, Target {} Frames 0 {} Child { Id 1 } }",
    ] {
        assert!(Record::decode_mdl(source).is_err(), "{source}");
    }
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Collision")]
struct Collision {
    #[mdl(flatten)]
    first: Position,
    #[mdl(flatten)]
    second: Position,
}

#[test]
fn conflicting_flattened_names_are_rejected_before_body_or_output() {
    assert!(Collision::decode_mdl("Collision {}").is_err());
    let value = Collision {
        first: Position { position: [0.0; 3] },
        second: Position { position: [0.0; 3] },
    };
    let mut output = Vec::new();
    let mut writer = mdl::MdlWriter::new(&mut output);
    assert!(writer.write(&value).is_err());
    assert!(output.is_empty());
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(fields)]
struct Animated {
    #[mdl(animatable = "Alpha", default, track = "GeosetTrack::Alpha")]
    alpha: f32,
    #[mdl(tracks)]
    tracks: Vec<GeosetTrack>,
    #[mdl(property = "Optional", delegate)]
    optional: Option<u32>,
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Animated")]
struct FlattenedAnimated {
    #[mdl(flatten)]
    inner: Animated,
}

#[test]
fn flattened_static_animated_and_delegated_properties_share_dispatch() {
    let fixed =
        FlattenedAnimated::decode_mdl("Animated { static Alpha 0.5, Optional 3, }").unwrap();
    assert_eq!(fixed.inner.alpha, 0.5);
    assert_eq!(fixed.inner.optional, Some(3));
    assert!(fixed.encode_mdl().unwrap().contains("static Alpha 0.5,"));
    let animated = FlattenedAnimated::decode_mdl("Animated { Alpha 0 { Linear, } }").unwrap();
    assert_eq!(animated.inner.tracks.len(), 1);
    assert!(animated.inner.optional.is_none());
    assert!(!animated.encode_mdl().unwrap().contains("static Alpha"));
    assert_eq!(
        FlattenedAnimated::decode_mdl("Animated { static Alpha 1, Alpha 0 { Linear, } }")
            .err()
            .unwrap()
            .kind,
        mdl::ReadErrorKind::DuplicateField
    );
}

#[test]
fn existing_sequence_roundtrips_flattened_extent_in_both_formats() {
    let source = "Anim \"Walk\" { MaximumExtent { 4, 5, 6 }, Interval { 0, 1000 }, BoundsRadius 7, MinimumExtent { 1, 2, 3 }, }";
    let sequence = Sequence::decode_mdl(source).unwrap();
    assert_eq!(sequence.bounds_radius(), 7.0);
    assert_eq!(sequence.minimum_extent(), [1.0, 2.0, 3.0]);
    assert_eq!(sequence.maximum_extent(), [4.0, 5.0, 6.0]);
    let text = sequence.encode_mdl().unwrap();
    assert!(text.find("MinimumExtent").unwrap() < text.find("MaximumExtent").unwrap());
    assert!(text.find("MaximumExtent").unwrap() < text.find("BoundsRadius").unwrap());
    assert_eq!(Sequence::decode_mdl(&text).unwrap(), sequence);
    use wc3::model::mdx::{Read as _, Write as _};
    let bytes = sequence.encode_mdx().unwrap();
    assert_eq!(bytes.len(), 132);
    assert_eq!(Sequence::decode_mdx(&bytes).unwrap(), sequence);
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(fields)]
struct SecondHeader {
    #[mdl(header)]
    second: u32,
    #[mdl(property = "Second", default)]
    value: u32,
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Headers", write_order(second, first))]
struct Headers {
    #[mdl(flatten)]
    first: Common<u32>,
    #[mdl(header)]
    middle: u32,
    #[mdl(flatten)]
    second: SecondHeader,
}

#[test]
fn body_order_keeps_flattened_headers_in_declaration_order() {
    let value = Headers::decode_mdl("Headers 1 2 3 { Value 4, Second 5, }").unwrap();
    let text = value.encode_mdl().unwrap();
    assert!(text.starts_with("Headers 1 2 3 {\n\tSecond 5,"));
    assert_eq!(Headers::decode_mdl(&text).unwrap(), value);
}

thread_local! {
    static OMIT_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
fn omit_zero(value: &u32) -> bool {
    OMIT_CALLS.with(|calls| calls.set(calls.get() + 1));
    *value == 0
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(fields)]
struct Omitted {
    #[mdl(property = "Zero", default, skip_if = "omit_zero")]
    value: u32,
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(fields)]
struct Recursive {
    #[mdl(flatten)]
    inner: Omitted,
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Omitted")]
struct OmittedRecord {
    #[mdl(flatten)]
    recursive: Recursive,
    #[mdl(block = "Nested")]
    nested: Omitted,
}
#[test]
fn recursive_flatten_and_nested_preflight_evaluate_omission_once() {
    let value = OmittedRecord::decode_mdl("Omitted { Nested {} }").unwrap();
    OMIT_CALLS.with(|calls| calls.set(0));
    assert_eq!(
        value.encode_mdl().unwrap(),
        "Omitted {\n\tNested {\n\t}\n}\n"
    );
    assert_eq!(OMIT_CALLS.with(|calls| calls.get()), 2);
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(
    fields,
    validate_read = "Validated::validate_read",
    validate_write = "Validated::validate_write"
)]
struct Validated {
    #[mdl(property = "Id")]
    id: u32,
}
impl Validated {
    fn validate_read(&self, span: mdl::Span) -> Result<(), mdl::ReadError> {
        if self.id == 0 {
            Err(mdl::ReadError::new(
                span,
                mdl::ReadErrorKind::UnsupportedField,
            ))
        } else {
            Ok(())
        }
    }
    fn validate_write(&self) -> Result<(), mdl::WriteError> {
        if self.id == 0 {
            Err(mdl::WriteError::Unsupported("zero id"))
        } else {
            Ok(())
        }
    }
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Validated")]
struct ValidatedRecord {
    #[mdl(block = "Nested")]
    nested: Validated,
}
#[test]
fn nested_validation_receives_record_span_and_runs_before_parent_output() {
    let source = "Validated { Nested { Id 0, } }";
    let error = ValidatedRecord::decode_mdl(source).err().unwrap();
    assert_eq!(
        &source[error.span.start..error.span.end],
        "Nested { Id 0, }"
    );
    let value = ValidatedRecord {
        nested: Validated { id: 0 },
    };
    let mut bytes = Vec::new();
    assert!(mdl::MdlWriter::new(&mut bytes).write(&value).is_err());
    assert!(bytes.is_empty());
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(fields)]
struct LiteralStatic {
    #[mdl(property = "static")]
    value: u32,
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Literal")]
struct Literal {
    #[mdl(flatten)]
    literal: LiteralStatic,
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Reserved")]
struct Reserved {
    #[mdl(flatten)]
    literal: LiteralStatic,
    #[mdl(flatten)]
    animated: Animated,
}
#[test]
fn static_keyword_is_reserved_only_when_static_properties_exist() {
    let value = Literal::decode_mdl("Literal { static 5, }").unwrap();
    assert_eq!(value.literal.value, 5);
    assert!(value.encode_mdl().unwrap().contains("static 5,"));
    assert!(Reserved::decode_mdl("Reserved {}").is_err());
}

struct NoProgress;
impl mdl::Read for NoProgress {
    fn read_mdl(_: &mut mdl::Parser<'_>) -> Result<Self, mdl::ReadError> {
        Ok(Self)
    }
}
#[derive(mdl::Read)]
#[mdl(block = "NoProgress")]
struct NoProgressRecord {
    #[mdl(repeated = "Child")]
    _children: Vec<NoProgress>,
}
#[test]
fn repeated_records_reject_nonadvancing_codecs() {
    let error = NoProgressRecord::decode_mdl("NoProgress { Child {} }")
        .err()
        .unwrap();
    assert_eq!(error.kind, mdl::ReadErrorKind::NoProgress);
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Defaults", default)]
struct Defaults {
    #[mdl(flatten)]
    position: Position,
    #[mdl(block = "Target", required)]
    target: Position,
    #[mdl(counted = "Frames", required)]
    frames: Vec<Frame>,
}
impl Default for Defaults {
    fn default() -> Self {
        Self {
            position: Position { position: [9.0; 3] },
            target: Position { position: [8.0; 3] },
            frames: vec![Frame(10)],
        }
    }
}
#[test]
fn required_structural_fields_override_container_defaults_and_flatten_keeps_its_own() {
    let value = Defaults::decode_mdl("Defaults { Target {} Frames 0 {} }").unwrap();
    assert_eq!(value.position.position, [0.0; 3]);
    assert_eq!(value.target.position, [0.0; 3]);
    assert!(value.frames.is_empty());
    let target = Defaults::decode_mdl("Defaults { Frames 0 {} }")
        .err()
        .unwrap();
    assert_eq!(target.kind, mdl::ReadErrorKind::MissingField("Target"));
    let frames = Defaults::decode_mdl("Defaults { Target {} }")
        .err()
        .unwrap();
    assert_eq!(frames.kind, mdl::ReadErrorKind::MissingField("Frames"));
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(
    block = "Unique",
    after_read = "finish_unique",
    validate_read = "check_unique"
)]
struct Unique {
    #[mdl(repeated(Child), unique_by = "child_id")]
    children: Vec<Child>,
    #[mdl(skip, default)]
    reconstructed: bool,
}
fn child_id(child: &Child) -> u32 {
    child.id
}
fn finish_unique(value: &mut Unique, _: mdl::Span) -> Result<(), mdl::ReadError> {
    value.reconstructed = true;
    Ok(())
}
fn check_unique(value: &Unique, span: mdl::Span) -> Result<(), mdl::ReadError> {
    if !value.reconstructed {
        return Err(mdl::ReadError::new(
            span,
            mdl::ReadErrorKind::UnsupportedField,
        ));
    }
    Ok(())
}
#[test]
fn unique_collection_keys_and_reconstruction_before_validation() {
    let value = Unique::decode_mdl("Unique { Child { Id 1, } Child { Id 2, } }").unwrap();
    assert!(value.reconstructed);
    assert_eq!(
        Unique::decode_mdl(&value.encode_mdl().unwrap()).unwrap(),
        value
    );
    let source = "Unique { Child { Id 1, } Child { Id 1, } }";
    let error = Unique::decode_mdl(source).unwrap_err();
    assert_eq!(error.kind, mdl::ReadErrorKind::DuplicateField);
    assert_eq!(error.span.start, source.rfind("Child").unwrap());
    let invalid = Unique {
        children: vec![Child { id: 1 }, Child { id: 1 }],
        reconstructed: true,
    };
    assert!(invalid.encode_mdl().is_err());
}

use wc3::model::scene::LightTrack;

#[derive(Debug, mdl::Read, mdl::Write)]
#[mdl(block = "TrackOnly")]
struct TrackOnly {
    #[mdl(
        animatable = "Visibility",
        track = "LightTrack::Visibility",
        animated_only,
        default
    )]
    visibility: f32,
    #[mdl(tracks)]
    tracks: Vec<LightTrack>,
}
#[test]
fn animated_only_channel_has_no_static_spelling_or_output() {
    let empty = TrackOnly::decode_mdl("TrackOnly {}").unwrap();
    assert_eq!(empty.encode_mdl().unwrap(), "TrackOnly {\n}\n");
    let value = TrackOnly::decode_mdl("TrackOnly { Visibility 0 { Linear, } }").unwrap();
    let text = value.encode_mdl().unwrap();
    assert!(!text.contains("static"));
    assert_eq!(TrackOnly::decode_mdl(&text).unwrap().tracks.len(), 1);
    assert!(TrackOnly::decode_mdl("TrackOnly { static Visibility 0.0, }").is_err());
    let duplicate =
        TrackOnly::decode_mdl("TrackOnly { Visibility 0 { Linear, } Visibility 0 { Linear, } }")
            .unwrap_err();
    assert_eq!(duplicate.kind, mdl::ReadErrorKind::DuplicateField);
    let hidden = TrackOnly {
        visibility: -0.0,
        tracks: vec![],
    };
    assert!(hidden.encode_mdl().is_err());
}
#[test]
fn counted_header_does_not_contribute_to_the_item_count() {
    let mut parser = mdl::Parser::new("2 { Id 7, -10, 20, }");
    let (header, items) = parser
        .counted_with_header::<Frame, _>(|parser| {
            parser.expect_ident("Id")?;
            parser.read_property::<u32>()
        })
        .unwrap();
    assert_eq!(header, 7);
    assert_eq!(
        items.collect::<Result<Vec<_>, _>>().unwrap(),
        vec![Frame(-10), Frame(20)]
    );
    parser.finish().unwrap();
    let mut parser = mdl::Parser::new("1 { Id 7, }");
    let (_, items) = parser
        .counted_with_header::<Frame, _>(|parser| {
            parser.expect_ident("Id")?;
            parser.read_property::<u32>()
        })
        .unwrap();
    assert_eq!(
        items.finish().unwrap_err().kind,
        mdl::ReadErrorKind::CountMismatch {
            expected: 1,
            actual: 0
        }
    );
}
