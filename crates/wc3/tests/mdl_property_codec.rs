use std::io::Write as IoWrite;
use wc3::model::mdl;
use wc3::model::mdl::{Field, MdlWriter, Parser, Read as _, ReadErrorKind, Span, Write as _};

trait Layout {
    type Extra;
}

#[derive(Debug, PartialEq)]
struct Old;
impl Layout for Old {
    type Extra = Absent;
}

#[derive(Debug, PartialEq)]
struct New;
impl Layout for New {
    type Extra = Present;
}

// These field types deliberately implement neither mdl::Read/Write nor Default.
// The generic derive should require only the delegated property interfaces.
#[derive(Debug, PartialEq)]
struct Present(f32);
impl mdl::ReadProperty for Present {
    fn read_mdl_property(
        parser: &mut Parser<'_>,
        _field: Field<'_>,
    ) -> Result<Self, mdl::ReadError> {
        parser.read_property().map(Self)
    }
    // The default missing-property policy makes this field required.
}
impl mdl::WriteProperty for Present {
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        writer: &mut MdlWriter<W>,
    ) -> Result<(), mdl::WriteError> {
        writer.property(name, &self.0)
    }
}

#[derive(Debug, PartialEq)]
struct Absent {
    reserved: u32,
}
impl mdl::ReadProperty for Absent {
    fn read_mdl_property(
        _parser: &mut Parser<'_>,
        field: Field<'_>,
    ) -> Result<Self, mdl::ReadError> {
        Err(mdl::ReadError::new(
            field.span,
            ReadErrorKind::UnsupportedField,
        ))
    }
    fn missing_mdl_property(_name: &'static str, _span: Span) -> Result<Self, mdl::ReadError> {
        Ok(Self { reserved: 0 })
    }
}
impl mdl::WriteProperty for Absent {
    fn validate_mdl_property(&self, name: &'static str) -> Result<(), mdl::WriteError> {
        if self.reserved != 0 {
            return Err(mdl::WriteError::Unsupported(name));
        }
        Ok(())
    }
    fn write_mdl_property<W: IoWrite>(
        &self,
        name: &'static str,
        _writer: &mut MdlWriter<W>,
    ) -> Result<(), mdl::WriteError> {
        self.validate_mdl_property(name)
    }
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Artificial", write_order(tail, extra))]
struct Record<V: Layout> {
    #[mdl(delegate, property = "Extra")]
    extra: V::Extra,
    #[mdl(property = "Tail")]
    tail: u32,
}

#[test]
fn generic_record_uses_field_type_policies_without_version_checks() {
    let old = Record::<Old>::decode_mdl("Artificial { Tail 4, }").unwrap();
    assert_eq!(old.extra, Absent { reserved: 0 });
    assert_eq!(old.encode_mdl().unwrap(), "Artificial {\n\tTail 4,\n}\n");
    let source = "Artificial { Extra 2.5, Tail 4, }";
    let new = Record::<New>::decode_mdl(source).unwrap();
    assert_eq!(new.extra, Present(2.5));
    let text = new.encode_mdl().unwrap();
    assert_eq!(text, "Artificial {\n\tTail 4,\n\tExtra 2.5,\n}\n");
    assert_eq!(Record::<New>::decode_mdl(&text).unwrap(), new);
    let error = Record::<Old>::decode_mdl(source).unwrap_err();
    assert_eq!(error.kind, ReadErrorKind::UnsupportedField);
    assert_eq!(&source[error.span.start..error.span.end], "Extra");
    // Presence is rejected by the absent field type before reading any payload.
    assert_eq!(
        Record::<Old>::decode_mdl("Artificial { Extra }")
            .unwrap_err()
            .kind,
        ReadErrorKind::UnsupportedField
    );
    let source = "Artificial { Tail 4, }";
    let error = Record::<New>::decode_mdl(source).unwrap_err();
    assert_eq!(error.kind, ReadErrorKind::MissingField("Extra"));
    assert_eq!(&source[error.span.start..error.span.end], "}");
}

#[test]
fn record_keeps_duplicate_unknown_field_and_payload_validation() {
    for (source, expected) in [
        (
            "Artificial { Extra 1, Extra 2, Tail 0, }",
            ReadErrorKind::DuplicateField,
        ),
        (
            "Artificial { Extra 1, Other 2, Tail 0, }",
            ReadErrorKind::UnknownField,
        ),
        (
            "Artificial { Extra 1 Tail 0, }",
            ReadErrorKind::Expected("','"),
        ),
    ] {
        let error = Record::<New>::decode_mdl(source).unwrap_err();
        assert_eq!(error.kind, expected);
    }
    assert!(Record::<New>::decode_mdl("Artificial { Extra 1, Tail 0, } 42").is_err());
}

#[test]
fn delegated_preflight_rejects_omission_that_would_discard_data() {
    let record = Record::<Old> {
        extra: Absent { reserved: 7 },
        tail: 0,
    };
    let mut bytes = Vec::new();
    let mut writer = MdlWriter::new(&mut bytes);
    assert!(matches!(
        writer.write(&record),
        Err(mdl::WriteError::Unsupported("Extra"))
    ));
    assert!(bytes.is_empty());
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Optional")]
struct Optional {
    #[mdl(delegate)]
    #[mdl(property = "Value")]
    value: Option<f32>,
}

#[test]
fn ordinary_optional_properties_use_the_same_delegation() {
    let absent = Optional::decode_mdl("Optional {}").unwrap();
    assert_eq!(absent.value, None);
    assert_eq!(absent.encode_mdl().unwrap(), "Optional {\n}\n");
    let present = Optional::decode_mdl("Optional { Value -0.0, }").unwrap();
    assert_eq!(present.value.unwrap().to_bits(), (-0.0f32).to_bits());
    assert_eq!(
        present.encode_mdl().unwrap(),
        "Optional {\n\tValue -0.0,\n}\n"
    );
    assert_eq!(
        Optional::decode_mdl("Optional { Value 1, Value 2, }")
            .unwrap_err()
            .kind,
        ReadErrorKind::DuplicateField
    );
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Defaulted", default)]
struct Defaulted {
    #[mdl(property = "Value", delegate)]
    value: Option<u32>,
    #[mdl(property = "Tail")]
    tail: u32,
}
impl Default for Defaulted {
    fn default() -> Self {
        Self {
            value: Some(99),
            tail: 7,
        }
    }
}

#[test]
fn delegated_missing_policy_is_independent_of_record_defaults() {
    let value = Defaulted::decode_mdl("Defaulted {}").unwrap();
    assert_eq!(value.value, None);
    assert_eq!(value.tail, 7);
    assert_eq!(value.encode_mdl().unwrap(), "Defaulted {\n\tTail 7,\n}\n");
}
