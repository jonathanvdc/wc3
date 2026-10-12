//! Explicit conversions between typed MDX layouts.
use crate::model::mdlx::mdx;
use crate::model::mdlx::scene::{set_node_kind, Node, NodeFlagInterpretation};
use crate::model::mdlx::visit_model;
use crate::model::mdlx::{Chunk, Encoder, ModelDialect, RawChunk};
use std::error::Error;
use std::fmt::Debug;
use std::fmt::{self, Display, Formatter};

use crate::model::mdlx::{
    Camera, CamerasChunk, DynamicModel, Geoset, GeosetsChunk, Layer, Light, LightsChunk, Material,
    MaterialsChunk, Model, ModelChunk, ModelVersion, Version, VersionChunk,
};

/// Whether conversion may discard data that the target cannot represent.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LossPolicy {
    #[default]
    /// Rejects conversion that would lose nondefault data.
    Reject,
    /// Allows unsupported data to be dropped and records it in the report.
    Drop,
}

/// Handling of opaque chunks when changing versions.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UnknownChunkPolicy {
    #[default]
    /// Rejects opaque chunks when changing model versions.
    Reject,
    /// Retain exact bytes without guaranteeing target-version compatibility.
    Preserve,
    /// Drops opaque chunks when changing model versions.
    Drop,
}

/// Conversion policies. Unknown chunks require a separate explicit choice.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConversionOptions {
    /// Handling of fields and records unsupported by the target version.
    pub loss_policy: LossPolicy,
    /// Handling of opaque chunks when changing versions.
    pub unknown_chunks: UnknownChunkPolicy,
}
impl ConversionOptions {
    /// Rejects unsupported nondefault data and opaque chunks when changing versions.
    pub fn strict() -> Self {
        Self::default()
    }
    /// Allows unsupported data to be dropped; opaque chunks still require a choice.
    pub fn lossy() -> Self {
        Self {
            loss_policy: LossPolicy::Drop,
            ..Self::default()
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Classification of a change or caveat produced by conversion.
pub enum ConversionIssueKind {
    /// Equivalent storage was normalized to the target layout without losing values.
    Normalized,
    /// A target-only field was initialized with its documented default.
    Initialized,
    /// A neutral source field was omitted because the target has no storage for it.
    OmittedDefault,
    /// Data was discarded with the caller's permission.
    Dropped,
    /// Opaque bytes were preserved without a compatibility guarantee.
    PreservedUnknown,
}

/// A change or compatibility caveat at a concrete source location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionIssue {
    /// Path identifying the affected chunk, record, or field.
    pub path: String,
    /// Classification of this change or caveat.
    pub kind: ConversionIssueKind,
    /// Human-readable explanation of the conversion result.
    pub description: String,
}
/// Changes and compatibility caveats to inspect after a successful conversion.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConversionReport {
    /// Changes and caveats in conversion traversal order.
    pub issues: Vec<ConversionIssue>,
}
/// A converted model or record together with the changes made to produce it.
#[derive(Clone, Debug)]
pub struct Conversion<T> {
    /// The converted model or record.
    pub model: T,
    /// Changes and caveats generated while producing the result.
    pub report: ConversionReport,
}

/// The first field or chunk that cannot be converted under the selected policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionError {
    /// Numeric version of the input model or record.
    pub source_version: Version,
    /// Requested numeric destination version.
    pub target_version: Version,
    /// Path identifying the field or chunk that prevented conversion.
    pub path: String,
    /// Human-readable reason conversion failed.
    pub description: String,
}
impl Display for ConversionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot convert MDX {} to {} at {}: {}",
            self.source_version, self.target_version, self.path, self.description
        )
    }
}
impl Error for ConversionError {}

