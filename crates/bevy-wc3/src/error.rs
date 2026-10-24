//! Errors from semantic compilation and native asset loading.
use crate::TextureInput;
use bevy::asset::{AssetPath, ReadAssetBytesError};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io;
use wc3::model::io::asset::{Error as NativeAssetError, Request};
use wc3::model::{ImageResource, ImageResourceId, ResourceLocator};

/// The preparation operation that failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PreparationOperation {
    /// Check authored model data or application selections.
    Validate,
    /// Compile a semantic model into renderer definitions.
    Compile,
    /// Project selected geometry into renderer meshes.
    Geometry,
    /// Resolve prepared child models or particle programmes.
    Dependencies,
    /// Read encoded image bytes.
    ImageRead,
    /// Decode an image or interpret its pixel format.
    ImageDecode,
    /// Compose decoded images into an atlas.
    Atlas,
    /// Bind compiled model components into an assembly.
    Compose,
}

/// An inspectable location or operation attached to a preparation failure.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PreparationContext {
    /// Preparation operation.
    Operation(PreparationOperation),
    /// Root or directly compiled assembly model.
    Model(ResourceLocator<'static>),
    /// Zero-based part position in the application assembly plan.
    Part {
        /// Position in the original ordered part list.
        index: usize,
        /// Resource identifying the component model.
        model: ResourceLocator<'static>,
    },
    /// Semantic image selected by the application or native adapter.
    Image(ImageResource<'static>),
    /// Resolved Bevy asset path, including its named source.
    Asset(AssetPath<'static>),
    /// Source-model image dependency selected for replacement.
    ImageDependency(ImageResourceId),
    /// Structural material or emitter input selected for replacement.
    ImageInput(TextureInput),
}

impl Display for PreparationContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Operation(operation) => f.write_str(match operation {
                PreparationOperation::Validate => "validate model selections",
                PreparationOperation::Compile => "compile model",
                PreparationOperation::Geometry => "compile geometry",
                PreparationOperation::Dependencies => "resolve model dependencies",
                PreparationOperation::ImageRead => "read image",
                PreparationOperation::ImageDecode => "decode image",
                PreparationOperation::Atlas => "compose atlas",
                PreparationOperation::Compose => "compose assembly",
            }),
            Self::Model(model) => write!(f, "model {model:?}"),
            Self::Part { index, model } => write!(f, "assembly part {index} ({model:?})"),
            Self::Image(image) => write!(f, "image {image:?}"),
            Self::Asset(path) => write!(f, "asset {path}"),
            Self::ImageDependency(id) => write!(f, "image dependency {id:?}"),
            Self::ImageInput(input) => write!(f, "image input {input:?}"),
        }
    }
}

/// A renderer preparation failure retaining its typed cause and context.
///
/// Inspect [`Self::contexts`] for operations and source identities, then follow
/// [`Error::source`] to the original validation, image, atlas or provider error.
/// Display text is explanatory and is not a stable classification interface.
#[derive(Debug)]
pub struct ModelError {
    context: Vec<PreparationContext>,
    source: Box<dyn Error + Send + Sync>,
}

impl ModelError {
    /// Describe an authored contract failure that has no underlying error value.
    /// Use [`Self::from_source`] when an operation already returned an error.
    pub fn new(message: impl Into<String>) -> Self {
        Self::from_source(MessageError(message.into()))
    }

    /// Retain an existing error without formatting or replacing its typed cause.
    pub fn from_source(source: impl Error + Send + Sync + 'static) -> Self {
        Self {
            context: Vec::new(),
            source: Box::new(source),
        }
    }

    /// Add an outer preparation location or operation without changing the cause.
    pub fn with_context(mut self, context: PreparationContext) -> Self {
        self.context.push(context);
        self
    }

    /// Attached context, from the innermost operation to the outermost caller.
    pub fn contexts(&self) -> &[PreparationContext] {
        &self.context
    }
}

impl Display for ModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        for context in self.context.iter().rev() {
            write!(f, "{context}: ")?;
        }
        Display::fmt(&self.source, f)
    }
}

impl Error for ModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[derive(Debug)]
struct MessageError(String);
impl Display for MessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}
impl Error for MessageError {}

