//! Native decoding and checked-import failures for independent MDLX models.
use crate::model::conversion;
use crate::model::formats::mdlx::{mdl, mdx};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A native decoding or checked semantic import failure.
///
/// Codec variants retain the original input location and structured reason.
/// A successfully decoded document can still fail checked import, whose error
/// retains its semantic field path. Each variant exposes its original cause
/// through [`StdError::source`].
#[derive(Clone, Debug)]
pub enum ReadError {
    /// Binary MDX decoding failed before semantic import.
    Mdx(mdx::ReadError),
    /// Text MDL decoding failed before semantic import.
    Mdl(mdl::ReadError),
    /// Native decoding succeeded, but checked semantic import failed.
    Import(conversion::Error),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mdx(source) => write!(f, "cannot decode MDX model: {source}"),
            Self::Mdl(source) => write!(f, "cannot decode MDL model: {source}"),
            Self::Import(source) => write!(f, "cannot import MDLX model: {source}"),
        }
    }
}

impl StdError for ReadError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match self {
            Self::Mdx(source) => source,
            Self::Mdl(source) => source,
            Self::Import(source) => source,
        })
    }
}
