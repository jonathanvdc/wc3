use super::{Conversion, Error, Report};

/// An inspected, validated draft whose execution does no external I/O.
#[derive(Clone, Debug)]
pub struct Plan<T> {
    conversion: Conversion<T>,
}
impl<T> Plan<T> {
    /// Creates a plan after adapter validation and native graph construction.
    pub fn new(value: T, report: Report) -> Self {
        Self {
            conversion: Conversion { value, report },
        }
    }
    /// Inspects the decisions before accepting the draft.
    pub fn report(&self) -> &Report {
        &self.conversion.report
    }
    /// Inspects the proposed native graph.
    pub fn draft(&self) -> &T {
        &self.conversion.value
    }
    /// Accepts the inspected draft.
    pub fn execute(self) -> Result<Conversion<T>, Error> {
        Ok(self.conversion)
    }
}