pub(crate) struct ConversionContext<'a> {
    options: &'a ConversionOptions,
    source: Version,
    target: Version,
    pub(crate) report: ConversionReport,
}
impl<'a> ConversionContext<'a> {
    fn new<S: ModelVersion, T: ModelVersion>(options: &'a ConversionOptions) -> Self {
        Self {
            options,
            source: S::NUMBER,
            target: T::NUMBER,
            report: ConversionReport::default(),
        }
    }
    pub(crate) fn issue(&mut self, path: &str, kind: ConversionIssueKind, description: &str) {
        self.report.issues.push(ConversionIssue {
            path: path.into(),
            kind,
            description: description.into(),
        });
    }
    pub(crate) fn error(&self, path: &str, description: &str) -> ConversionError {
        ConversionError {
            source_version: self.source,
            target_version: self.target,
            path: path.into(),
            description: description.into(),
        }
    }
    pub(crate) fn drop(&mut self, path: &str, description: &str) -> Result<(), ConversionError> {
        if self.options.loss_policy == LossPolicy::Reject {
            return Err(self.error(path, description));
        }
        self.issue(path, ConversionIssueKind::Dropped, description);
        Ok(())
    }
    /// Absent target fields may omit neutral defaults without a loss.
    pub(crate) fn field<T: PartialEq>(
        &mut self,
        source: Option<T>,
        target: Option<&mut T>,
        neutral: T,
        path: &str,
    ) -> Result<(), ConversionError> {
        match (source, target) {
            (Some(value), Some(target)) => *target = value,
            (Some(value), None) if value != neutral => {
                self.drop(path, "field is not supported by the target")?
            }
            (None, Some(_)) => self.issue(
                path,
                ConversionIssueKind::Initialized,
                "initialized with the target field default",
            ),
            (Some(_), None) => self.issue(
                path,
                ConversionIssueKind::OmittedDefault,
                "omitted neutral default; field is not supported by the target",
            ),
            (None, None) => {}
        }
        Ok(())
    }
}

macro_rules! record_conversion {
    ($($record:ident),+) => {$(
        impl<'data, V: ModelVersion> $record<'data, V> {
            /// Converts this record without modifying it. Neutral defaults may be omitted.
            pub fn convert<'view,T: ModelVersion>(&'view self, options: &ConversionOptions) -> Result<Conversion<$record<'view, T>>, ConversionError> {
                let mut context = ConversionContext::new::<V, T>(options);
                let model = self.as_borrowed().convert_with::<T>(&mut context, "record")?;
                Ok(Conversion { model, report: context.report })
            }
        }
    )+};
}
record_conversion!(Material, Layer, Geoset, Light, Camera);

