//! Structured WEM codec failures.
use std::{error::Error, fmt};

/// Classification used to distinguish malformed bytes from an unknown layout.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadErrorKind {
    /// A required span, value, or graph relationship is invalid.
    Malformed,
    /// A discriminator selects a payload whose shape is not implemented.
    UnsupportedLayout,
}

/// A malformed WEM document or an unsupported record layout.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadError {
    /// Byte offset within the addressed source.
    ///
    /// Cross-record validation uses zero when no encoded location is available.
    pub offset: usize,
    /// Description of the failed format requirement.
    pub field: &'static str,
    /// Whether raw storage may retain the entire section as opaque.
    pub kind: ReadErrorKind,
}
impl ReadError {
    /// Reports an invalid value or bounds violation.
    pub fn invalid(offset: usize, field: &'static str) -> Self {
        Self {
            offset,
            field,
            kind: ReadErrorKind::Malformed,
        }
    }
    /// Reports a future payload shape without treating bounded bytes as corrupt.
    pub fn unsupported(offset: usize, field: &'static str) -> Self {
        Self {
            offset,
            field,
            kind: ReadErrorKind::UnsupportedLayout,
        }
    }
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} WEM {} at byte {}",
            if self.kind == ReadErrorKind::Malformed {
                "invalid"
            } else {
                "unsupported"
            },
            self.field,
            self.offset
        )
    }
}
impl Error for ReadError {}

/// A value that cannot be represented by a WEM document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WriteError {
    /// Description of the failed format requirement.
    pub field: &'static str,
}
impl WriteError {
    /// Reports an unrepresentable value or layout.
    pub fn invalid(field: &'static str) -> Self {
        Self { field }
    }
}
impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cannot encode WEM {}", self.field)
    }
}
impl Error for WriteError {}
