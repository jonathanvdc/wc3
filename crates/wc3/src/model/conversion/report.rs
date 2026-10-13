use super::super::wem;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// Classification of a conversion decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssueKind {
    /// Equivalent representation change.
    Normalized,
    /// Destination-only default initialization.
    Initialized,
    /// Explicitly accepted approximation.
    Approximated,
    /// Explicitly accepted semantic loss.
    Dropped,
    /// Uninterpreted payload retained separately from active model data.
    Archived,
}
/// A diagnostic at a stable semantic location.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Issue {
    /// Model/mesh/node/channel or source field path.
    pub path: String,
    /// Decision classification.
    pub kind: IssueKind,
    /// Explanation and practical consequence.
    pub description: String,
}
/// Ordered conversion decisions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
    /// Issues in stable traversal order.
    pub issues: Vec<Issue>,
}
/// A completed conversion together with its decisions.
#[derive(Clone, Debug)]
pub struct Conversion<T> {
    /// Converted native or editable asset.
    pub value: T,
    /// Conversion diagnostics.
    pub report: Report,
}
/// An unrepresentable feature or invalid conversion input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    /// Concrete semantic location.
    pub path: String,
    /// Failed requirement.
    pub description: String,
}
impl Error {
    /// Creates a conversion failure.
    pub fn new(path: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            description: description.into(),
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "conversion failed at {}: {}",
            self.path, self.description
        )
    }
}
impl StdError for Error {}
impl From<wem::ReadError> for Error {
    fn from(error: wem::ReadError) -> Self {
        Self::new(format!("wem.bytes[{}]", error.offset), error.to_string())
    }
}
impl From<wem::WriteError> for Error {
    fn from(error: wem::WriteError) -> Self {
        Self::new("wem", error.to_string())
    }
}
