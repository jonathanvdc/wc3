use std::io::Write;
use std::marker::PhantomData;
use wc3::model::mdl;
use wc3::model::mdl::Read as _;
use wc3::model::mdl::{MdlWriter, Parser, ReadError, ReadErrorKind, Span, WriteError};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;
use wc3::model::scene::ModelInfo;
use wc3::model::FixedText;

fn print<T: mdl::Write>(value: &T) -> Result<String, WriteError> {
    value.encode_mdl()
}
fn positive_zero(value: &f32) -> bool {
    value.to_bits() == 0
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
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
    let value = DerivedSequence::decode_mdl(
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
    assert_eq!(DerivedSequence::decode_mdl(&text).unwrap(), value);
    let mut value =
        DerivedSequence::decode_mdl("Anim \"Stand\" { Interval { 0, 1000 }, }").unwrap();
    assert!(!value.non_looping);
    assert!(!print(&value).unwrap().contains("MoveSpeed"));
    value.move_speed = -0.0;
    assert!(print(&value).unwrap().contains("MoveSpeed -0.0,"));
}

#[test]
fn required_duplicate_unknown_and_malformed_fields_have_source_spans() {
    let source = "Anim \"A\" {}";
    let error = DerivedSequence::decode_mdl(source).unwrap_err();
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
        let error = DerivedSequence::decode_mdl(source).unwrap_err();
        assert_eq!(error.kind, kind);
        assert_eq!(&source[error.span.start..error.span.end], offending);
    }
    assert!(DerivedSequence::decode_mdl("Anim \"A\" { Interval { 0, 1 } }").is_err());
    assert!(
        DerivedSequence::decode_mdl("Anim \"A\" { NonLooping 1, Interval { 0, 1 }, }").is_err()
    );
}

#[derive(Debug, PartialEq)]
struct Special(u32);
impl mdl::ValueEq for Special {
    fn eq_mdl(&self, other: &Self) -> bool {
        self == other
    }
}
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

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
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
    let value = Custom::decode_mdl("Custom 42 {}").unwrap();
    assert_eq!(value.value, Special(7));
    assert_eq!(value.reserved, [0; 4]);
    assert_eq!(print(&value).unwrap(), "Custom 42 {\n}\n");
    let value = Custom::decode_mdl("Custom 42 { Value 8, }").unwrap();
    assert_eq!(Custom::decode_mdl(&print(&value).unwrap()).unwrap(), value);
    let source = "Custom 42 { Value 101, }";
    let error = Custom::decode_mdl(source).unwrap_err();
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
// must not introduce extra mdl::Read/mdl::Write requirements.
struct NotACodec;
#[derive(mdl::Read, mdl::Write)]
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
        Generic::<NotACodec, u32, 2>::decode_mdl("Generic { Vector { 1, 2 }, Value 9, }").unwrap();
    let text = print(&value).unwrap();
    assert_eq!(
        Generic::<NotACodec, u32, 2>::decode_mdl(&text)
            .unwrap()
            .value,
        9
    );
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Headers")]
struct Headers {
    #[mdl(header)]
    label: FixedText<32>,
    #[mdl(header)]
    id: i32,
    #[mdl(skip, default)]
    marker: PhantomData<NotACodec>,
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Empty")]
struct Empty {}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "RequiredFlag")]
struct RequiredFlag {
    #[mdl(flag = "Enabled")]
    enabled: bool,
}

#[test]
fn multiple_headers_empty_blocks_and_required_flags_work() {
    let value = Headers::decode_mdl("Headers \"a\\b\" -3 {}").unwrap();
    assert_eq!(print(&value).unwrap(), "Headers \"a\\b\" -3 {\n}\n");
    assert_eq!(
        print(&Empty::decode_mdl("Empty {}").unwrap()).unwrap(),
        "Empty {\n}\n"
    );
    assert!(Empty::decode_mdl("Empty { Bad, }").is_err());
    assert!(RequiredFlag::decode_mdl("RequiredFlag {}").is_err());
    assert!(matches!(
        print(&RequiredFlag { enabled: false }),
        Err(WriteError::Unsupported(_))
    ));
    assert!(
        RequiredFlag::decode_mdl(&print(&RequiredFlag { enabled: true }).unwrap())
            .unwrap()
            .enabled
    );
}

