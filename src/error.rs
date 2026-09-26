//! Errors that can occur when reading or writing MDX files.
use crate::{Tag, Version};

use std::{error::Error as StdError, fmt};

/// Errors caused by malformed MDX input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// The input does not begin with `MDLX`.
    InvalidMagic,
    /// A typed chunk decoder encountered another chunk tag.
    UnexpectedChunkTag { expected: Tag, actual: Tag },
    /// The chunk header is incomplete.
    TruncatedHeader { offset: usize },
    /// The declared chunk payload exceeds the input.
    TruncatedChunk { tag: Tag, offset: usize, size: u32 },
    /// `VERS` has no four-byte version number.
    InvalidVersionChunk,
    /// A known chunk is too short for its fixed layout.
    MalformedChunk {
        tag: Tag,
        size: usize,
        expected: usize,
    },
    /// A size-bounded record or section is malformed at the given offset.
    MalformedRecord { tag: Tag, offset: usize },
    /// A decoder stopped before the end of an exact record input.
    TrailingRecordBytes { consumed: usize, total: usize },
    /// A bounded cursor could not read the requested number of bytes.
    UnexpectedEnd { offset: usize, needed: usize },
    /// A size prefix is smaller than the size of the prefix itself.
    InvalidRecordLength { offset: usize, length: usize },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "expected MDLX magic"),
            Self::UnexpectedChunkTag { expected, actual } => write!(
                f,
                "expected {:?} chunk, found {:?}",
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(actual)
            ),
            Self::TruncatedHeader { offset } => {
                write!(f, "truncated chunk header at byte {offset}")
            }
            Self::TruncatedChunk { tag, offset, size } => write!(
                f,
                "truncated {:?} chunk at byte {offset} (declared size {size})",
                String::from_utf8_lossy(tag)
            ),
            Self::InvalidVersionChunk => write!(f, "VERS chunk has fewer than four bytes"),
            Self::MalformedChunk {
                tag,
                size,
                expected,
            } => write!(
                f,
                "{:?} chunk has {size} bytes; expected at least {expected}",
                String::from_utf8_lossy(tag)
            ),
            Self::MalformedRecord { tag, offset } => write!(
                f,
                "malformed {:?} record at byte {offset}",
                String::from_utf8_lossy(tag)
            ),
            Self::TrailingRecordBytes { consumed, total } => {
                write!(f, "record consumed {consumed} of {total} bytes")
            }
            Self::UnexpectedEnd { offset, needed } => {
                write!(f, "cannot read {needed} bytes at offset {offset}")
            }
            Self::InvalidRecordLength { offset, length } => {
                write!(f, "invalid record length {length} at offset {offset}")
            }
        }
    }
}

impl StdError for DecodeError {}

/// Errors caused by values that cannot be written as MDX.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncodeError {
    VersionMismatch { expected: Version, actual: Version },
    ChunkTooLarge { tag: Tag, size: usize },
    MalformedRecord { tag: Tag, offset: usize },
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VersionMismatch { expected, actual } => write!(
                f,
                "record version {actual} does not match model version {expected}"
            ),
            Self::ChunkTooLarge { tag, size } => write!(
                f,
                "{:?} chunk size {size} exceeds u32",
                String::from_utf8_lossy(tag)
            ),
            Self::MalformedRecord { tag, offset } => write!(
                f,
                "cannot encode {:?} record at byte {offset}",
                String::from_utf8_lossy(tag)
            ),
        }
    }
}

impl StdError for EncodeError {}

/// Errors caused by a value supplied to a constructor or setter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueError {
    InvalidString {
        max_bytes: usize,
    },
    VersionMismatch {
        expected: Version,
        actual: Version,
    },
    UnsupportedVersion {
        tag: Tag,
        minimum: Version,
        actual: Version,
    },
    UnavailableField {
        tag: Tag,
        field: &'static str,
    },
    InvalidTrackTag {
        record: Tag,
        track: Tag,
    },
    IndexOutOfBounds {
        tag: Tag,
        index: usize,
        len: usize,
    },
    LengthMismatch {
        tag: Tag,
        expected: usize,
        actual: usize,
    },
    MissingField {
        tag: Tag,
        field: &'static str,
    },
    CountTooLarge {
        tag: Tag,
        count: usize,
    },
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidString { max_bytes } => {
                write!(f, "string must fit in {max_bytes} bytes and contain no NUL")
            }
            Self::VersionMismatch { expected, actual } => write!(
                f,
                "value version {actual} does not match destination version {expected}"
            ),
            Self::UnsupportedVersion {
                tag,
                minimum,
                actual,
            } => write!(
                f,
                "{:?} requires version {minimum}, got {actual}",
                String::from_utf8_lossy(tag)
            ),
            Self::UnavailableField { tag, field } => write!(
                f,
                "{field} is unavailable in {:?}",
                String::from_utf8_lossy(tag)
            ),
            Self::InvalidTrackTag { record, track } => write!(
                f,
                "track {:?} is not valid in {:?}",
                String::from_utf8_lossy(track),
                String::from_utf8_lossy(record)
            ),
            Self::IndexOutOfBounds { tag, index, len } => write!(
                f,
                "index {index} is outside {:?} length {len}",
                String::from_utf8_lossy(tag)
            ),
            Self::LengthMismatch {
                tag,
                expected,
                actual,
            } => write!(
                f,
                "{:?} has {actual} values; expected {expected}",
                String::from_utf8_lossy(tag)
            ),
            Self::MissingField { tag, field } => {
                write!(f, "{:?} requires {field}", String::from_utf8_lossy(tag))
            }
            Self::CountTooLarge { tag, count } => write!(
                f,
                "{:?} count {count} exceeds u32",
                String::from_utf8_lossy(tag)
            ),
        }
    }
}

impl StdError for ValueError {}
