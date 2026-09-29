use std::{error::Error, fmt};

/// Reason a BLP container could not be read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadErrorKind {
    InvalidMagic,
    UnexpectedEnd,
    InvalidValue { field: &'static str, value: u32 },
    InvalidRange { level: usize },
    MissingMipmap { level: usize },
}

/// A read failure at an absolute byte offset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadError {
    pub offset: usize,
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
    InvalidValue { field: &'static str },
    MissingMipmap { level: usize },
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
