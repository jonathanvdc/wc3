use std::io::Write as IoWrite;
use std::marker::PhantomData;

use wc3::model::chunks::{Chunk, KnownChunk, ModelChunk, RawChunk, UnknownChunk};
use wc3::model::geometry::{BindPoseMatrix, Geoset};
use wc3::model::materials::Material;
use wc3::model::mdl::{self, Read as _, Write as _};
use wc3::model::mdx::{self, Extension as _, Read as _, Write as _};
use wc3::model::scene::ModelInfo;
use wc3::model::{
    visit_model, CommonModelAccess, ConversionIssueKind, ConversionOptions, Cursor, DynamicModel,
    Encoder, Extended, IoError, Model, ModelDialect, ModelVersion, NoExtensions, Tag,
    TryModelAccess, UnknownChunkPolicy, V1800, V800,
};

#[derive(Clone, Debug, PartialEq)]
enum ApplicationChunk<V: ModelVersion = V1800> {
    Note(u32),
    VersionBound(u32, PhantomData<V>),
    Reserved,
    WrongTag,
}

impl<V: ModelVersion> Chunk for ApplicationChunk<V> {
    fn tag(&self) -> Tag {
        match self {
            Self::Note(_) => *b"NOTE",
            Self::VersionBound(_, _) => *b"VBND",
            Self::Reserved => *b"VERS",
            Self::WrongTag => *b"DIFF",
        }
    }

    fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        match self {
            Self::Note(value) => output.write(value),
            Self::VersionBound(value, _) => {
                output.write(&V::NUMBER)?;
                output.write(value)
            }
            Self::Reserved | Self::WrongTag => Ok(()),
        }
    }
}

impl<V: ModelVersion> mdx::Extension for ApplicationChunk<V> {
    fn read_extension(tag: Tag, input: &mut Cursor<'_>) -> Result<Option<Self>, mdx::ReadError> {
        Ok(match &tag {
            b"NOTE" => Some(Self::Note(input.read()?)),
            b"VBND" => {
                let encoded_version: u32 = input.read()?;
                if encoded_version != V::NUMBER {
                    return Err(mdx::ReadError::new(
                        input.absolute_position(),
                        mdx::ReadErrorKind::VersionMismatch {
                            expected: V::NUMBER,
                            actual: encoded_version,
                        },
                    ));
                }
                Some(Self::VersionBound(input.read()?, PhantomData))
            }
            b"WRNG" => Some(Self::WrongTag),
            _ => None,
        })
    }
}

type Application1800 = Extended<V1800, ApplicationChunk>;
type Application800 = Extended<V800, ApplicationChunk<V800>>;

#[test]
fn standard_records_use_the_base_version_and_custom_dialects_are_open() {
    #[derive(Clone, Debug)]
    struct ApplicationDialect;
    impl ModelDialect for ApplicationDialect {
        type Version = V1800;
        type Extension = ApplicationChunk;
    }
    let mut model = Model::<ApplicationDialect>::new();
    model.set_geosets(&[Geoset::<V1800>::new(&[], &[], &[]).unwrap()]);
    model.set_materials(&[Material::<V1800>::new()]);
    model.set_bind_poses(&[BindPoseMatrix([0.; 12])]);
    model
        .chunks
        .push(ModelChunk::Extension(ApplicationChunk::Note(42)));
    let bytes = model.encode_mdx().unwrap();
    let decoded = Model::<ApplicationDialect>::decode_mdx(&bytes).unwrap();
    assert_eq!(decoded.version(), 1800);
    assert_eq!(decoded.geosets().len(), 1);
    assert_eq!(decoded.materials().len(), 1);
    assert!(matches!(
        decoded.chunks.last(),
        Some(ModelChunk::Extension(ApplicationChunk::Note(42)))
    ));
    assert_eq!(decoded.encode_mdx().unwrap(), bytes);
}

