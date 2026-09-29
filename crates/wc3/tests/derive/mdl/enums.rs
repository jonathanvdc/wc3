use std::io::Write as IoWrite;
use wc3::model::animation::Interpolation;
use wc3::model::animation::Track;
use wc3::model::mdl;
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::IoError;
use wc3::model::Vec3;

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(value)]
enum Filter {
    None,
    #[mdl(name = "Blend")]
    AlphaBlend,
    Additive,
}
#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Layer")]
struct Layer {
    #[mdl(property = "FilterMode")]
    filter: Filter,
}
#[test]
fn scalar_enums_leave_property_punctuation_to_the_record() {
    for (text, value) in [
        ("None", Filter::None),
        ("Blend", Filter::AlphaBlend),
        ("Additive", Filter::Additive),
    ] {
        assert_eq!(Filter::decode_mdl(text).unwrap(), value);
        assert_eq!(value.encode_mdl().unwrap(), text);
        let layer = Layer::decode_mdl(&format!("Layer {{ FilterMode {text}, }}")).unwrap();
        assert_eq!(layer.filter, value);
        assert_eq!(
            Layer::decode_mdl(&layer.encode_mdl().unwrap()).unwrap(),
            layer
        );
    }
    assert_eq!(
        Filter::decode_mdl("Unknown").unwrap_err().kind,
        mdl::ReadErrorKind::UnknownField
    );
    assert_eq!(
        Filter::decode_mdl("None,").unwrap_err().kind,
        mdl::ReadErrorKind::TrailingInput
    );
    assert!(Filter::decode_mdl("\"None\"").is_err());
    assert!(Filter::decode_mdl("1").is_err());
    assert!(Filter::decode_mdl("").is_err());
    assert!(Layer::decode_mdl("Layer { FilterMode None }").is_err());
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(fields)]
struct Position {
    #[mdl(property = "Position")]
    value: [f32; 3],
}
#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Child")]
struct Child {
    #[mdl(property = "Id")]
    id: u32,
}
const CHILD_NAME: &str = "Child";
#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(tagged)]
enum Record<T> {
    #[mdl(flag = "Ready")]
    Ready,
    #[mdl(property = "Duration")]
    Duration(T),
    #[mdl(block = "Target")]
    Target(Position),
    #[mdl(name = CHILD_NAME, delegate)]
    Child(Child),
}
#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Records")]
struct Records {
    #[mdl(counted = "Items")]
    items: Vec<Record<u32>>,
}
#[test]
fn tagged_enums_frame_flags_properties_blocks_and_delegate_complete_records() {
    let value = Records::decode_mdl("Records { Items 4 { Ready, Duration 20, Target { Position { 1, 2, 3 }, } Child { Id 4, } } }").unwrap();
    assert_eq!(
        value.items,
        vec![
            Record::Ready,
            Record::Duration(20),
            Record::Target(Position {
                value: [1.0, 2.0, 3.0]
            }),
            Record::Child(Child { id: 4 })
        ]
    );
    assert_eq!(
        Records::decode_mdl(&value.encode_mdl().unwrap()).unwrap(),
        value
    );
    let error = Record::<u32>::decode_mdl("Child { Surprise 1, }").unwrap_err();
    assert_eq!(error.kind, mdl::ReadErrorKind::UnknownField);
    assert_eq!(error.span, mdl::Span::new(8, 16));
    for input in [
        "Ready",
        "Duration 20",
        "Target { Position { 1, 2, 3 } }",
        "Child { Id 4 }",
        "Target { Position { 1, 2, 3 }, },",
    ] {
        assert!(Record::<u32>::decode_mdl(input).is_err(), "{input}");
    }
}

#[derive(mdl::Read)]
#[mdl(tagged)]
enum ReadOnly<T> {
    #[mdl(property = "Value")]
    Value(T),
}
#[derive(mdl::Write)]
#[mdl(tagged)]
enum WriteOnly<T> {
    #[mdl(property = "Value")]
    Value(T),
}
struct Input(u32);
impl mdl::Read for Input {
    fn read_mdl(parser: &mut mdl::Parser<'_>) -> Result<Self, mdl::ReadError> {
        parser.read().map(Self)
    }
}
struct Output(u32);
impl mdl::Write for Output {
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut mdl::Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        writer.write(&self.0)
    }
}
#[test]
fn generic_payload_bounds_are_format_and_direction_specific() {
    let ReadOnly::Value(Input(value)) = ReadOnly::<Input>::decode_mdl("Value 5,").unwrap();
    assert_eq!(value, 5);
    assert_eq!(
        WriteOnly::Value(Output(5)).encode_mdl().unwrap(),
        "Value 5,\n"
    );
}