impl<'data, V: ModelDialect> Model<'data, V>
where
    V::Extension: Chunk,
{
    /// Returns a normalized copy and its report, preserving the model version.
    ///
    /// Equivalent camera variants and missing node kind bits are normalized.
    /// The source is unchanged. This is strict conversion to the same version;
    /// it does not guarantee that every binary value can be exported to MDL.
    pub fn normalized<'view>(&'view self) -> Result<Conversion<Model<'view, V>>, ConversionError>
    where
        V: 'static,
        V::Extension: for<'input> mdx::Extension<'input>,
    {
        self.convert::<V>(&ConversionOptions::strict())
    }

    /// Converts ordered chunks to another dialect without modifying the source.
    ///
    /// Application chunks are encoded by their chunk codec and offered to the
    /// target dialect's decoder. Unknown payloads are offered to it as well. Tags
    /// not recognized by the target remain opaque and follow `unknown_chunks` when
    /// the base version changes. Malformed recognized payloads fail conversion.
    ///
    /// ```
    /// use wc3::model::mdlx::{ConversionOptions, Model, V800, V1100};
    /// let source = Model::<V800>::new();
    /// let converted = source.convert::<V1100>(&ConversionOptions::strict())?;
    /// assert_eq!(converted.model.version(), 1100);
    /// # Ok::<(), wc3::model::mdlx::ConversionError>(())
    /// ```
    ///
    /// Shared fields retain exact storage, except equivalent camera variants 0/3
    /// are normalized to the target default and reported. Missing node kind bits
    /// are inferred from their record type and reported; conflicting kind bits are retained.
    /// New fields use constructor defaults.
    /// Unsupported non-default fields and tracks fail unless dropping is enabled.
    /// A missing VERS is inserted so the encoded result identifies its target layout.
    pub fn convert<'view, T: ModelDialect + 'static>(
        &'view self,
        options: &ConversionOptions,
    ) -> Result<Conversion<Model<'view, T>>, ConversionError>
    where
        T::Extension: for<'input> mdx::Extension<'input>,
    {
        self.as_borrowed().into_convert::<T>(options)
    }

    /// Consumes a model, moving unchanged chunks and compatible buffers.
    pub fn into_convert<T: ModelDialect + 'static>(
        self,
        options: &ConversionOptions,
    ) -> Result<Conversion<Model<'data, T>>, ConversionError>
    where
        T::Extension: for<'input> mdx::Extension<'input>,
    {
        let mut context = ConversionContext::new::<V::Version, T::Version>(options);
        let mut model = Model::<T> {
            chunks: Vec::with_capacity(self.chunks.len() + 1),
        };
        for (index, chunk) in self.chunks.into_iter().enumerate() {
            let path = format!("chunks[{index}]");
            if T::Version::NUMBER < 900
                && matches!(
                    chunk,
                    ModelChunk::BindPose(_)
                        | ModelChunk::FaceFx(_)
                        | ModelChunk::PopcornEmitters(_)
                )
            {
                context.drop(&path, "chunk is not supported by the target")?;
                continue;
            }
            macro_rules! records {
                ($value:expr, $collection:ident, $name:literal) => {{
                    let records = $value
                        .records
                        .into_iter()
                        .enumerate()
                        .map(|(i, record)| {
                            record.convert_with::<T::Version>(
                                &mut context,
                                &format!("{path}.{}[{i}]", $name),
                            )
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    $collection::<T::Version>::new(records).into()
                }};
            }
            let mut converted = match chunk {
                ModelChunk::Version(value) => {
                    let mut version = VersionChunk::<T::Version>::new();
                    version.extension = value.extension;
                    version.into()
                }
                ModelChunk::Materials(value) => records!(value, MaterialsChunk, "materials"),
                ModelChunk::Geosets(value) => records!(value, GeosetsChunk, "geosets"),
                ModelChunk::Lights(value) => records!(value, LightsChunk, "lights"),
                ModelChunk::Cameras(value) => records!(value, CamerasChunk, "cameras"),
                chunk @ (ModelChunk::Extension(_) | ModelChunk::Unknown(_)) => {
                    let raw = match chunk {
                        ModelChunk::Unknown(value) => value.into_raw(),
                        chunk => {
                            let mut payload = Vec::new();
                            chunk
                                .encode_payload_to(&mut Encoder::new(&mut payload))
                                .map_err(|error| {
                                    context.error(
                                        &path,
                                        &format!("extension payload could not be encoded: {error}"),
                                    )
                                })?;
                            RawChunk::new(chunk.tag(), payload)
                        }
                    };
                    let converted = ModelChunk::<T>::from_raw(raw).map_err(|error| {
                        context.error(
                            &path,
                            &format!(
                                "chunk payload is incompatible with the target dialect: {error}"
                            ),
                        )
                    })?;
                    if V::Version::NUMBER != T::Version::NUMBER
                        && matches!(converted, ModelChunk::Unknown(_))
                    {
                        match options.unknown_chunks {
                            UnknownChunkPolicy::Reject => {
                                return Err(
                                    context.error(&path, "opaque chunk compatibility is unknown")
                                )
                            }
                            UnknownChunkPolicy::Drop => {
                                context.issue(
                                    &path,
                                    ConversionIssueKind::Dropped,
                                    "dropped opaque chunk",
                                );
                                continue;
                            }
                            UnknownChunkPolicy::Preserve => context.issue(
                                &path,
                                ConversionIssueKind::PreservedUnknown,
                                "retained opaque bytes; target compatibility is unknown",
                            ),
                        }
                    }
                    converted
                }
                ModelChunk::ModelInfo(value) => ModelChunk::ModelInfo(value),
                ModelChunk::Sequences(value) => ModelChunk::Sequences(value),
                ModelChunk::GlobalSequences(value) => ModelChunk::GlobalSequences(value),
                ModelChunk::Textures(value) => ModelChunk::Textures(value),
                ModelChunk::GeosetAnimations(value) => ModelChunk::GeosetAnimations(value),
                ModelChunk::Bones(value) => ModelChunk::Bones(value),
                ModelChunk::Helpers(value) => ModelChunk::Helpers(value),
                ModelChunk::Attachments(value) => ModelChunk::Attachments(value),
                ModelChunk::EventObjects(value) => ModelChunk::EventObjects(value),
                ModelChunk::CollisionShapes(value) => ModelChunk::CollisionShapes(value),
                ModelChunk::ParticleEmitters(value) => ModelChunk::ParticleEmitters(value),
                ModelChunk::ParticleEmitters2(value) => ModelChunk::ParticleEmitters2(value),
                ModelChunk::RibbonEmitters(value) => ModelChunk::RibbonEmitters(value),
                ModelChunk::PopcornEmitters(value) => ModelChunk::PopcornEmitters(value),
                ModelChunk::TextureAnimations(value) => ModelChunk::TextureAnimations(value),
                ModelChunk::FaceFx(value) => ModelChunk::FaceFx(value),
                ModelChunk::PivotPoints(value) => ModelChunk::PivotPoints(value),
                ModelChunk::BindPose(value) => ModelChunk::BindPose(value),
                ModelChunk::Gliders(value) => ModelChunk::Gliders(value),
            };
            macro_rules! normalize_nodes {
                ($value:expr, $name:literal, $kind:expr) => {
                    for (i, record) in $value.records.iter_mut().enumerate() {
                        normalize_node_kind(
                            &mut record.node,
                            $kind,
                            &mut context,
                            &format!("{path}.{}[{i}].node.flags", $name),
                        );
                    }
                };
            }
            match &mut converted {
                ModelChunk::Bones(value) => normalize_nodes!(value, "bones", 0x100),
                ModelChunk::Lights(value) => normalize_nodes!(value, "lights", 0x200),
                ModelChunk::EventObjects(value) => normalize_nodes!(value, "event_objects", 0x400),
                ModelChunk::Attachments(value) => normalize_nodes!(value, "attachments", 0x800),
                ModelChunk::ParticleEmitters(value) => {
                    normalize_nodes!(value, "particle_emitters", 0x1000)
                }
                ModelChunk::ParticleEmitters2(value) => {
                    normalize_nodes!(value, "particle_emitters2", 0x1000)
                }
                ModelChunk::PopcornEmitters(value) => {
                    normalize_nodes!(value, "popcorn_emitters", 0x1000)
                }
                ModelChunk::CollisionShapes(value) => {
                    normalize_nodes!(value, "collision_shapes", 0x2000)
                }
                ModelChunk::RibbonEmitters(value) => {
                    normalize_nodes!(value, "ribbon_emitters", 0x4000)
                }
                _ => {}
            }
            model.chunks.push(converted);
        }
        if model.chunk(*b"VERS").is_none() {
            model
                .chunks
                .insert(0, VersionChunk::<T::Version>::new().into());
            context.issue(
                "VERS",
                ConversionIssueKind::Initialized,
                "inserted target version chunk",
            );
        }
        Ok(Conversion {
            model,
            report: context.report,
        })
    }
}

fn normalize_node_kind<'data, F: NodeFlagInterpretation>(
    node: &mut Node<'data, F>,
    kind: u32,
    context: &mut ConversionContext<'_>,
    path: &str,
) {
    // Only the seven object-kind bits are redundant with the enclosing record.
    // Transform, emitter behavior, and unknown bits must remain untouched.
    if node.flags.bits() & 0x7f00 == 0 {
        set_node_kind(node, kind);
        context.issue(
            path,
            ConversionIssueKind::Normalized,
            &format!("added missing node object-kind bit {kind:#x} inferred from record type"),
        );
    }
}

