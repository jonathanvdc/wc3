use super::Request;
use crate::model::common::FileKey;
use crate::model::conversion;
use crate::model::formats::{
    m2,
    mdlx::{mdl, mdx},
    wmo,
};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::str::Utf8Error;

/// The original codec failure, retaining native offsets and diagnostics.
#[derive(Debug)]
pub enum DecodeError {
    /// Binary Warcraft III model decoding.
    Mdx(mdx::ReadError),
    /// Text Warcraft III model decoding.
    Mdl(mdl::ReadError),
    /// M2 model or supporting document decoding.
    M2(m2::ReadError),
    /// WMO root or group decoding.
    Wmo(wmo::ReadError),
    /// A text model contains invalid UTF-8.
    Utf8(Utf8Error),
}
impl Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mdx(e) => Display::fmt(e, f),
            Self::Mdl(e) => Display::fmt(e, f),
            Self::M2(e) => Display::fmt(e, f),
            Self::Wmo(e) => Display::fmt(e, f),
            Self::Utf8(e) => write!(f, "MDL source is not UTF-8: {e}"),
        }
    }
}
impl StdError for DecodeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match self {
            Self::Mdx(e) => e,
            Self::Mdl(e) => e,
            Self::M2(e) => e,
            Self::Wmo(e) => e,
            Self::Utf8(e) => e,
        })
    }
}
/// Native model loading, import, or session-protocol failure.
#[derive(Debug)]
pub enum Error {
    /// Native decoding failed for this root or companion identity.
    Decode {
        /// Root identity, or a companion identity when its decode site is known.
        key: FileKey,
        /// Original native codec or text-encoding failure.
        source: DecodeError,
    },
    /// Checked semantic import rejected native meaning.
    Import {
        /// Identity of the imported native root.
        key: FileKey,
        /// Original checked conversion failure.
        source: conversion::Error,
    },
    /// The selected WMO layout has no incremental asset adapter.
    UnsupportedProfile {
        /// Explicit caller-selected layout without a loader adapter.
        profile: wmo::Profile,
    },
    /// A caller supplied bytes for a request other than the pending one.
    UnexpectedCompanion {
        /// Current request; absent before polling or after completion.
        expected: Option<Box<Request>>,
        /// Request supplied by the caller.
        supplied: Box<Request>,
    },
    /// The completed reader has already returned its checked model.
    Finished,
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decode { key, source } => write!(f, "cannot decode native model {key:?}: {source}"),
            Self::Import { key, source } => write!(f, "cannot import native model {key:?}: {source}"),
            Self::UnsupportedProfile { profile } => write!(f, "incremental WMO loading is not implemented for {profile:?}; import its native bundle directly"),
            Self::UnexpectedCompanion { expected, supplied } => write!(f, "unexpected model companion {supplied:?}; pending request is {expected:?}"),
            Self::Finished => f.write_str("native model reader is already complete"),
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Decode { source, .. } => Some(source),
            Self::Import { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Native loading or caller-provided companion access failure.
///
/// The generic source retains application-specific storage errors without
/// converting them to strings or requiring a particular I/O implementation.
#[derive(Debug)]
pub enum LoadError<E> {
    /// Native decoding or loader protocol failed.
    Native(Error),
    /// The caller could not supply a required native companion.
    Companion {
        /// Full identity, declaring owner, and native role of the missing bytes.
        request: Request,
        /// Original application storage error.
        source: E,
    },
}
impl<E: Display> Display for LoadError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Native(source) => Display::fmt(source, f),
            Self::Companion { request, source } => write!(
                f,
                "cannot read required {:?} companion {:?} declared by {:?}: {source}",
                request.kind, request.key, request.owner,
            ),
        }
    }
}
impl<E: StdError + 'static> StdError for LoadError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match self {
            Self::Native(source) => source,
            Self::Companion { source, .. } => source,
        })
    }
}