const FIRST: &str = "Same";
const SECOND: &str = "Same";
#[derive(mdl::Read, mdl::Write)]
#[mdl(value)]
enum Duplicate {
    #[mdl(name = FIRST)]
    First,
    #[mdl(name = SECOND)]
    Second,
}
const INVALID: &str = "bad name";
#[derive(mdl::Read, mdl::Write)]
#[mdl(value)]
enum Invalid {
    #[mdl(name = INVALID)]
    Value,
}
#[test]
fn constant_names_are_checked_for_duplicates_and_valid_identifiers() {
    assert!(Duplicate::decode_mdl("Same").is_err());
    assert!(Invalid::decode_mdl("bad").is_err());
    let mut bytes = Vec::new();
    assert!(mdl::Writer::new(&mut bytes)
        .write(&Duplicate::First)
        .is_err());
    assert!(bytes.is_empty());
    assert!(mdl::Writer::new(&mut bytes).write(&Invalid::Value).is_err());
    assert!(bytes.is_empty());
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(
    tagged,
    validate_read = "Checked::check_read",
    validate_write = "Checked::check_write"
)]
enum Checked {
    #[mdl(property = "Id")]
    Id(u32),
}
impl Checked {
    fn check_read(&self, span: mdl::Span) -> Result<(), mdl::ReadError> {
        if matches!(self, Self::Id(0)) {
            Err(mdl::ReadError::new(
                span,
                mdl::ReadErrorKind::UnsupportedField,
            ))
        } else {
            Ok(())
        }
    }
    fn check_write(&self) -> Result<(), mdl::WriteError> {
        if matches!(self, Self::Id(0)) {
            Err(mdl::WriteError::Unrepresentable { field: "zero id" })
        } else {
            Ok(())
        }
    }
}
#[test]
fn enum_validation_uses_complete_span_and_precedes_output() {
    let error = Checked::decode_mdl("  Id 0,").unwrap_err();
    assert_eq!(error.span, mdl::Span::new(2, 7));
    let mut bytes = Vec::new();
    assert!(mdl::Writer::new(&mut bytes).write(&Checked::Id(0)).is_err());
    assert!(bytes.is_empty());
}

#[test]
fn existing_interpolation_and_track_group_enums_roundtrip() {
    for (name, mode) in [
        ("DontInterp", Interpolation::Step),
        ("Linear", Interpolation::Linear),
        ("Hermite", Interpolation::Hermite),
        ("Bezier", Interpolation::Bezier),
    ] {
        assert_eq!(Interpolation::decode_mdl(name).unwrap(), mode);
        assert_eq!(mode.encode_mdl().unwrap(), name);
    }
    let track = Track::<Vec3>::decode_mdl("Track 0 { Linear, }").unwrap();
    assert_eq!(
        Track::<Vec3>::decode_mdl(&track.encode_mdl().unwrap()).unwrap(),
        track
    );
}

#[allow(dead_code)]
mod hygiene {
    use wc3::model::mdl;
    use wc3::model::mdl::{Read as _, Write as _};
    struct Some;
    struct Ok;
    struct Err;
    type Result = u32;
    #[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
    #[mdl(value)]
    enum Keyword {
        Ready,
    }
    #[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
    #[mdl(tagged)]
    enum Generic<__Wc3MdlSink> {
        #[mdl(property = "Value")]
        Value(__Wc3MdlSink),
    }
    #[test]
    fn generated_code_qualifies_runtime_constructors_and_avoids_sink_collisions() {
        assert_eq!(
            Keyword::decode_mdl("Ready").unwrap().encode_mdl().unwrap(),
            "Ready"
        );
        assert_eq!(
            Generic::<u32>::decode_mdl("Value 9,")
                .unwrap()
                .encode_mdl()
                .unwrap(),
            "Value 9,\n"
        );
    }
}

#[derive(Debug, Default, PartialEq, mdl::Read, mdl::Write)]
#[mdl(choice, default)]
enum OptionalChoice {
    #[default]
    First,
    #[mdl(name = "Other")]
    Second,
    #[mdl(unknown)]
    Unknown(u32),
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(choice)]
enum RequiredChoice {
    Ready,
    Waiting,
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Choices")]
struct Choices {
    #[mdl(flatten)]
    optional: OptionalChoice,
    #[mdl(property = "Id")]
    id: u32,
    #[mdl(flatten)]
    required: RequiredChoice,
}

#[test]
fn flattened_choices_interleave_with_properties_and_enforce_one_variant() {
    let value = Choices::decode_mdl("Choices { Ready, Id 4, }").unwrap();
    assert_eq!(value.optional, OptionalChoice::First);
    assert_eq!(value.required, RequiredChoice::Ready);
    assert_eq!(
        value.encode_mdl().unwrap(),
        "Choices {\n\tFirst,\n\tId 4,\n\tReady,\n}\n"
    );
    let other = Choices::decode_mdl("Choices { Waiting, Other, Id 4, }").unwrap();
    assert_eq!(other.optional, OptionalChoice::Second);
    assert_eq!(
        Choices::decode_mdl(&other.encode_mdl().unwrap()).unwrap(),
        other
    );
    assert!(Choices::decode_mdl("Choices { Id 4, }").is_err());
    for (source, second) in [
        ("Choices { First, Id 4, Other, Ready, }", "Other"),
        ("Choices { Ready, Id 4, Waiting, }", "Waiting"),
        ("Choices { Ready, Id 4, Ready, }", "Ready"),
    ] {
        let error = Choices::decode_mdl(source).unwrap_err();
        assert_eq!(error.kind, mdl::ReadErrorKind::DuplicateField);
        assert_eq!(&source[error.span.start..error.span.end], second);
    }
    for source in [
        "Choices { Id 4, Unknown, Ready, }",
        "Choices { Id 4, static Ready, }",
        "Choices { Id 4, Ready }",
    ] {
        assert!(Choices::decode_mdl(source).is_err());
    }
}

#[test]
fn unknown_choices_fail_before_record_output() {
    let value = Choices {
        optional: OptionalChoice::Unknown(99),
        id: 4,
        required: RequiredChoice::Ready,
    };
    let mut output = Vec::new();
    assert!(mdl::Writer::new(&mut output).write(&value).is_err());
    assert!(output.is_empty());
}
