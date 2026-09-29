//! Model validation and transport errors.
use super::{Tag, Version};
use std::{error::Error as StdError, fmt, io};

/// Errors caused by a value supplied to a constructor or setter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValueError {
    InvalidString {
        max_bytes: usize,
    },
    UnsupportedVersion {
        tag: Tag,
        minimum: Version,
        actual: Version,
    },
    UnsupportedField {
        tag: Tag,
        field: &'static str,
        actual: Version,
    },
    UnavailableField {
        tag: Tag,
        field: &'static str,
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
    CountTooLarge {
        tag: Tag,
        count: usize,
    },
    InvalidGlobalSequenceId {
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
    Io(io::Error),
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