/// A native asset load failure retaining document identities and source errors.
#[derive(Debug)]
pub enum ModelLoadError {
    /// The root asset reader failed before native decoding.
    RootRead {
        /// Bevy path, including its registered source.
        owner: AssetPath<'static>,
        /// Underlying reader failure.
        source: io::Error,
    },
    /// A native document could not be decoded or imported.
    Native {
        /// Bevy path of the root model.
        owner: AssetPath<'static>,
        /// Structured codec, import, or loading protocol failure.
        source: Box<NativeAssetError>,
    },
    /// A required companion candidate could not be accessed.
    CompanionRead {
        /// Bevy path of the root model.
        owner: AssetPath<'static>,
        /// Required native document and its declaring owner.
        request: Box<Request>,
        /// Candidate whose read failed.
        path: Box<AssetPath<'static>>,
        /// Underlying source, metadata, or reader failure.
        source: Box<ReadAssetBytesError>,
    },
    /// None of the provider's paths supplied a required native companion.
    MissingCompanion {
        /// Bevy path of the root model.
        owner: AssetPath<'static>,
        /// Required native document and its declaring owner.
        request: Box<Request>,
        /// Provider candidates, in attempted order.
        candidates: Vec<AssetPath<'static>>,
    },
    /// WMO decoding requires explicit client layout provenance.
    MissingWmoProfile {
        /// Bevy path whose provider omitted provenance.
        owner: AssetPath<'static>,
    },
    /// The loader does not accept the root asset's extension.
    UnsupportedExtension {
        /// Bevy path with an unsupported extension.
        owner: AssetPath<'static>,
        /// Lowercase extension, or an empty string when absent.
        extension: String,
    },
    /// The semantic model or its optional rendering resources could not compile.
    Compile {
        /// Bevy path of the root model.
        owner: AssetPath<'static>,
        /// Renderer or resource preparation failure.
        source: ModelError,
    },
}

impl Display for ModelLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::RootRead { owner, source } => write!(f, "cannot read model {owner}: {source}"),
            Self::Native { owner, source } => write!(f, "cannot load native model {owner}: {source}"),
            Self::CompanionRead { owner, request, path, source } => write!(
                f,
                "cannot read required model companion {:?} ({:?}, declared by {:?}) for {owner} at {path}: {source}",
                request.key, request.kind, request.owner,
            ),
            Self::MissingCompanion { owner, request, candidates } => write!(
                f,
                "unresolved required model companion {:?} ({:?}, declared by {:?}) for {owner}; tried {candidates:?}",
                request.key, request.kind, request.owner,
            ),
            Self::MissingWmoProfile { owner } => write!(
                f,
                "WMO loading for {owner} requires explicit client layout provenance from ResourceResolver::wmo_profile",
            ),
            Self::UnsupportedExtension { owner, extension } => write!(
                f,
                "native model loading for {owner} is not implemented for .{extension}",
            ),
            Self::Compile { owner, source } => write!(f, "cannot prepare model {owner}: {source}"),
        }
    }
}

impl Error for ModelLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RootRead { source, .. } => Some(source),
            Self::Native { source, .. } => Some(source.as_ref()),
            Self::CompanionRead { source, .. } => Some(source.as_ref()),
            Self::Compile { source, .. } => Some(source),
            Self::MissingCompanion { .. }
            | Self::MissingWmoProfile { .. }
            | Self::UnsupportedExtension { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::AtlasError;

    #[test]
    fn asset_failure_retains_preparation_context_and_typed_cause() {
        let error = ModelLoadError::Compile {
            owner: AssetPath::from("source://body.m2"),
            source: ModelError::from_source(AtlasError::PixelCount)
                .with_context(PreparationContext::Operation(PreparationOperation::Atlas))
                .with_context(PreparationContext::Part {
                    index: 3,
                    model: ResourceLocator::FileDataId(17),
                }),
        };
        let preparation = error
            .source()
            .unwrap()
            .downcast_ref::<ModelError>()
            .unwrap();
        assert!(matches!(
            preparation.contexts(),
            [
                PreparationContext::Operation(PreparationOperation::Atlas),
                PreparationContext::Part {
                    index: 3,
                    model: ResourceLocator::FileDataId(17)
                }
            ]
        ));
        assert_eq!(
            preparation.source().unwrap().downcast_ref::<AtlasError>(),
            Some(&AtlasError::PixelCount)
        );
        assert!(error.to_string().contains("source://body.m2"));
    }
}