#[test]
fn derived_model_info_roundtrips_binary_data_and_rejects_animation_file_data() {
    let source = "Model \"Example\" { BlendTime 150, BoundsRadius 10.0, MinimumExtent { -1.0, -2.0, -3.0 }, MaximumExtent { 1.0, 2.0, 3.0 }, }";
    let value = ModelInfo::decode_mdl(source).unwrap();
    assert_eq!(value.name(), "Example");
    let bytes = value.encode_mdx().unwrap();
    assert_eq!(
        ModelInfo::decode_mdl(&print(&ModelInfo::decode_mdx(&bytes).unwrap()).unwrap())
            .unwrap()
            .encode_mdx()
            .unwrap(),
        bytes
    );
    let mut bytes = bytes;
    bytes[80..85].copy_from_slice(b"a.mdx");
    assert!(matches!(
        print(&ModelInfo::decode_mdx(&bytes).unwrap()),
        Err(WriteError::Unsupported(_))
    ));
}

struct ReadOnly(u32);
impl mdl::Read for ReadOnly {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, ReadError> {
        Ok(Self(parser.read()?))
    }
}
struct WriteOnly(u32);
impl mdl::Write for WriteOnly {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        writer.write(&self.0)
    }
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Independent")]
struct Independent<T> {
    #[mdl(property = "Value")]
    value: T,
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Optional")]
struct Optional<T> {
    #[mdl(property = "Value", default)]
    value: T,
}
#[test]
fn read_write_and_default_bounds_are_independent() {
    let value = Independent::<ReadOnly>::decode_mdl("Independent { Value 42, }").unwrap();
    assert_eq!(value.value.0, 42);
    assert!(print(&Independent {
        value: WriteOnly(42)
    })
    .unwrap()
    .contains("Value 42,"));
    // WriteOnly implements neither Default nor mdl::Read.
    assert!(print(&Optional {
        value: WriteOnly(7)
    })
    .unwrap()
    .contains("Value 7,"));
}

// Packed storage uses bitfield traits without either value codec.
bitfield::bitfield! {
    #[derive(Default)]
    struct BareFlags(u32);
    bits, _: 31, 0;
}
#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Packed", write_order(flags, id))]
struct Packed<T> {
    #[mdl(property = "Id", default)]
    id: u32,
    #[mdl(flags(First = 1, Last = 0x80000000))]
    flags: T,
}
#[test]
fn packed_flags_share_storage_but_track_duplicates_independently() {
    let value = Packed::<BareFlags>::decode_mdl("Packed { Id 7, Last, First, }").unwrap();
    assert_eq!(value.flags.bits(), 0x80000001);
    assert_eq!(
        print(&value).unwrap(),
        "Packed {\n\tFirst,\n\tLast,\n\tId 7,\n}\n"
    );
    let empty = Packed::<BareFlags>::decode_mdl("Packed {}").unwrap();
    assert_eq!(empty.flags.bits(), 0);
    let source = "Packed { Last, Last, }";
    let error = Packed::<BareFlags>::decode_mdl(source).err().unwrap();
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

#[derive(mdl::Read, mdl::Write)]
#[mdl(property = "Number")]
struct Number<T>(T);
#[derive(mdl::Read, mdl::Write)]
#[mdl(entry)]
struct Entry<T>(T);
#[test]
fn tuple_value_forms_preserve_prefixes_and_entry_punctuation() {
    assert_eq!(
        print(&Number::<u32>::decode_mdl("Number 42,").unwrap()).unwrap(),
        "Number 42,\n"
    );
    assert_eq!(
        Entry::<[f32; 3]>::decode_mdl("{ 1.0, 2.0, 3.0 },")
            .unwrap()
            .0,
        [1.0, 2.0, 3.0]
    );
    assert_eq!(print(&Entry(3u32)).unwrap(), "3,\n");
    assert!(Number::<u32>::decode_mdl("Other 42,").is_err());
    assert!(Number::<u32>::decode_mdl("Number 42").is_err());
    assert!(Entry::<u32>::decode_mdl("3").is_err());
}

#[test]
fn typed_record_flags_keep_their_original_binary_layout_and_unknown_bits() {
    use wc3::model::animation::{Sequence, SequenceFlags};
    use wc3::model::materials::{Texture, TextureFlags};
    let raw = 0x80000003u32.to_le_bytes();
    assert_eq!(
        TextureFlags::decode_mdx(&raw)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        raw
    );
    let raw = 0x80000001u32.to_le_bytes();
    assert_eq!(
        SequenceFlags::decode_mdx(&raw)
            .unwrap()
            .encode_mdx()
            .unwrap(),
        raw
    );
    let mut texture = Texture::new("a").unwrap();
    texture.set_flags(TextureFlags(0x80000003));
    let bytes = texture.encode_mdx().unwrap();
    assert_eq!(&bytes[264..268], &0x80000003u32.to_le_bytes());
    assert_eq!(
        Texture::decode_mdx(&bytes).unwrap().encode_mdx().unwrap(),
        bytes
    );
    assert!(print(&texture).is_err());
    let mut sequence = Sequence::new("a", [0, 1]).unwrap();
    sequence.set_flags(SequenceFlags(0x80000001));
    let bytes = sequence.encode_mdx().unwrap();
    assert_eq!(&bytes[92..96], &0x80000001u32.to_le_bytes());
    assert_eq!(
        Sequence::decode_mdx(&bytes).unwrap().encode_mdx().unwrap(),
        bytes
    );
    assert!(print(&sequence).is_err());
    sequence.set_flags(SequenceFlags(1));
    sequence.set_move_speed(2.0);
    sequence.set_rarity(3.0);
    sequence.set_sync_point(4);
    assert_eq!(print(&sequence).unwrap(), "Anim \"a\" {\n\tInterval { 0, 1 },\n\tNonLooping,\n\tMoveSpeed 2.0,\n\tRarity 3.0,\n\tSyncPoint 4,\n\tMinimumExtent { 0.0, 0.0, 0.0 },\n\tMaximumExtent { 0.0, 0.0, 0.0 },\n\tBoundsRadius 0.0,\n}\n");
}

#[derive(Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Static")]
struct StaticProperties<T> {
    #[mdl(static_property = "Required")]
    required: T,
    #[mdl(static_property = "Optional", default, skip_if = "positive_zero")]
    optional: f32,
}

#[test]
fn static_properties_require_prefix_and_preserve_defaults_and_bits() {
    let value = StaticProperties::<u32>::decode_mdl("Static { static Required 7, }").unwrap();
    assert_eq!(value.optional.to_bits(), 0);
    assert_eq!(
        print(&value).unwrap(),
        "Static {\n\tstatic Required 7,\n}\n"
    );
    let value =
        StaticProperties::<u32>::decode_mdl("Static { static Optional -0.0, static Required 8, }")
            .unwrap();
    assert!(print(&value).unwrap().contains("static Optional -0.0,"));
    for source in [
        "Static { Required 7, }",
        "Static { static Unknown 7, }",
        "Static { static Required 7 }",
        "Static { }",
    ] {
        assert!(StaticProperties::<u32>::decode_mdl(source).is_err());
    }
    let error =
        StaticProperties::<u32>::decode_mdl("Static { static Required 7, static Required 8, }")
            .unwrap_err();
    assert_eq!(error.kind, ReadErrorKind::DuplicateField);
}

use wc3::model::animation::{AnimationTrack, GeosetAlpha, GeosetColor, GeosetTrack};

fn full_alpha() -> f32 {
    1.0
}

#[derive(Debug, mdl::Read, mdl::Write)]
#[mdl(
    block = "Linked",
    write_order(enabled, alpha, tracks),
    validate_read = "Self::check_read"
)]
struct LinkedProperties<T> {
    #[mdl(
        animatable = "Alpha",
        track = "GeosetTrack::Alpha",
        default = "full_alpha",
        enabled_if = "Self::is_enabled",
        enable_with = "Self::enable"
    )]
    alpha: f32,
    #[mdl(tracks)]
    tracks: Vec<GeosetTrack>,
    #[mdl(flag = "Enabled", default)]
    enabled: bool,
    #[mdl(skip, default)]
    marker: PhantomData<T>,
}

