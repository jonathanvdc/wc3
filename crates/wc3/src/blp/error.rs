use std::{error::Error, fmt};

/// Reason a BLP container could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadErrorKind {
    /// The input does not begin with a BLP1 or BLP2 signature.
    InvalidMagic,
    /// The input ends before a required header or content field.
    UnexpectedEnd,
    /// A container field has an invalid or unsupported value.
    InvalidValue {
        /// Name of the invalid field.
        field: &'static str,
        /// Rejected value as stored in the container.
        value: u32,
    },
    /// A mipmap byte range is invalid or extends outside the input.
    InvalidRange {
        /// Zero-based mipmap level, with level zero at full resolution.
        level: usize,
    },
    /// A required or requested mipmap level is absent.
    MissingMipmap {
        /// Zero-based mipmap level, with level zero at full resolution.
        level: usize,
    },
}

/// A read failure at an absolute byte offset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadError {
    /// Absolute byte offset in the input where the failure was detected.
    pub offset: usize,
    /// Reason the container could not be read.
    pub kind: ReadErrorKind,
}

impl ReadError {
    pub(crate) const fn new(offset: usize, kind: ReadErrorKind) -> Self {
        Self { offset, kind }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BLP read error at byte {}: ", self.offset)?;
        match &self.kind {
            ReadErrorKind::InvalidMagic => f.write_str("expected BLP1 or BLP2 magic"),
            ReadErrorKind::UnexpectedEnd => f.write_str("unexpected end of input"),
            ReadErrorKind::InvalidValue { field, value } => {
                write!(f, "invalid {field} value {value}")
            }
            ReadErrorKind::InvalidRange { level } => {
                write!(f, "invalid mipmap {level} range")
            }
            ReadErrorKind::MissingMipmap { level } => {
                write!(f, "missing mipmap {level}")
            }
        }
    }
}

impl Error for ReadError {}

/// A container that cannot be represented as a BLP file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteError {
    /// A container field has an invalid or unsupported value.
    InvalidValue {
        /// Name of the invalid field.
        field: &'static str,
    },
    /// A required or requested mipmap level is absent.
    MissingMipmap {
        /// Zero-based mipmap level, with level zero at full resolution.
        level: usize,
    },
    /// An encoded size or offset cannot be represented by the BLP format.
    SizeOverflow,
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidValue { field } => write!(f, "invalid BLP {field}"),
            Self::MissingMipmap { level } => write!(f, "missing BLP mipmap {level}"),
            Self::SizeOverflow => f.write_str("BLP file exceeds u32 offset or size limit"),
        }
    }
}

impl Error for WriteError {}
