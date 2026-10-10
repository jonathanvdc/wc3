//! Whole-model assembly: top-level MDL blocks map to typed binary chunks.
use super::{Field, Fields, Parser, Span, TokenKind, Writer};
use crate::model::mdl;
use crate::model::IoError;
use crate::model::{
    AttachmentsChunk, BindPoseChunk, BonesChunk, CamerasChunk, CollectionChunk,
    CollisionShapesChunk, DynamicModel, EventObjectsChunk, FaceFxChunk, GeosetAnimationsChunk,
    GeosetsChunk, GlidersChunk, GlobalSequencesChunk, HelpersChunk, LightsChunk, MaterialsChunk,
    Model, ModelChunk, ModelInfoChunk, ModelVersion, ParticleEmitters2Chunk, ParticleEmittersChunk,
    PivotPointsChunk, PopcornEmittersChunk, RibbonEmittersChunk, SequencesChunk,
    TextureAnimationsChunk, TexturesChunk, VersionChunk, V1000, V1100, V1200, V1300, V1400, V1600,
    V1800, V800, V900,
};
use crate::model::{Extended, ModelDialect, ModelExtension};
use mdl::Read as _;
use std::io::Write as IoWrite;
use std::str::FromStr;

impl<V: ModelVersion> VersionChunk<V> {
    pub(crate) fn mdl_version(&self) -> u32 {
        V::NUMBER
    }
    pub(crate) fn set_mdl_version(
        &mut self,
        actual: u32,
        _: bool,
        span: Span,
    ) -> Result<(), mdl::ReadError> {
        if actual != V::NUMBER {
            return Err(mdl::ReadError::new(
                span,
                mdl::ReadErrorKind::VersionMismatch {
                    expected: V::NUMBER,
                    actual,
                },
            ));
        }
        Ok(())
    }
    pub(crate) fn validate_mdl_write(&self) -> Result<(), mdl::WriteError> {
        if !self.extension.is_empty() {
            return Err(mdl::WriteError::Unrepresentable {
                field: "version chunk extension",
            });
        }
        Ok(())
    }
}

fn require_version_first(parser: &mut Parser<'_>) -> Result<(), mdl::ReadError> {
    match parser.peek()? {
        Some(token) if token.kind == TokenKind::Ident("Version") => Ok(()),
        _ => Err(parser.error(mdl::ReadErrorKind::Expected("Version as the first block"))),
    }
}
fn read_collection<C: CollectionChunk>(parser: &mut Parser<'_>) -> Result<C, mdl::ReadError>
where
    C::Item: mdl::Read,
{
    let records = parser.counted::<C::Item>()?.collect::<Result<_, _>>()?;
    Ok(C::from_records(records))
}

impl<V: ModelDialect> mdl::Read for Model<V> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        require_version_first(parser)?;
        parser.read::<VersionChunk<V::Version>>()?;
        let mut model = Self::new();
        let mut fields = Fields::default();
        let mut has_info = false;
        while let Some(token) = parser.peek()? {
            let name = match token.kind {
                TokenKind::Ident(name) => name,
                _ => return Err(parser.error(mdl::ReadErrorKind::TrailingInput)),
            };
            let field = Field {
                name,
                span: token.span,
            };
            macro_rules! counted {
                ($bit:expr, $chunk:ty) => {{
                    fields.mark($bit, field)?;
                    parser.next_token()?;
                    model.chunks.push(read_collection::<$chunk>(parser)?.into());
                }};
            }
            macro_rules! repeated {
                ($variant:ident, $chunk:ty) => {{
                    let record = parser.read()?;
                    if let Some(chunk) = model.chunks.iter_mut().find_map(|chunk| match chunk {
                        ModelChunk::$variant(chunk) => Some(chunk),
                        _ => None,
                    }) {
                        chunk.records.push(record);
                    } else {
                        model
                            .chunks
                            .push(<$chunk>::from_records(vec![record]).into());
                    }
                }};
            }
            match name {
                "Version" => {
                    return Err(mdl::ReadError::new(
                        field.span,
                        mdl::ReadErrorKind::DuplicateField,
                    ))
                }
                "Model" => {
                    fields.mark(0, field)?;
                    model
                        .chunks
                        .push(ModelInfoChunk::new(parser.read()?, Vec::new()).into());
                    has_info = true;
                }
                "Sequences" => counted!(1, SequencesChunk),
                "GlobalSequences" => counted!(2, GlobalSequencesChunk),
                "Textures" => counted!(3, TexturesChunk),
                "Materials" => counted!(4, MaterialsChunk<V::Version>),
                "TextureAnims" => counted!(5, TextureAnimationsChunk),
                "PivotPoints" => counted!(6, PivotPointsChunk),
                "Geoset" => repeated!(Geosets, GeosetsChunk<V::Version>),
                "GeosetAnim" => repeated!(GeosetAnimations, GeosetAnimationsChunk),
                "Bone" => repeated!(Bones, BonesChunk),
                "Light" => repeated!(Lights, LightsChunk<V::Version>),
                "Helper" => repeated!(Helpers, HelpersChunk),
                "Attachment" => repeated!(Attachments, AttachmentsChunk),
                "ParticleEmitter" => repeated!(ParticleEmitters, ParticleEmittersChunk),
                "RibbonEmitter" => repeated!(RibbonEmitters, RibbonEmittersChunk),
                "EventObject" => repeated!(EventObjects, EventObjectsChunk),
                "CollisionShape" => repeated!(CollisionShapes, CollisionShapesChunk),
                "Glider" => repeated!(Gliders, GlidersChunk),
                "Camera" => repeated!(Cameras, CamerasChunk<V::Version>),
                "ParticleEmitter2" => repeated!(ParticleEmitters2, ParticleEmitters2Chunk),
                "ParticleEmitterPopcorn" if V::Version::NUMBER >= 900 => {
                    repeated!(PopcornEmitters, PopcornEmittersChunk)
                }
                "FaceFX" if V::Version::NUMBER >= 900 => repeated!(FaceFx, FaceFxChunk),
                "BindPose" if V::Version::NUMBER >= 900 => {
                    fields.mark(7, field)?;
                    model.chunks.push(parser.read::<BindPoseChunk>()?.into());
                }
                "FaceFX" | "BindPose" | "ParticleEmitterPopcorn" => {
                    return Err(mdl::ReadError::new(
                        field.span,
                        mdl::ReadErrorKind::UnsupportedField,
                    ))
                }
                _ => {
                    return Err(mdl::ReadError::new(
                        field.span,
                        mdl::ReadErrorKind::UnknownField,
                    ))
                }
            }
        }
        if !has_info {
            return Err(parser.error(mdl::ReadErrorKind::MissingField("Model")));
        }
        Ok(model)
    }
}