impl<T> LinkedProperties<T> {
    fn check_read(&self, span: Span) -> Result<(), ReadError> {
        if (!self.tracks.is_empty() || self.alpha != 1.0) && !self.enabled {
            return Err(ReadError::new(
                span,
                ReadErrorKind::MissingField("enabled property"),
            ));
        }
        Ok(())
    }
    fn is_enabled(&self) -> bool {
        self.enabled
    }
    fn enable(&mut self) {
        self.enabled = true;
    }
}

#[test]
fn linked_properties_enable_both_forms_and_validate_the_collection() {
    struct NoCodec;
    let absent = LinkedProperties::<NoCodec>::decode_mdl("Linked { }").unwrap();
    assert_eq!(absent.alpha, 1.0);
    assert!(!absent.enabled);
    assert_eq!(print(&absent).unwrap(), "Linked {\n}\n");
    let fixed = LinkedProperties::<NoCodec>::decode_mdl("Linked { static Alpha 0.5, }").unwrap();
    assert!(fixed.enabled);
    assert_eq!(fixed.alpha, 0.5);
    assert!(print(&fixed).unwrap().contains("static Alpha 0.5,"));
    let animated =
        LinkedProperties::<NoCodec>::decode_mdl("Linked { Alpha 0 { Linear, } }").unwrap();
    assert!(animated.enabled);
    assert_eq!(animated.alpha, 1.0);
    assert_eq!(animated.tracks.len(), 1);
    assert!(!print(&animated).unwrap().contains("static Alpha"));
    assert!(print(&animated).unwrap().contains("Alpha 0 {"));
    for source in [
        "Linked { static Alpha 1, Alpha 0 { Linear, } }",
        "Linked { Alpha 0 { Linear, } static Alpha 1, }",
        "Linked { Alpha 0 { Linear, } Alpha 0 { Linear, } }",
    ] {
        assert_eq!(
            LinkedProperties::<NoCodec>::decode_mdl(source)
                .err()
                .unwrap()
                .kind,
            ReadErrorKind::DuplicateField
        );
    }
    let mut invalid = absent;
    let alpha = AnimationTrack::<GeosetAlpha>::linear(Vec::new(), None).unwrap();
    invalid.tracks = vec![alpha.clone().into(), alpha.into()];
    let mut writer = MdlWriter::new(Vec::new());
    assert!(writer.write(&invalid).is_err());
    assert!(writer.finish().unwrap().is_empty());
    invalid.tracks = vec![AnimationTrack::<GeosetColor>::linear(Vec::new(), None)
        .unwrap()
        .into()];
    assert!(print(&invalid).is_err());
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "CustomStatic")]
struct CustomStatic {
    #[mdl(
        static_property = "Value",
        read_with = "read_special",
        write_with = "write_special"
    )]
    value: Special,
}

