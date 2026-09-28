//! Whole-model assembly: top-level MDL blocks map to typed binary chunks.
use super::{Field, Fields, MdlWriter, Parser, ReadErrorKind, Span, TokenKind};
use crate::model::mdl;
use crate::model::{
    AttachmentsChunk, BindPoseChunk, BonesChunk, CollectionChunk, CollisionShapesChunk,
    DynamicModel, EventObjectsChunk, FaceFxChunk, GeosetAnimationsChunk, GeosetsChunk,
    GlidersChunk, GlobalSequencesChunk, HelpersChunk, LightsChunk, MaterialsChunk, Model,
    ModelChunk, ModelInfoChunk, ModelVersion, ParticleEmittersChunk, PivotPointsChunk,
    RibbonEmittersChunk, SequencesChunk, TextureAnimationsChunk, TexturesChunk, VersionChunk,
    V1000, V1100, V1200, V1300, V1400, V1600, V1800, V800, V900,
};
use std::io::Write as IoWrite;

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
                ReadErrorKind::VersionMismatch {
                    expected: V::NUMBER,
                    actual,
                },
            ));
        }
        Ok(())
    }
    pub(crate) fn validate_mdl_write(&self) -> Result<(), mdl::WriteError> {
        if !self.extension.is_empty() {
            return Err(mdl::WriteError::Unsupported("version chunk extension"));
        }
        Ok(())
    }
}

fn require_version_first(parser: &mut Parser<'_>) -> Result<(), mdl::ReadError> {
    match parser.peek()? {
        Some(token) if token.kind == TokenKind::Ident("Version") => Ok(()),
        _ => Err(parser.error(ReadErrorKind::Expected("Version as the first block"))),
    }
}
fn read_collection<C: CollectionChunk>(parser: &mut Parser<'_>) -> Result<C, mdl::ReadError>
where
    C::Item: mdl::Read,
{
    let records = parser.counted::<C::Item>()?.collect::<Result<_, _>>()?;
    Ok(C::from_records(records))
}

impl<V: ModelVersion> mdl::Read for Model<V> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        require_version_first(parser)?;
        parser.read::<VersionChunk<V>>()?;
        let mut model = Self::new();
        let mut fields = Fields::default();
        let mut has_info = false;
        while let Some(token) = parser.peek()? {
            let name = match token.kind {
                TokenKind::Ident(name) => name,
                _ => return Err(parser.error(ReadErrorKind::TrailingInput)),
            };
            let field = Field {
                name,
                span: token.span,
            };
            macro_rules! counted {
                ($bit:expr, $chunk:ty) => {{
                    fields.mark($bit, field)?;
                    parser.next_token()?;
                    model.push(read_collection::<$chunk>(parser)?.into());
                }};
            }
            macro_rules! repeated {
                ($variant:ident, $chunk:ty) => {{
                    let record = parser.read()?;
                    if let Some(chunk) = model.chunks_mut().iter_mut().find_map(|chunk| match chunk
                    {
                        ModelChunk::$variant(chunk) => Some(chunk),
                        _ => None,
                    }) {
                        chunk.records.push(record);
                    } else {
                        model.push(<$chunk>::from_records(vec![record]).into());
                    }
                }};
            }
            match name {
                "Version" => {
                    return Err(mdl::ReadError::new(
                        field.span,
                        ReadErrorKind::DuplicateField,
                    ))
                }
                "Model" => {
                    fields.mark(0, field)?;
                    model.push(ModelInfoChunk::new(parser.read()?, Vec::new()).into());
                    has_info = true;
                }
                "Sequences" => counted!(1, SequencesChunk),
                "GlobalSequences" => counted!(2, GlobalSequencesChunk),
                "Textures" => counted!(3, TexturesChunk),
                "Materials" => counted!(4, MaterialsChunk<V>),
                "TextureAnims" => counted!(5, TextureAnimationsChunk),
                "PivotPoints" => counted!(6, PivotPointsChunk),
                "Geoset" => repeated!(Geosets, GeosetsChunk<V>),
                "GeosetAnim" => repeated!(GeosetAnimations, GeosetAnimationsChunk),
                "Bone" => repeated!(Bones, BonesChunk),
                "Light" => repeated!(Lights, LightsChunk<V>),
                "Helper" => repeated!(Helpers, HelpersChunk),
                "Attachment" => repeated!(Attachments, AttachmentsChunk),
                "ParticleEmitter" => repeated!(ParticleEmitters, ParticleEmittersChunk),
                "RibbonEmitter" => repeated!(RibbonEmitters, RibbonEmittersChunk),
                "EventObject" => repeated!(EventObjects, EventObjectsChunk),
                "CollisionShape" => repeated!(CollisionShapes, CollisionShapesChunk),
                "Glider" => repeated!(Gliders, GlidersChunk),
                "FaceFX" if V::NUMBER >= 900 => repeated!(FaceFx, FaceFxChunk),
                "BindPose" if V::NUMBER >= 900 => {
                    fields.mark(7, field)?;
                    model.push(parser.read::<BindPoseChunk>()?.into());
                }
                "FaceFX"
                | "BindPose"
                | "Camera"
                | "ParticleEmitter2"
                | "ParticleEmitterPopcorn" => {
                    return Err(mdl::ReadError::new(
                        field.span,
                        ReadErrorKind::UnsupportedField,
                    ))
                }
                _ => return Err(mdl::ReadError::new(field.span, ReadErrorKind::UnknownField)),
            }
        }
        if !has_info {
            return Err(parser.error(ReadErrorKind::MissingField("Model")));
        }
        Ok(model)
    }
}

