//! Structured M3 codec failures.
use std::{error::Error, fmt};

/// A malformed M3 document or record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadError {
    /// Byte offset within the addressed source.
    ///
    /// Cross-record validation uses zero when no encoded location is available.
    pub offset: usize,
    /// Description of the failed format requirement.
    pub field: &'static str,
}
impl ReadError {
    /// Reports an invalid value or bounds violation.
    pub fn invalid(offset: usize, field: &'static str) -> Self {
        Self { offset, field }
    }
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid M3 {} at byte {}", self.field, self.offset)
    }
}
impl Error for ReadError {}

/// A value that cannot be represented by an M3 document.
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
        write!(f, "cannot encode M3 {}", self.field)
    }
}
impl Error for WriteError {}