#[test]
fn static_value_hooks_do_not_require_codec_traits() {
    let value = CustomStatic::decode_mdl("CustomStatic { static Value 9, }").unwrap();
    assert_eq!(
        print(&value).unwrap(),
        "CustomStatic {\n\tstatic Value 9,\n}\n"
    );
}

// The collection enum deliberately has no Read implementation.
enum LinkedTrack<T> {
    Alpha(AnimationTrack<GeosetAlpha>),
    Unmapped(PhantomData<T>),
}
impl<T> mdl::Write for LinkedTrack<T> {
    fn write_mdl<W: Write>(&self, writer: &mut MdlWriter<W>) -> Result<(), WriteError> {
        match self {
            Self::Alpha(track) => writer.write(track),
            Self::Unmapped(_) => Err(WriteError::Unsupported("unmapped track")),
        }
    }
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "GenericLinked")]
struct GenericLinked<T> {
    #[mdl(
        animatable = "Alpha",
        track = "LinkedTrack::Alpha",
        default = "full_alpha"
    )]
    alpha: f32,
    #[mdl(tracks)]
    tracks: Vec<LinkedTrack<T>>,
}

#[test]
fn linked_enum_construction_preserves_generic_bounds() {
    struct NoCodec;
    let mut value =
        GenericLinked::<NoCodec>::decode_mdl("GenericLinked { Alpha 0 { Linear, } }").unwrap();
    assert_eq!(value.alpha, 1.0);
    assert!(print(&value).unwrap().contains("Alpha 0 {"));
    value.tracks.push(LinkedTrack::Unmapped(PhantomData));
    assert!(print(&value).is_err());
}

fn count_override() -> u32 {
    12
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(block = "Defaults", default)]
struct RecordDefaults {
    #[mdl(header)]
    name: FixedText<16>,
    #[mdl(property = "Id", required)]
    id: u32,
    #[mdl(static_property = "Alpha")]
    alpha: f32,
    #[mdl(property = "Count", default = "count_override")]
    count: u32,
    #[mdl(flags(A = 1, B = 2))]
    flags: u32,
    #[mdl(skip)]
    hidden: Special,
}

impl Default for RecordDefaults {
    fn default() -> Self {
        Self {
            name: FixedText::default(),
            id: 0,
            alpha: 1.0,
            count: 9,
            flags: 2,
            hidden: Special(7),
        }
    }
}

#[test]
fn record_default_supplies_fields_without_field_default_bounds() {
    let value = RecordDefaults::decode_mdl("Defaults \"Example\" { Id 3, A, }").unwrap();
    assert_eq!(value.name.text(), "Example");
    assert_eq!(value.id, 3);
    assert_eq!(value.alpha, 1.0);
    assert_eq!(value.count, 12);
    assert_eq!(value.flags, 3);
    assert_eq!(value.hidden.0, 7);
    assert!(print(&value).unwrap().contains("static Alpha 1.0,"));
    let value =
        RecordDefaults::decode_mdl("Defaults \"Example\" { Id 4, static Alpha -0.0, Count 20, }")
            .unwrap();
    assert_eq!(value.alpha.to_bits(), (-0.0f32).to_bits());
    assert_eq!(value.count, 20);
    assert_eq!(value.flags, 2);
    let mut cleared = value;
    cleared.flags = 0;
    assert!(print(&cleared).is_err());
    assert_eq!(
        RecordDefaults::decode_mdl("Defaults \"Example\" { }")
            .err()
            .unwrap()
            .kind,
        ReadErrorKind::MissingField("Id")
    );
}