fn write_collection<V: ModelVersion, C: CollectionChunk>(
    model: &Model<V>,
    name: Option<&str>,
    writer: &mut MdlWriter<impl IoWrite>,
) -> Result<(), mdl::WriteError>
where
    C::Item: mdl::Write,
    for<'a> &'a C: TryFrom<&'a ModelChunk<V>>,
{
    let count = model
        .decoded_chunks::<C>()
        .try_fold(0usize, |total, chunk| {
            total.checked_add(chunk.records().len())
        })
        .ok_or(mdl::WriteError::Unsupported("collection count overflow"))?;
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

impl<V: ModelVersion> mdl::Write for Model<V> {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        let mut version = None;
        let mut info = None;
        for chunk in self.chunks() {
            match chunk {
                ModelChunk::Version(chunk) => {
                    if version.replace(chunk).is_some() {
                        return Err(mdl::WriteError::Unsupported("multiple Version chunks"));
                    }
                    chunk.validate_mdl_write()?;
                }
                ModelChunk::ModelInfo(chunk) => {
                    if info.replace(&chunk.info).is_some() {
                        return Err(mdl::WriteError::Unsupported("multiple Model chunks"));
                    }
                    if !chunk.extension.is_empty() {
                        return Err(mdl::WriteError::Unsupported(
                            "model information chunk extension",
                        ));
                    }
                }
                ModelChunk::Unknown(_) => {
                    return Err(mdl::WriteError::Unsupported("opaque binary chunk"))
                }
                ModelChunk::Cameras(chunk) => {
                    if !chunk.records.is_empty() {
                        return Err(mdl::WriteError::Unsupported("Camera record codec"));
                    }
                }
                ModelChunk::ParticleEmitters2(chunk) => {
                    if !chunk.records.is_empty() {
                        return Err(mdl::WriteError::Unsupported(
                            "ParticleEmitter2 record codec",
                        ));
                    }
                }
                ModelChunk::PopcornEmitters(chunk) => {
                    if !chunk.records.is_empty() {
                        return Err(mdl::WriteError::Unsupported(
                            "ParticleEmitterPopcorn record codec",
                        ));
                    }
                }
                ModelChunk::FaceFx(chunk) => {
                    if V::NUMBER < 900 && !chunk.records.is_empty() {
                        return Err(mdl::WriteError::Unsupported("FaceFX before version 900"));
                    }
                }
                ModelChunk::BindPose(chunk) => {
                    if V::NUMBER < 900 && !chunk.records.is_empty() {
                        return Err(mdl::WriteError::Unsupported("BindPose before version 900"));
                    }
                }
                ModelChunk::Sequences(_)
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
        let version = version.ok_or(mdl::WriteError::Unsupported("missing Version chunk"))?;
        let info = info.ok_or(mdl::WriteError::Unsupported("missing Model chunk"))?;
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
        collection!(MaterialsChunk<V>, Some("Materials"));
        collection!(TextureAnimationsChunk, Some("TextureAnims"));
        collection!(GeosetsChunk<V>, None);
        collection!(GeosetAnimationsChunk, None);
        collection!(BonesChunk, None);
        collection!(LightsChunk<V>, None);
        collection!(HelpersChunk, None);
        collection!(AttachmentsChunk, None);
        collection!(PivotPointsChunk, Some("PivotPoints"));
        collection!(ParticleEmittersChunk, None);
        collection!(RibbonEmittersChunk, None);
        collection!(EventObjectsChunk, None);
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
impl mdl::Read for DynamicModel {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        require_version_first(parser)?;
        let start = parser.position();
        let mut checkpoint = *parser;
        let version = checkpoint.read::<VersionHeader>()?.version;
        macro_rules! dispatch { ($($number:literal => $variant:ident($ty:ty)),*) => { match version {
            $($number => parser.read::<Model<$ty>>().map(Self::$variant),)*
            _ => Err(mdl::ReadError::new(Span::new(start, checkpoint.position()), ReadErrorKind::UnsupportedVersion { version })),
        } }; }
        dispatch!(800 => V800(V800), 900 => V900(V900), 1000 => V1000(V1000), 1100 => V1100(V1100), 1200 => V1200(V1200), 1300 => V1300(V1300), 1400 => V1400(V1400), 1600 => V1600(V1600), 1800 => V1800(V1800))
    }
}
impl mdl::Write for DynamicModel {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        use crate::model::visit_model;
        visit_model!(self, |model| writer.write(model))
    }
}