#[test]
fn unknown_chunks_and_repeated_extensions_retain_order() {
    let mut model = Model::<Application1800>::new();
    model
        .chunks
        .push(ModelChunk::Extension(ApplicationChunk::Note(1)));
    model.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk::new(*b"FUTR", vec![9, 8, 7])).unwrap(),
    ));
    model
        .chunks
        .push(ModelChunk::Extension(ApplicationChunk::Note(2)));
    let bytes = model.encode_mdx().unwrap();
    let decoded = Model::<Application1800>::decode_mdx(&bytes).unwrap();
    assert_eq!(
        decoded
            .chunks
            .iter()
            .map(ModelChunk::tag)
            .collect::<Vec<_>>(),
        [*b"VERS", *b"NOTE", *b"FUTR", *b"NOTE"]
    );
    assert!(
        matches!(&decoded.chunks[2], ModelChunk::Unknown(chunk) if chunk.raw().data == [9, 8, 7])
    );
    assert_eq!(decoded.encode_mdx().unwrap(), bytes);
    let standard = Model::<V1800>::decode_mdx(&bytes).unwrap();
    assert!(matches!(&standard.chunks[1], ModelChunk::Unknown(_)));
    assert_eq!(standard.encode_mdx().unwrap(), bytes);
}

#[test]
fn runtime_dispatch_supports_extensions_in_every_version() {
    for version in [800u32, 900, 1000, 1100, 1200, 1300, 1400, 1600, 1800] {
        let mut bytes = b"MDLX".to_vec();
        let mut output = Encoder::new(&mut bytes);
        RawChunk::new(*b"VERS", version.to_le_bytes().to_vec())
            .write_mdx(&mut output)
            .unwrap();
        let payload = 17u32.to_le_bytes().to_vec();
        RawChunk::new(*b"NOTE", payload)
            .write_mdx(&mut output)
            .unwrap();
        let mut model = DynamicModel::<NoteChunk>::decode_mdx(&bytes, 800).unwrap();
        assert_eq!(model.version(), version);
        model.set_global_sequences(&[100]);
        assert_eq!(model.global_sequences(), [100]);
        assert_eq!(model.try_bind_poses().is_ok(), version >= 900);
        visit_model!(&model, |typed| assert!(matches!(
            typed.chunks[1],
            ModelChunk::Extension(NoteChunk(NotePayload { value: 17 }))
        )));
        let normalized = model.normalized().unwrap().model;
        assert_eq!(
            normalized.encode_mdx().unwrap(),
            model.encode_mdx().unwrap()
        );
    }
}

#[test]
fn missing_vers_uses_the_runtime_fallback_for_extensions() {
    let mut bytes = b"MDLX".to_vec();
    RawChunk::new(*b"NOTE", 5u32.to_le_bytes().to_vec())
        .write_mdx(&mut Encoder::new(&mut bytes))
        .unwrap();
    for version in [800, 1800] {
        let model = DynamicModel::<NoteChunk>::decode_mdx(&bytes, version).unwrap();
        assert_eq!(model.version(), version);
        assert_eq!(model.encode_mdx().unwrap(), bytes);
    }
}

#[test]
fn recognized_malformed_chunks_fail_with_payload_offsets_and_tag_context() {
    let mut bytes = Model::<V1800>::new().encode_mdx().unwrap();
    let offset = bytes.len();
    RawChunk::new(*b"NOTE", vec![1, 2])
        .write_mdx(&mut Encoder::new(&mut bytes))
        .unwrap();
    let error = Model::<Application1800>::decode_mdx(&bytes).unwrap_err();
    assert_eq!(error.tag, Some(*b"NOTE"));
    assert_eq!(error.offset, offset + 8);
    assert!(matches!(
        error.kind,
        mdx::ReadErrorKind::UnexpectedEnd { .. }
    ));
    let error =
        ModelChunk::<Application1800>::from_raw(RawChunk::new(*b"NOTE", vec![0; 5])).unwrap_err();
    assert_eq!(error.tag, Some(*b"NOTE"));
    assert!(matches!(
        error.kind,
        mdx::ReadErrorKind::TrailingBytes { remaining: 1 }
    ));
}

