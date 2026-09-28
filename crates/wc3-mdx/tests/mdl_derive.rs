use std::io::Write;
use std::marker::PhantomData;
use wc3_mdx::mdl::{MdlWriter, Parser, ReadError, ReadErrorKind, Span, WriteError};
use wc3_mdx::scene::ModelInfo;
use wc3_mdx::{FixedText, MdlFlags, MdlRead, MdlWrite, Readable, Writable};

fn print<T: MdlWrite>(value: &T) -> Result<String, WriteError> {
    let mut writer = MdlWriter::new(Vec::new());
    writer.write(value)?;
    Ok(String::from_utf8(writer.finish()?).unwrap())
}
fn positive_zero(value: &f32) -> bool {
    value.to_bits() == 0
}

#[derive(Debug, PartialEq, MdlRead, MdlWrite)]
#[mdl(block = "Anim")]
struct DerivedSequence {
    #[mdl(header)]
    name: FixedText<80>,
    #[mdl(property = "Interval")]
    interval: [u32; 2],
    #[mdl(property = "MoveSpeed", default, skip_if = "positive_zero")]
    move_speed: f32,
    #[mdl(flag = "NonLooping", default)]
    non_looping: bool,
}

#[test]
fn derived_codecs_read_any_field_order_and_print_declaration_order() {
    let value = DerivedSequence::parse_mdl(
        r#"Anim "Walk" { NonLooping, MoveSpeed 270.0, Interval { 3334, 6667 }, }"#,
    )
    .unwrap();
    assert!(value.non_looping);
    assert_eq!(value.interval, [3334, 6667]);
    let text = print(&value).unwrap();
    assert_eq!(
        text,
        "Anim \"Walk\" {\n\tInterval { 3334, 6667 },\n\tMoveSpeed 270.0,\n\tNonLooping,\n}\n"
    );
    assert_eq!(DerivedSequence::parse_mdl(&text).unwrap(), value);
    let mut value = DerivedSequence::parse_mdl("Anim \"Stand\" { Interval { 0, 1000 }, }").unwrap();
    assert!(!value.non_looping);
    assert!(!print(&value).unwrap().contains("MoveSpeed"));
    value.move_speed = -0.0;
    assert!(print(&value).unwrap().contains("MoveSpeed -0.0,"));
}

#[test]
fn required_duplicate_unknown_and_malformed_fields_have_source_spans() {
    let source = "Anim \"A\" {}";
    let error = DerivedSequence::parse_mdl(source).unwrap_err();
    assert_eq!(error.kind, ReadErrorKind::MissingField("Interval"));
    assert_eq!(&source[error.span.start..error.span.end], "}");
    for (source, kind, offending) in [
        (
            "Anim \"A\" { Interval { 0, 1 }, Interval { 2, 3 }, }",
            ReadErrorKind::DuplicateField,
            "Interval",
        ),
        (
            "Anim \"A\" { NonLooping, NonLooping, }",
            ReadErrorKind::DuplicateField,
            "NonLooping",
        ),
        (
            "Anim \"A\" { Bogus 3, }",
            ReadErrorKind::UnknownField,
            "Bogus",
        ),
    ] {
        let error = DerivedSequence::parse_mdl(source).unwrap_err();
        assert_eq!(error.kind, kind);
        assert_eq!(&source[error.span.start..error.span.end], offending);
    }
    assert!(DerivedSequence::parse_mdl("Anim \"A\" { Interval { 0, 1 } }").is_err());
    assert!(DerivedSequence::parse_mdl("Anim \"A\" { NonLooping 1, Interval { 0, 1 }, }").is_err());
}

#[derive(Debug, PartialEq)]
struct Special(u32);
fn special_default() -> Special {
    Special(7)
}
fn read_special(parser: &mut Parser<'_>) -> Result<Special, ReadError> {
    Ok(Special(parser.read()?))
}
fn write_special<W: Write>(value: &Special, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
    writer.write(&value.0)
}
fn special_is_default(value: &Special) -> bool {
    value.0 == 7
}
fn reserved_default() -> [u8; 4] {
    [0; 4]
}

