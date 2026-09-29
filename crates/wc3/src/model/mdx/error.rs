//! Structured errors for the binary MDX codec.
use crate::model::{Tag, Version};
use std::{error::Error as StdError, fmt};

/// Why an MDX value could not be decoded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadErrorKind {
    InvalidMagic,
    UnknownEnumValue { enum_name: &'static str, value: u32 },
    UnexpectedTag { expected: Tag, actual: Tag },
    UnknownTag { actual: Tag },
    VersionMismatch { expected: Version, actual: Version },
    UnsupportedVersion { version: Version },
    SizeMismatch { actual: usize, expected: usize },
    InvalidValue { field: &'static str },
    SizeOverflow,
    NoProgress,
    TrailingBytes { remaining: usize },
    UnexpectedEnd { needed: usize, remaining: usize },
    InvalidRecordLength { length: usize },
}
/// A failure located in the original input, with optional chunk/section context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadError {
    pub offset: usize,
    pub tag: Option<Tag>,
    pub kind: ReadErrorKind,
}
impl ReadError {
    pub const fn new(offset: usize, kind: ReadErrorKind) -> Self {
        Self {
            offset,
            tag: None,
            kind,
        }
    }
    /// Adds enclosing chunk context without replacing a more specific section tag.
    pub fn in_chunk(mut self, tag: Tag) -> Self {
        if self.tag.is_none() {
            self.tag = Some(tag);
        }
        self
    }
    pub fn with_tag(mut self, tag: Tag) -> Self {
        self.tag = Some(tag);
        self
    }
}
impl fmt::Display for ReadErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => f.write_str("expected MDLX magic"),
            Self::UnknownEnumValue { enum_name, value } => {
                write!(f, "unknown {enum_name} value {value}")
            }
            Self::UnexpectedTag { expected, actual } => write!(
                f,
                "expected {:?} tag, found {:?}",
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(actual)
            ),
            Self::UnknownTag { actual } => {
                write!(f, "unknown tag {:?}", String::from_utf8_lossy(actual))
            }
            Self::VersionMismatch { expected, actual } => {
                write!(f, "expected model version {expected}, found {actual}")
            }
            Self::UnsupportedVersion { version } => {
                write!(f, "unsupported model version {version}")
            }
            Self::SizeMismatch { actual, expected } => {
                write!(f, "expected {expected} bytes, found {actual}")
            }
            Self::InvalidValue { field } => write!(f, "invalid {field}"),
            Self::SizeOverflow => f.write_str("encoded size exceeds addressable input"),
            Self::NoProgress => f.write_str("record reader consumed no input"),
            Self::TrailingBytes { remaining } => write!(f, "{remaining} unexpected trailing bytes"),
            Self::UnexpectedEnd { needed, remaining } => {
                write!(f, "need {needed} bytes, only {remaining} remain")
            }
            Self::InvalidRecordLength { length } => {
                write!(f, "record length {length} is smaller than its prefix")
            }
        }
    }
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.kind, self.offset)?;
        if let Some(tag) = self.tag {
            write!(f, " in {:?}", String::from_utf8_lossy(&tag))?;
        }
        Ok(())
    }
}
impl StdError for ReadError {}

/// Values that cannot be encoded as MDX.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteError {
    SizeOverflow {
        tag: Tag,
        field: &'static str,
        size: usize,
    },
    InvalidValue {
        tag: Tag,
        field: &'static str,
    },
}
impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SizeOverflow { tag, field, size } => write!(
                f,
                "{field} {size} exceeds u32 in {:?}",
                String::from_utf8_lossy(tag)
            ),
            Self::InvalidValue { tag, field } => {
                write!(f, "invalid {field} in {:?}", String::from_utf8_lossy(tag))
            }
        }
    }
}
impl StdError for WriteError {}