#[test]
fn standard_tags_and_decoder_tag_changes_are_rejected() {
    let chunk = ModelChunk::<Application1800>::Extension(ApplicationChunk::Reserved);
    assert!(
        matches!(chunk.encode_mdx(), Err(mdx::WriteError::InvalidValue { tag, .. }) if tag == *b"VERS")
    );
    let error =
        ModelChunk::<Application1800>::from_raw(RawChunk::new(*b"WRNG", vec![])).unwrap_err();
    assert_eq!(error.tag, Some(*b"WRNG"));
    assert!(matches!(
        error.kind,
        mdx::ReadErrorKind::InvalidValue { .. }
    ));
    // The standard decoder always owns standard tags.
    assert!(ModelChunk::<Application1800>::from_raw(RawChunk::new(*b"VERS", vec![])).is_err());
}

#[test]
fn conversion_reclassifies_extensions_and_opaque_chunks_without_losing_bytes() {
    let mut model = Model::<Application1800>::new();
    model
        .chunks
        .push(ModelChunk::Extension(ApplicationChunk::Note(99)));
    let bytes = model.encode_mdx().unwrap();
    let standard = model
        .convert::<V1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert!(matches!(standard.chunks[1], ModelChunk::Unknown(_)));
    assert_eq!(standard.encode_mdx().unwrap(), bytes);
    let application = standard
        .convert::<Application1800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert!(matches!(
        application.chunks[1],
        ModelChunk::Extension(ApplicationChunk::Note(99))
    ));
    assert_eq!(application.encode_mdx().unwrap(), bytes);
    let downgraded = application
        .convert::<Application800>(&ConversionOptions::strict())
        .unwrap()
        .model;
    assert!(matches!(
        downgraded.chunks[1],
        ModelChunk::Extension(ApplicationChunk::Note(99))
    ));
}

#[test]
fn conversion_checks_version_bound_extensions_and_reports_invalid_targets() {
    let mut model = Model::<Application1800>::new();
    model
        .chunks
        .push(ModelChunk::Extension(ApplicationChunk::VersionBound(
            7,
            PhantomData,
        )));
    let original = model.encode_mdx().unwrap();
    let error = model
        .convert::<Application800>(&ConversionOptions::strict())
        .unwrap_err();
    assert_eq!(error.path, "chunks[1]");
    assert_eq!(model.encode_mdx().unwrap(), original);
    let mut standard = Model::<V1800>::new();
    standard.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk::new(*b"NOTE", vec![0])).unwrap(),
    ));
    assert!(standard
        .convert::<Application1800>(&ConversionOptions::strict())
        .is_err());
}

#[test]
fn mdl_extensions_round_trip_with_standard_records_and_dynamic_dispatch() {
    let mut model = Model::<Application1800>::new();
    model.set_model_info(&ModelInfo::new("Example").unwrap());
    for value in [42, 7] {
        model
            .chunks
            .push(ModelChunk::Extension(ApplicationChunk::Note(value)));
    }
    let source = model.encode_mdl().unwrap();
    let decoded = Model::<Application1800>::decode_mdl(&source).unwrap();
    assert_eq!(decoded.encode_mdx().unwrap(), model.encode_mdx().unwrap());
    assert_eq!(decoded.encode_mdl().unwrap(), source);
    let dynamic = DynamicModel::<ApplicationChunk>::decode_mdl(&source).unwrap();
    assert_eq!(dynamic.version(), 1800);
    assert_eq!(dynamic.encode_mdl().unwrap(), source);
    assert!(DynamicModel::<NoExtensions>::decode_mdl(&source).is_err());
    assert!(Model::<Application1800>::decode_mdl(
        "Version { FormatVersion 1800, } Model \"Example\" {} Note { Value \"bad\", }"
    )
    .is_err());
    assert!(Model::<Application1800>::decode_mdl(
        "Version { FormatVersion 1800, } Model \"Example\" {} Future {}"
    )
    .is_err());
    model.chunks.push(ModelChunk::Unknown(
        UnknownChunk::new(RawChunk::new(*b"FUTR", vec![1])).unwrap(),
    ));
    assert!(matches!(
        model.encode_mdl(),
        Err(mdl::WriteError::Unrepresentable { .. })
    ));
}

