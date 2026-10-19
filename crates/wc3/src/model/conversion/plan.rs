use super::{Conversion, Report};

/// An inspected, validated draft whose execution does no external I/O.
#[derive(Clone, Debug)]
pub struct Plan<T> {
    conversion: Conversion<T>,
}
impl<T> Plan<T> {
    /// Creates a plan after validating its proposed output.
    pub fn new(value: T, report: Report) -> Self {
        Self {
            conversion: Conversion { value, report },
        }
    }
    /// Inspects the decisions before accepting the draft.
    pub fn report(&self) -> &Report {
        &self.conversion.report
    }
    /// Inspects the proposed output.
    pub fn draft(&self) -> &T {
        &self.conversion.value
    }
    /// Accepts the inspected draft.
    pub fn execute(self) -> Conversion<T> {
        self.conversion
    }
}
