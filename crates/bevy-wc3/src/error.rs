//! Errors from semantic compilation and native asset loading.
use bevy::asset::{AssetPath, ReadAssetBytesError};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io;
use wc3::model::io::asset::{Error as NativeAssetError, Request};

/// A model decoding, strict conversion, or geometry preparation failure.
#[derive(Debug)]
pub struct ModelError(pub(crate) String);

impl ModelError {
    /// Construct a model error from an application resource provider or compiler.
    pub fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for ModelError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(&self.0)
    }
}

impl Error for ModelError {}

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