fn write_collection<V: ModelDialect, C: CollectionChunk>(
    model: &Model<V>,
    name: Option<&str>,
    writer: &mut Writer<impl IoWrite>,
) -> Result<(), IoError<mdl::WriteError>>
where
    C::Item: mdl::Write,
    for<'a> &'a C: TryFrom<&'a ModelChunk<V>>,
{
    let count = model
        .decoded_chunks::<C>()
        .try_fold(0usize, |total, chunk| {
            total.checked_add(chunk.records().len())
        })
        .ok_or(mdl::WriteError::SizeOverflow {
            field: "collection count overflow",
        })?;
    if count == 0 {
        return Ok(());
    }
    if let Some(name) = name {
        writer.begin_counted_block(name, count)?;
    }
    for chunk in model.decoded_chunks::<C>() {
        for record in chunk.records() {
            writer.write(record)?;
        }
    }
    if name.is_some() {
        writer.end_block()?;
    }
    Ok(())
}

impl<V: ModelDialect> mdl::Write for Model<V> {
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        let mut version = None;
        let mut info = None;
        for chunk in self.chunks.as_slice() {
            match chunk {
                ModelChunk::Version(chunk) => {
                    if version.replace(chunk).is_some() {
                        return Err(mdl::WriteError::InvalidStructure {
                            field: "multiple Version chunks",
                        }
                        .into());
                    }
                    chunk.validate_mdl_write()?;
                }
                ModelChunk::ModelInfo(chunk) => {
                    if info.replace(&chunk.info).is_some() {
                        return Err(mdl::WriteError::InvalidStructure {
                            field: "multiple Model chunks",
                        }
                        .into());
                    }
                    if !chunk.extension.is_empty() {
                        return Err(mdl::WriteError::Unrepresentable {
                            field: "model information chunk extension",
                        }
                        .into());
                    }
                }
                ModelChunk::Unknown(_) => {
                    return Err(mdl::WriteError::Unrepresentable {
                        field: "opaque binary chunk",
                    }
                    .into())
                }
                ModelChunk::Extension(_) => {
                    return Err(mdl::WriteError::Unrepresentable {
                        field: "application binary chunk",
                    }
                    .into())
                }
                ModelChunk::PopcornEmitters(chunk) => {
                    if V::Version::NUMBER < 900 && !chunk.records.is_empty() {
                        return Err(mdl::WriteError::Unrepresentable {
                            field: "ParticleEmitterPopcorn before version 900",
                        }
                        .into());
                    }
                }
                ModelChunk::FaceFx(chunk) => {
                    if V::Version::NUMBER < 900 && !chunk.records.is_empty() {
                        return Err(mdl::WriteError::Unrepresentable {
                            field: "FaceFX before version 900",
                        }
                        .into());
                    }
                }
                ModelChunk::BindPose(chunk) => {
                    if V::Version::NUMBER < 900 && !chunk.records.is_empty() {
                        return Err(mdl::WriteError::Unrepresentable {
                            field: "BindPose before version 900",
                        }
                        .into());
                    }
                }
                ModelChunk::Cameras(_)
                | ModelChunk::ParticleEmitters2(_)
                | ModelChunk::Sequences(_)
                | ModelChunk::GlobalSequences(_)
                | ModelChunk::Textures(_)
                | ModelChunk::Materials(_)
                | ModelChunk::Geosets(_)
                | ModelChunk::GeosetAnimations(_)
                | ModelChunk::Bones(_)
                | ModelChunk::Helpers(_)
                | ModelChunk::Attachments(_)
                | ModelChunk::EventObjects(_)
                | ModelChunk::CollisionShapes(_)
                | ModelChunk::ParticleEmitters(_)
                | ModelChunk::RibbonEmitters(_)
                | ModelChunk::Lights(_)
                | ModelChunk::TextureAnimations(_)
                | ModelChunk::PivotPoints(_)
                | ModelChunk::Gliders(_) => {}
            }
        }
        let version = version.ok_or(mdl::WriteError::InvalidStructure {
            field: "missing Version chunk",
        })?;
        let info = info.ok_or(mdl::WriteError::InvalidStructure {
            field: "missing Model chunk",
        })?;
        writer.write(version.as_ref())?;
        writer.write(info)?;
        macro_rules! collection {
            ($chunk:ty, $name:expr) => {
                write_collection::<V, $chunk>(self, $name, writer)?
            };
        }
        collection!(SequencesChunk, Some("Sequences"));
        collection!(GlobalSequencesChunk, Some("GlobalSequences"));
        collection!(TexturesChunk, Some("Textures"));
        collection!(MaterialsChunk<V::Version>, Some("Materials"));
        collection!(TextureAnimationsChunk, Some("TextureAnims"));
        collection!(GeosetsChunk<V::Version>, None);
        collection!(GeosetAnimationsChunk, None);
        collection!(BonesChunk, None);
        collection!(LightsChunk<V::Version>, None);
        collection!(HelpersChunk, None);
        collection!(AttachmentsChunk, None);
        collection!(PivotPointsChunk, Some("PivotPoints"));
        collection!(ParticleEmittersChunk, None);
        collection!(ParticleEmitters2Chunk, None);
        collection!(RibbonEmittersChunk, None);
        collection!(PopcornEmittersChunk, None);
        collection!(EventObjectsChunk, None);
        collection!(CamerasChunk<V::Version>, None);
        collection!(CollisionShapesChunk, None);
        collection!(FaceFxChunk, None);
        if self
            .decoded_chunks::<BindPoseChunk>()
            .any(|chunk| !chunk.records.is_empty())
        {
            writer.begin_block("BindPose")?;
            collection!(BindPoseChunk, Some("Matrices"));
            writer.end_block()?;
        }
        collection!(GlidersChunk, None);
        Ok(())
    }
}

