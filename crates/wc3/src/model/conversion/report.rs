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