#[derive(Debug, PartialEq, MdlRead, MdlWrite)]
#[mdl(
    block = "Custom",
    validate_read = "Custom::validate_read",
    validate_write = "Custom::validate_write"
)]
struct Custom {
    #[mdl(header, read_with = "read_special", write_with = "write_special")]
    id: Special,
    #[mdl(
        property = "Value",
        default = "special_default",
        read_with = "read_special",
        write_with = "write_special",
        skip_if = "special_is_default"
    )]
    value: Special,
    #[mdl(skip, default = "reserved_default")]
    reserved: [u8; 4],
}
impl Custom {
    fn validate_read(&self, span: Span) -> Result<(), ReadError> {
        if self.value.0 > 100 {
            Err(ReadError::new(
                span,
                ReadErrorKind::InvalidNumber("value <= 100"),
            ))
        } else {
            Ok(())
        }
    }
    fn validate_write(&self) -> Result<(), WriteError> {
        if self.reserved != [0; 4] || self.value.0 > 100 {
            Err(WriteError::Unsupported("custom data"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn hooks_do_not_require_codec_or_default_traits_on_the_field_type() {
    let value = Custom::parse_mdl("Custom 42 {}").unwrap();
    assert_eq!(value.value, Special(7));
    assert_eq!(value.reserved, [0; 4]);
    assert_eq!(print(&value).unwrap(), "Custom 42 {\n}\n");
    let value = Custom::parse_mdl("Custom 42 { Value 8, }").unwrap();
    assert_eq!(Custom::parse_mdl(&print(&value).unwrap()).unwrap(), value);
    let source = "Custom 42 { Value 101, }";
    let error = Custom::parse_mdl(source).unwrap_err();
    assert_eq!(error.span, Span::new(0, source.len()));
    let invalid = Custom {
        reserved: [1; 4],
        ..value
    };
    let mut output = Vec::new();
    assert!(matches!(
        MdlWriter::new(&mut output).write(&invalid),
        Err(WriteError::Unsupported(_))
    ));
    assert!(output.is_empty());
}

// Non-codec parameters, const parameters, existing bounds, and sink-name clashes
// must not introduce extra MdlRead/MdlWrite requirements.
struct NotACodec;
#[derive(MdlRead, MdlWrite)]
#[mdl(block = "Generic")]
struct Generic<'a, __Wc3MdlSink, Fields, const N: usize>
where
    Fields: Copy,
{
    #[mdl(property = "Value")]
    value: Fields,
    #[mdl(property = "Vector")]
    vector: [u32; N],
    #[mdl(skip, default)]
    marker: PhantomData<&'a __Wc3MdlSink>,
}
#[test]
fn bounds_apply_only_to_fields_that_use_them() {
    let value =
        Generic::<NotACodec, u32, 2>::parse_mdl("Generic { Vector { 1, 2 }, Value 9, }").unwrap();
    let text = print(&value).unwrap();
    assert_eq!(
        Generic::<NotACodec, u32, 2>::parse_mdl(&text)
            .unwrap()
            .value,
        9
    );
}

#[derive(MdlRead, MdlWrite)]
#[mdl(block = "Headers")]
struct Headers {
    #[mdl(header)]
    label: FixedText<32>,
    #[mdl(header)]
    id: i32,
    #[mdl(skip, default)]
    marker: PhantomData<NotACodec>,
}
#[derive(MdlRead, MdlWrite)]
#[mdl(block = "Empty")]
struct Empty {}
#[derive(MdlRead, MdlWrite)]
#[mdl(block = "RequiredFlag")]
struct RequiredFlag {
    #[mdl(flag = "Enabled")]
    enabled: bool,
}

#[test]
fn multiple_headers_empty_blocks_and_required_flags_work() {
    let value = Headers::parse_mdl("Headers \"a\\b\" -3 {}").unwrap();
    assert_eq!(print(&value).unwrap(), "Headers \"a\\b\" -3 {\n}\n");
    assert_eq!(
        print(&Empty::parse_mdl("Empty {}").unwrap()).unwrap(),
        "Empty {\n}\n"
    );
    assert!(Empty::parse_mdl("Empty { Bad, }").is_err());
    assert!(RequiredFlag::parse_mdl("RequiredFlag {}").is_err());
    assert!(matches!(
        print(&RequiredFlag { enabled: false }),
        Err(WriteError::Unsupported(_))
    ));
    assert!(
        RequiredFlag::parse_mdl(&print(&RequiredFlag { enabled: true }).unwrap())
            .unwrap()
            .enabled
    );
}

#[test]
fn derived_model_info_roundtrips_binary_data_and_rejects_animation_file_data() {
    let source = "Model \"Example\" { BlendTime 150, BoundsRadius 10.0, MinimumExtent { -1.0, -2.0, -3.0 }, MaximumExtent { 1.0, 2.0, 3.0 }, }";
    let value = ModelInfo::parse_mdl(source).unwrap();
    assert_eq!(value.name(), "Example");
    let bytes = value.encode().unwrap();
    assert_eq!(
        ModelInfo::parse_mdl(&print(&ModelInfo::decode(&bytes).unwrap()).unwrap())
            .unwrap()
            .encode()
            .unwrap(),
        bytes
    );
    let mut bytes = bytes;
    bytes[80..85].copy_from_slice(b"a.mdx");
    assert!(matches!(
        print(&ModelInfo::decode(&bytes).unwrap()),
        Err(WriteError::Unsupported(_))
    ));
}

struct ReadOnly(u32);
impl MdlRead for ReadOnly {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, ReadError> {
        Ok(Self(parser.read()?))
    }
}
struct WriteOnly(u32);
impl MdlWrite for WriteOnly {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        writer.write(&self.0)
    }
}
#[derive(MdlRead, MdlWrite)]
#[mdl(block = "Independent")]
struct Independent<T> {
    #[mdl(property = "Value")]
    value: T,
}
#[derive(MdlRead, MdlWrite)]
#[mdl(block = "Optional")]
struct Optional<T> {
    #[mdl(property = "Value", default)]
    value: T,
}
#[test]
fn read_write_and_default_bounds_are_independent() {
    let value = Independent::<ReadOnly>::parse_mdl("Independent { Value 42, }").unwrap();
    assert_eq!(value.value.0, 42);
    assert!(print(&Independent {
        value: WriteOnly(42)
    })
    .unwrap()
    .contains("Value 42,"));
    // WriteOnly implements neither Default nor MdlRead.
    assert!(print(&Optional {
        value: WriteOnly(7)
    })
    .unwrap()
    .contains("Value 7,"));
}

// Packed storage needs only MdlFlags, not Default or either value codec.
struct BareFlags(u32);
impl MdlFlags for BareFlags {
    fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    fn bits(&self) -> u32 {
        self.0
    }
}
#[derive(MdlRead, MdlWrite)]
#[mdl(block = "Packed", write_order(flags, id))]
struct Packed<T> {
    #[mdl(property = "Id", default)]
    id: u32,
    #[mdl(flags(First = 1, Last = 0x80000000))]
    flags: T,
}
#[test]
fn packed_flags_share_storage_but_track_duplicates_independently() {
    let value = Packed::<BareFlags>::parse_mdl("Packed { Id 7, Last, First, }").unwrap();
    assert_eq!(value.flags.bits(), 0x80000001);
    assert_eq!(
        print(&value).unwrap(),
        "Packed {\n\tFirst,\n\tLast,\n\tId 7,\n}\n"
    );
    let empty = Packed::<BareFlags>::parse_mdl("Packed {}").unwrap();
    assert_eq!(empty.flags.bits(), 0);
    let source = "Packed { Last, Last, }";
    let error = Packed::<BareFlags>::parse_mdl(source).err().unwrap();
    assert_eq!(error.kind, ReadErrorKind::DuplicateField);
    assert_eq!(&source[error.span.start..error.span.end], "Last");
    let mut output = Vec::new();
    assert!(matches!(
        MdlWriter::new(&mut output).write(&Packed {
            id: 7,
            flags: BareFlags(2)
        }),
        Err(WriteError::Unsupported(_))
    ));
    assert!(output.is_empty());
}

#[derive(MdlRead, MdlWrite)]
#[mdl(property = "Number")]
struct Number<T>(T);
#[derive(MdlRead, MdlWrite)]
#[mdl(entry)]
struct Entry<T>(T);
#[test]
fn tuple_value_forms_preserve_prefixes_and_entry_punctuation() {
    assert_eq!(
        print(&Number::<u32>::parse_mdl("Number 42,").unwrap()).unwrap(),
        "Number 42,\n"
    );
    assert_eq!(
        Entry::<[f32; 3]>::parse_mdl("{ 1.0, 2.0, 3.0 },")
            .unwrap()
            .0,
        [1.0, 2.0, 3.0]
    );
    assert_eq!(print(&Entry(3u32)).unwrap(), "3,\n");
    assert!(Number::<u32>::parse_mdl("Other 42,").is_err());
    assert!(Number::<u32>::parse_mdl("Number 42").is_err());
    assert!(Entry::<u32>::parse_mdl("3").is_err());
}

#[test]
fn typed_record_flags_keep_their_original_binary_layout_and_unknown_bits() {
    use wc3_mdx::animation::{Sequence, SequenceFlags};
    use wc3_mdx::materials::{Texture, TextureFlags};
    let raw = 0x80000003u32.to_le_bytes();
    assert_eq!(TextureFlags::decode(&raw).unwrap().encode().unwrap(), raw);
    let raw = 0x80000001u32.to_le_bytes();
    assert_eq!(SequenceFlags::decode(&raw).unwrap().encode().unwrap(), raw);
    let mut texture = Texture::new("a").unwrap();
    texture.set_flags(TextureFlags(0x80000003));
    let bytes = texture.encode().unwrap();
    assert_eq!(&bytes[264..268], &0x80000003u32.to_le_bytes());
    assert_eq!(Texture::decode(&bytes).unwrap().encode().unwrap(), bytes);
    assert!(print(&texture).is_err());
    let mut sequence = Sequence::new("a", [0, 1]).unwrap();
    sequence.set_flags(SequenceFlags(0x80000001));
    let bytes = sequence.encode().unwrap();
    assert_eq!(&bytes[92..96], &0x80000001u32.to_le_bytes());
    assert_eq!(Sequence::decode(&bytes).unwrap().encode().unwrap(), bytes);
    assert!(print(&sequence).is_err());
    sequence.set_flags(SequenceFlags(1));
    sequence.set_move_speed(2.0);
    sequence.set_rarity(3.0);
    sequence.set_sync_point(4);
    assert_eq!(print(&sequence).unwrap(), "Anim \"a\" {\n\tInterval { 0, 1 },\n\tNonLooping,\n\tMoveSpeed 2.0,\n\tRarity 3.0,\n\tSyncPoint 4,\n\tMinimumExtent { 0.0, 0.0, 0.0 },\n\tMaximumExtent { 0.0, 0.0, 0.0 },\n\tBoundsRadius 0.0,\n}\n");
}
