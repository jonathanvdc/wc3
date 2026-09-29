//! Explicit conversions between typed MDX layouts.
use crate::model::visit_model;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::model::{
    Camera, CamerasChunk, DynamicModel, Geoset, GeosetsChunk, Layer, Light, LightsChunk, Material,
    MaterialsChunk, Model, ModelChunk, ModelVersion, UnknownChunk, Version, VersionChunk,
};

/// Whether conversion may discard data that the target cannot represent.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LossPolicy {
    #[default]
    Reject,
    Drop,
}

/// Handling of opaque chunks when changing versions.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UnknownChunkPolicy {
    #[default]
    Reject,
    /// Retain exact bytes without guaranteeing target-version compatibility.
    Preserve,
    Drop,
}

/// Conversion policies. Unknown chunks require a separate explicit choice.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConversionOptions {
    pub loss_policy: LossPolicy,
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
    pub path: String,
    pub kind: ConversionIssueKind,
    pub description: String,
}
/// Changes and compatibility caveats to inspect after a successful conversion.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConversionReport {
    pub issues: Vec<ConversionIssue>,
}
/// A converted model or record together with the changes made to produce it.
#[derive(Clone, Debug)]
pub struct Conversion<T> {
    pub model: T,
    pub report: ConversionReport,
}

/// The first field or chunk that cannot be converted under the selected policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionError {
    pub source_version: Version,
    pub target_version: Version,
    pub path: String,
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
        impl<V: ModelVersion> $record<V> {
            /// Converts this record without modifying it. Neutral defaults may be omitted.
            pub fn convert<T: ModelVersion>(&self, options: &ConversionOptions) -> Result<Conversion<$record<T>>, ConversionError> {
                let mut context = ConversionContext::new::<V, T>(options);
                let model = self.convert_with::<T>(&mut context, "record")?;
                Ok(Conversion { model, report: context.report })
            }
        }
    )+};
}
record_conversion!(Material, Layer, Geoset, Light, Camera);

impl<V: ModelVersion> Model<V> {
    /// Converts ordered chunks to another layout without modifying the source.
    ///
    /// ```
    /// use wc3::model::{ConversionOptions, Model, V800, V1100};
    /// let source = Model::<V800>::new();
    /// let converted = source.convert::<V1100>(&ConversionOptions::strict())?;
    /// assert_eq!(converted.model.version(), 1100);
    /// # Ok::<(), wc3::model::ConversionError>(())
    /// ```
    ///
    /// Shared fields retain exact storage, except equivalent camera variants 0/3
    /// are normalized to the target default and reported. New fields use constructor defaults.
    /// Unsupported non-default fields and tracks fail unless dropping is enabled.
    /// A missing VERS is inserted so the encoded result identifies its target layout.
    pub fn convert<T: ModelVersion>(
        &self,
        options: &ConversionOptions,
    ) -> Result<Conversion<Model<T>>, ConversionError> {
        let mut context = ConversionContext::new::<V, T>(options);
        let mut model = Model::<T>::new();
        model.chunks.clear();
        for (index, chunk) in self.chunks.iter().enumerate() {
            let path = format!("chunks[{index}]");
            if T::NUMBER < 900
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
                        .iter()
                        .enumerate()
                        .map(|(i, record)| {
                            record
                                .convert_with::<T>(&mut context, &format!("{path}.{}[{i}]", $name))
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    $collection::<T>::new(records).into()
                }};
            }
            let converted = match chunk {
                ModelChunk::Version(value) => {
                    let mut version = VersionChunk::<T>::new();
                    version.extension = value.extension.clone();
                    version.into()
                }
                ModelChunk::Materials(value) => records!(value, MaterialsChunk, "materials"),
                ModelChunk::Geosets(value) => records!(value, GeosetsChunk, "geosets"),
                ModelChunk::Lights(value) => records!(value, LightsChunk, "lights"),
                ModelChunk::Cameras(value) => records!(value, CamerasChunk, "cameras"),
                ModelChunk::Unknown(value) => {
                    if V::NUMBER != T::NUMBER {
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
                    ModelChunk::Unknown(
                        UnknownChunk::<T>::new(value.raw().clone())
                            .expect("unknown tag remains unknown"),
                    )
                }
                ModelChunk::ModelInfo(value) => ModelChunk::ModelInfo(value.clone()),
                ModelChunk::Sequences(value) => ModelChunk::Sequences(value.clone()),
                ModelChunk::GlobalSequences(value) => ModelChunk::GlobalSequences(value.clone()),
                ModelChunk::Textures(value) => ModelChunk::Textures(value.clone()),
                ModelChunk::GeosetAnimations(value) => ModelChunk::GeosetAnimations(value.clone()),
                ModelChunk::Bones(value) => ModelChunk::Bones(value.clone()),
                ModelChunk::Helpers(value) => ModelChunk::Helpers(value.clone()),
                ModelChunk::Attachments(value) => ModelChunk::Attachments(value.clone()),
                ModelChunk::EventObjects(value) => ModelChunk::EventObjects(value.clone()),
                ModelChunk::CollisionShapes(value) => ModelChunk::CollisionShapes(value.clone()),
                ModelChunk::ParticleEmitters(value) => ModelChunk::ParticleEmitters(value.clone()),
                ModelChunk::ParticleEmitters2(value) => {
                    ModelChunk::ParticleEmitters2(value.clone())
                }
                ModelChunk::RibbonEmitters(value) => ModelChunk::RibbonEmitters(value.clone()),
                ModelChunk::PopcornEmitters(value) => ModelChunk::PopcornEmitters(value.clone()),
                ModelChunk::TextureAnimations(value) => {
                    ModelChunk::TextureAnimations(value.clone())
                }
                ModelChunk::FaceFx(value) => ModelChunk::FaceFx(value.clone()),
                ModelChunk::PivotPoints(value) => ModelChunk::PivotPoints(value.clone()),
                ModelChunk::BindPose(value) => ModelChunk::BindPose(value.clone()),
                ModelChunk::Gliders(value) => ModelChunk::Gliders(value.clone()),
            };
            model.chunks.push(converted);
        }
        if model.chunk(*b"VERS").is_none() {
            model.chunks.insert(0, VersionChunk::<T>::new().into());
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

impl DynamicModel {
    /// Converts a runtime-dispatched source to a typed target layout.
    pub fn convert<T: ModelVersion>(
        &self,
        options: &ConversionOptions,
    ) -> Result<Conversion<Model<T>>, ConversionError> {
        visit_model!(self, |model| model.convert::<T>(options))
    }
}
