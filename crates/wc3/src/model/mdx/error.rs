//! Structured errors for the binary MDX codec.
use crate::model::{Tag, Version};
use std::{error::Error as StdError, fmt};

/// Why an MDX value could not be decoded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadErrorKind {
    /// The file does not begin with `MDLX`.
    InvalidMagic,
    /// An enum discriminator has no recognized variant.
    UnknownEnumValue {
        /// Name of the enum being decoded.
        enum_name: &'static str,
        /// Unrecognized numeric discriminator.
        value: u32,
    },
    /// A required section tag differs from the input.
    UnexpectedTag {
        /// Required tag.
        expected: Tag,
        /// Tag encountered in the input.
        actual: Tag,
    },
    /// A record contains an unrecognized section or animation tag.
    UnknownTag {
        /// Unrecognized tag.
        actual: Tag,
    },
    /// The file version differs from the requested model version.
    VersionMismatch {
        /// Version requested by the caller.
        expected: Version,
        /// Version stored in the input.
        actual: Version,
    },
    /// No codec is available for the file version.
    UnsupportedVersion {
        /// Version stored in the input.
        version: Version,
    },
    /// A fixed-size payload has an incorrect length.
    SizeMismatch {
        /// Number of bytes supplied.
        actual: usize,
        /// Number of bytes required.
        expected: usize,
    },
    /// A decoded field violates its format constraints.
    InvalidValue {
        /// Field or constraint that failed validation.
        field: &'static str,
    },
    /// An encoded size exceeds the addressable input range.
    SizeOverflow,
    /// A record reader succeeded without consuming bytes.
    NoProgress,
    /// A complete-value read left bytes unconsumed.
    TrailingBytes {
        /// Number of unconsumed bytes.
        remaining: usize,
    },
    /// The input ends before the requested read can be satisfied.
    UnexpectedEnd {
        /// Number of bytes requested by the read.
        needed: usize,
        /// Number of bytes available at that position.
        remaining: usize,
    },
    /// An inclusive record length is smaller than its own size prefix.
    InvalidRecordLength {
        /// Inclusive length read from the input.
        length: usize,
    },
}
/// A failure located in the original input, with optional chunk/section context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadError {
    /// Absolute byte offset in the original input.
    pub offset: usize,
    /// Optional enclosing chunk or more specific section tag.
    pub tag: Option<Tag>,
    /// Structured reason decoding failed.
    pub kind: ReadErrorKind,
}
impl ReadError {
    /// Creates an error at an absolute offset without tag context.
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
    /// Sets the tag context, replacing any existing tag.
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
    /// An encoded size or count exceeds its 32-bit MDX representation.
    SizeOverflow {
        /// Chunk or section containing the field.
        tag: Tag,
        /// Field or constraint that could not be encoded.
        field: &'static str,
        /// Size or count that exceeded the format limit.
        size: usize,
    },
    /// A value violates the constraints of its MDX field.
    InvalidValue {
        /// Chunk or section containing the field.
        tag: Tag,
        /// Field or constraint that could not be encoded.
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
