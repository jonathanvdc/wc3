//! Errors that can occur when reading or writing MDX files.

use std::fmt;

/// Errors caused by malformed input or a payload too large for the MDX format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// The input does not begin with `MDLX`.
    InvalidMagic,
    /// The chunk header is incomplete.
    TruncatedHeader { offset: usize },
    /// The declared chunk payload exceeds the input.
    TruncatedChunk {
        tag: [u8; 4],
        offset: usize,
        size: u32,
    },
    /// `VERS` has no four-byte version number.
    InvalidVersionChunk,
    /// A record belongs to a different MDX version than its destination.
    VersionMismatch { expected: u32, actual: u32 },
    /// The payload cannot be represented by a 32-bit MDX chunk size.
    ChunkTooLarge { tag: [u8; 4], size: usize },
    /// A known chunk is too short for its fixed layout.
    MalformedChunk {
        tag: [u8; 4],
        size: usize,
        expected: usize,
    },
    /// A fixed-width string is too long or contains a NUL.
    InvalidString { max_bytes: usize },
    /// A size-bounded record or section is malformed at the given offset.
    MalformedRecord { tag: [u8; 4], offset: usize },
    /// A decoder stopped before the end of an exact record input.
    TrailingRecordBytes { consumed: usize, total: usize },
    /// A bounded cursor could not read the requested number of bytes.
    UnexpectedEnd { offset: usize, needed: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic => write!(f, "expected MDLX magic"),
            Self::TruncatedHeader { offset } => {
                write!(f, "truncated chunk header at byte {offset}")
            }
            Self::TruncatedChunk { tag, offset, size } => write!(
                f,
                "truncated {:?} chunk at byte {offset} (declared size {size})",
                String::from_utf8_lossy(tag)
            ),
            Self::InvalidVersionChunk => write!(f, "VERS chunk has fewer than four bytes"),
            Self::VersionMismatch { expected, actual } => write!(
                f,
                "record version {actual} does not match model version {expected}"
            ),
            Self::ChunkTooLarge { tag, size } => write!(
                f,
                "{:?} chunk size {size} exceeds u32",
                String::from_utf8_lossy(tag)
            ),
            Self::MalformedChunk {
                tag,
                size,
                expected,
            } => write!(
                f,
                "{:?} chunk has {size} bytes; expected at least {expected}",
                String::from_utf8_lossy(tag)
            ),
            Self::InvalidString { max_bytes } => {
                write!(f, "string must fit in {max_bytes} bytes and contain no NUL")
            }
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
        }
    }
}

impl std::error::Error for Error {}