impl<'data, E: Clone + Debug + Chunk> DynamicModel<'data, E> {
    /// Returns a normalized copy and its report without changing the runtime version.
    /// Uses the same rules as [`Model::normalized`] and leaves the source unchanged.
    pub fn normalized<'view>(
        &'view self,
    ) -> Result<Conversion<DynamicModel<'view, E>>, ConversionError>
    where
        E: 'static + for<'input> mdx::Extension<'input>,
    {
        macro_rules! normalize {
            ($($variant:ident),+) => {
                match self {
                    $(Self::$variant(model) => {
                        let converted = model.normalized()?;
                        Ok(Conversion {
                            model: DynamicModel::$variant(converted.model),
                            report: converted.report,
                        })
                    }),+
                }
            };
        }
        normalize!(V800, V900, V1000, V1100, V1200, V1300, V1400, V1600, V1800)
    }

    /// Converts a runtime-dispatched source to a typed target layout.
    pub fn convert<'view, T: ModelDialect + 'static>(
        &'view self,
        options: &ConversionOptions,
    ) -> Result<Conversion<Model<'view, T>>, ConversionError>
    where
        T::Extension: for<'input> mdx::Extension<'input>,
    {
        visit_model!(self, |model| model.convert::<T>(options))
    }
    /// Converts the contained owned model without copying unchanged chunks.
    pub fn into_convert<T: ModelDialect + 'static>(
        self,
        options: &ConversionOptions,
    ) -> Result<Conversion<Model<'data, T>>, ConversionError>
    where
        T::Extension: for<'input> mdx::Extension<'input>,
    {
        visit_model!(self, |model| model.into_convert::<T>(options))
    }
}
