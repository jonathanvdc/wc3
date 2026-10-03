//! Model validation and transport errors.
use super::{Tag, Version};
use std::{error::Error as StdError, fmt, io};

/// Errors caused by a value supplied to a constructor or setter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueError {
    /// The text contains NUL or exceeds its fixed-width byte capacity.
    InvalidString {
        /// Maximum text length in bytes.
        max_bytes: usize,
    },
    /// The chunk is unavailable in this model version.
    UnsupportedVersion {
        /// Tag of the affected chunk.
        tag: Tag,
        /// Earliest supported model version.
        minimum: Version,
        /// Model version selected by the caller.
        actual: Version,
    },
    /// The field is unavailable in this model version.
    UnsupportedField {
        /// Tag of the affected chunk.
        tag: Tag,
        /// Name of the affected field.
        field: &'static str,
        /// Model version selected by the caller.
        actual: Version,
    },
    /// The optional field is absent from this record.
    UnavailableField {
        /// Tag of the affected chunk.
        tag: Tag,
        /// Name of the affected field.
        field: &'static str,
    },
    /// An index is outside the referenced collection.
    IndexOutOfBounds {
        /// Tag of the affected chunk.
        tag: Tag,
        /// Index supplied by the caller.
        index: usize,
        /// Number of available entries.
        len: usize,
    },
    /// A supplied array has an unexpected length.
    LengthMismatch {
        /// Tag of the affected chunk.
        tag: Tag,
        /// Required number of entries.
        expected: usize,
        /// Number of entries supplied by the caller.
        actual: usize,
    },
    /// The record count exceeds the wire-format capacity.
    CountTooLarge {
        /// Tag of the affected chunk.
        tag: Tag,
        /// Number of entries supplied by the caller.
        count: usize,
    },
    /// The global-sequence ID cannot be represented as a signed wire value.
    InvalidGlobalSequenceId {
        /// Global-sequence ID supplied by the caller.
        id: u32,
    },
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidString { max_bytes } => {
                write!(f, "string must fit in {max_bytes} bytes and contain no NUL")
            }
            Self::UnsupportedVersion {
                tag,
                minimum,
                actual,
            } => write!(
                f,
                "{:?} requires version {minimum}, got {actual}",
                String::from_utf8_lossy(tag)
            ),
            Self::UnsupportedField { tag, field, actual } => write!(
                f,
                "{field} is unsupported in {:?} version {actual}",
                String::from_utf8_lossy(tag)
            ),
            Self::UnavailableField { tag, field } => write!(
                f,
                "{field} is unavailable in {:?}",
                String::from_utf8_lossy(tag)
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
            Self::CountTooLarge { tag, count } => write!(
                f,
                "{:?} count {count} exceeds u32",
                String::from_utf8_lossy(tag)
            ),
            Self::InvalidGlobalSequenceId { id } => {
                write!(f, "global sequence ID {id} is reserved for no sequence")
            }
        }
    }
}

impl StdError for ValueError {}

/// A transport failure or an error produced by a codec.
#[derive(Debug)]
pub enum IoError<E> {
    /// An underlying stream operation failed.
    Io(io::Error),
    /// The format codec rejected the data.
    Codec(E),
}
impl<E> From<io::Error> for IoError<E> {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl<E: fmt::Display> fmt::Display for IoError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Codec(error) => error.fmt(f),
        }
    }
}
impl<E: StdError + 'static> StdError for IoError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match self {
            Self::Io(error) => error,
            Self::Codec(error) => error,
        })
    }
}