#[test]
fn extensions_unknown_to_a_new_version_follow_the_opaque_policy() {
    let mut model = Model::<Application1800>::new();
    model
        .chunks
        .push(ModelChunk::Extension(ApplicationChunk::Note(42)));
    assert!(model.convert::<V800>(&ConversionOptions::strict()).is_err());
    let mut options = ConversionOptions::strict();
    options.unknown_chunks = UnknownChunkPolicy::Preserve;
    let retained = model.convert::<V800>(&options).unwrap();
    assert!(matches!(retained.model.chunks[1], ModelChunk::Unknown(_)));
    assert_eq!(
        retained.report.issues[0].kind,
        ConversionIssueKind::PreservedUnknown
    );
    options.unknown_chunks = UnknownChunkPolicy::Drop;
    let dropped = model.convert::<V800>(&options).unwrap();
    assert_eq!(dropped.model.chunks.len(), 1);
    assert_eq!(dropped.report.issues[0].kind, ConversionIssueKind::Dropped);
    // A decoder that owns the tag can establish compatibility without opaque policy.
    let standard = Model::<V1800>::decode_mdx(&model.encode_mdx().unwrap()).unwrap();
    let recognized = standard
        .convert::<Application800>(&ConversionOptions::strict())
        .unwrap();
    assert!(matches!(
        recognized.model.chunks[1],
        ModelChunk::Extension(ApplicationChunk::Note(42))
    ));
}

#[test]
fn io_adapter_uses_the_selected_extension_codec() {
    let mut model = Model::<Application1800>::new();
    model
        .chunks
        .push(ModelChunk::Extension(ApplicationChunk::Note(42)));
    let bytes = model.encode_mdx().unwrap();
    let decoded: DynamicModel<ApplicationChunk> =
        mdx::from_reader_with_version(bytes.as_slice(), 800).unwrap();
    assert_eq!(decoded.encode_mdx().unwrap(), bytes);
    visit_model!(&decoded, |typed| assert!(matches!(
        typed.chunks[1],
        ModelChunk::Extension(ApplicationChunk::Note(42))
    )));
}

#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
struct NotePayload {
    value: u32,
}

#[derive(Clone, Debug, PartialEq)]
struct NoteChunk(NotePayload);

impl Chunk for NoteChunk {
    fn tag(&self) -> Tag {
        Self::TAG
    }

    fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        output.write(&self.0)
    }
}

impl KnownChunk for NoteChunk {
    const TAG: Tag = *b"NOTE";

    fn decode_payload(input: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        input.read().map(Self)
    }
}

#[test]
fn known_chunks_reuse_standard_codecs_and_dispatch_automatically() {
    type Dialect = Extended<V1800, NoteChunk>;
    let note = NoteChunk(NotePayload { value: 42 });
    let mut model = Model::<Dialect>::new();
    model.chunks.push(ModelChunk::Extension(note.clone()));
    let bytes = model.encode_mdx().unwrap();
    let decoded = Model::<Dialect>::decode_mdx(&bytes).unwrap();
    assert!(matches!(&decoded.chunks[1], ModelChunk::Extension(value) if value == &note));
    assert_eq!(decoded.encode_mdx().unwrap(), bytes);
    // Chunk codecs include exactly one header when used independently.
    assert_eq!(
        NoteChunk::decode_mdx(&note.encode_mdx().unwrap()).unwrap(),
        note
    );

    let mut input = Cursor::new(&[1, 2, 3]);
    assert_eq!(
        NoteChunk::read_extension(*b"ELSE", &mut input).unwrap(),
        None
    );
    assert_eq!(input.remaining(), &[1, 2, 3]);
    assert!(NoteChunk::read_extension(*b"NOTE", &mut input).is_err());
    assert!(ModelChunk::<Dialect>::from_raw(RawChunk::new(*b"NOTE", vec![0; 5])).is_err());
}