#[derive(mdl::Read)]
#[mdl(block = "Version")]
struct VersionHeader {
    #[mdl(property = "FormatVersion")]
    version: u32,
}
impl<E: ModelExtension> mdl::Read for DynamicModel<E> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        require_version_first(parser)?;
        let start = parser.position();
        let mut checkpoint = *parser;
        let version = checkpoint.read::<VersionHeader>()?.version;
        macro_rules! dispatch { ($($number:literal => $variant:ident($ty:ty)),*) => { match version {
            $($number => parser.read::<Model<Extended<$ty, E>>>().map(Self::$variant),)*
            _ => Err(mdl::ReadError::new(Span::new(start, checkpoint.position()), mdl::ReadErrorKind::UnsupportedVersion { version })),
        } }; }
        dispatch!(800 => V800(V800), 900 => V900(V900), 1000 => V1000(V1000), 1100 => V1100(V1100), 1200 => V1200(V1200), 1300 => V1300(V1300), 1400 => V1400(V1400), 1600 => V1600(V1600), 1800 => V1800(V1800))
    }
}
impl<E: ModelExtension> mdl::Write for DynamicModel<E> {
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        use crate::model::visit_model;
        visit_model!(self, |model| writer.write(model))
    }
}

impl<V: ModelDialect> FromStr for Model<V> {
    type Err = mdl::ReadError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::decode_mdl(source)
    }
}

impl<E: ModelExtension> FromStr for DynamicModel<E> {
    type Err = mdl::ReadError;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        Self::decode_mdl(source)
    }
}
