//! Checked construction and graph-edit errors.
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A semantic value or relationship that cannot form a valid model.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    path: String,
    message: String,
}
impl Error {
    /// Identifies the invalid field and explains its constraint.
    pub fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
    /// Field or object responsible for the failure.
    pub fn path(&self) -> &str {
        &self.path
    }
    /// Constraint that was violated.
    pub fn message(&self) -> &str {
        &self.message
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}: {}", self.path, self.message)
    }
}
impl StdError for Error {}