#[derive(Clone, Debug, mdl::Read, mdl::Write)]
#[mdl(block = "Note")]
struct NoteText {
    #[mdl(property = "Value")]
    value: u32,
}

impl<V: ModelVersion> mdl::Extension for ApplicationChunk<V> {
    fn read_extension(
        name: &str,
        parser: &mut mdl::Parser<'_>,
    ) -> Result<Option<Self>, mdl::ReadError> {
        match name {
            "Note" => parser
                .read::<NoteText>()
                .map(|note| Some(Self::Note(note.value))),
            _ => Ok(None),
        }
    }
}

impl<V: ModelVersion> mdl::Write for ApplicationChunk<V> {
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut mdl::Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        match self {
            Self::Note(value) => writer.write(&NoteText { value: *value }),
            _ => Err(mdl::WriteError::Unrepresentable {
                field: "application chunk",
            }
            .into()),
        }
    }
}

impl mdl::Extension for NoteText {
    fn read_extension(
        name: &str,
        parser: &mut mdl::Parser<'_>,
    ) -> Result<Option<Self>, mdl::ReadError> {
        if name == "Note" {
            parser.read().map(Some)
        } else {
            Ok(None)
        }
    }
}

#[test]
fn mdl_only_extensions_support_model_io_and_standard_accessors() {
    type TextDialect = Extended<V800, NoteText>;
    let mut model = Model::<TextDialect>::decode_mdl(
        "Version { FormatVersion 800, } Model \"Example\" {} Note { Value 9, }",
    )
    .unwrap();
    model.set_global_sequences(&[100]);
    model.set_gliders(&[]);
    assert_eq!(model.global_sequences(), [100]);
    let source = model.encode_mdl().unwrap();
    let mut dynamic = DynamicModel::<NoteText>::decode_mdl(&source).unwrap();
    assert_eq!(dynamic.version(), 800);
    dynamic.set_global_sequences(&[200]);
    assert_eq!(dynamic.global_sequences(), [200]);
    assert!(dynamic.encode_mdl().unwrap().contains("Value 9,"));
}

#[derive(Clone, Debug)]
struct ReadOnlyNote(NoteText);

impl mdl::Extension for ReadOnlyNote {
    fn read_extension(
        name: &str,
        parser: &mut mdl::Parser<'_>,
    ) -> Result<Option<Self>, mdl::ReadError> {
        if name == "Note" {
            parser.read().map(|value| Some(Self(value)))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone, Debug)]
struct WriteOnlyNote(NoteText);

impl mdl::Write for WriteOnlyNote {
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut mdl::Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        writer.write(&self.0)
    }
}

#[test]
fn mdl_reading_and_writing_require_only_their_respective_codecs() {
    let source = "Version { FormatVersion 800, } Model \"Example\" {} Note { Value 9, }";
    let model: Model<Extended<V800, ReadOnlyNote>> = source.parse().unwrap();
    assert!(matches!(&model.chunks[2], ModelChunk::Extension(note) if note.0.value == 9));
    let dynamic = DynamicModel::<ReadOnlyNote>::decode_mdl(source).unwrap();
    assert_eq!(dynamic.version(), 800);

    let mut output = Model::<Extended<V800, WriteOnlyNote>>::new();
    output.set_model_info(&ModelInfo::new("Example").unwrap());
    output
        .chunks
        .push(ModelChunk::Extension(WriteOnlyNote(NoteText { value: 9 })));
    let dynamic = DynamicModel::V800(output);
    assert!(dynamic.encode_mdl().unwrap().contains("Value 9,"));
}
