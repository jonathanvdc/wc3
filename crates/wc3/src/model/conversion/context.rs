use super::{
    ApproximationPolicy, Error, ExportOptions, ImportOptions, Issue, IssueKind, LossPolicy, Report,
    UnknownPolicy,
};
use crate::model::error::Location;
use std::error::Error as StdError;

/// Shared policy enforcement used by native adapters.
pub struct Context {
    /// Collected conversion decisions.
    pub report: Report,
    /// Semantic loss policy.
    pub loss_policy: LossPolicy,
    /// Unknown data policy.
    pub unknown: UnknownPolicy,
    /// Numeric approximation policy.
    pub approximation: ApproximationPolicy,
}
impl Context {
    /// Applies import policies.
    pub fn import(options: &ImportOptions) -> Self {
        Self {
            report: Report::default(),
            loss_policy: options.loss_policy,
            unknown: options.unknown,
            approximation: options.approximation,
        }
    }
    /// Applies export policies.
    pub fn export(options: &ExportOptions) -> Self {
        Self {
            report: Report::default(),
            loss_policy: options.loss_policy,
            unknown: UnknownPolicy::Reject,
            approximation: options.approximation,
        }
    }
    /// Records a representation decision.
    pub fn issue(
        &mut self,
        path: impl Into<String>,
        kind: IssueKind,
        description: impl Into<String>,
    ) {
        self.report.issues.push(Issue {
            path: path.into(),
            kind,
            description: description.into(),
        });
    }
    /// Rejects or reports unsupported nondefault semantics.
    pub fn unsupported(
        &mut self,
        path: Location,
        description: impl Into<String>,
    ) -> Result<(), Error> {
        let description = description.into();
        if self.loss_policy == LossPolicy::Reject {
            return Err(Error::requirement(path, description));
        }
        self.issue(path.to_string(), IssueKind::Dropped, description);
        Ok(())
    }
    /// Retains a rejected operation's cause, or reports its explicitly accepted loss.
    pub fn unsupported_source(
        &mut self,
        location: Location,
        source: impl StdError + Send + Sync + 'static,
    ) -> Result<(), Error> {
        if self.loss_policy == LossPolicy::Reject {
            return Err(Error::from_source(location, source));
        }
        self.issue(location.to_string(), IssueKind::Dropped, source.to_string());
        Ok(())
    }
    /// Rejects or reports discarded source data without a semantic representation.
    pub fn unknown_data(
        &mut self,
        path: Location,
        description: impl Into<String>,
    ) -> Result<(), Error> {
        let description = description.into();
        match self.unknown {
            UnknownPolicy::Reject => Err(Error::requirement(path, description)),
            UnknownPolicy::Drop => {
                self.issue(path.to_string(), IssueKind::Dropped, description);
                Ok(())
            }
        }
    }
    /// Checks one explicit numeric approximation against the configured tolerance.
    /// Builds diagnostic context only when the representation changes.
    pub fn approximate(
        &mut self,
        path: impl FnOnce() -> Location,
        error: f32,
        description: impl Into<String>,
    ) -> Result<(), Error> {
        if error == 0.0 {
            return Ok(());
        }
        let path = path();
        let description = description.into();
        if let ApproximationPolicy::Allow { tolerance } = self.approximation {
            if tolerance.is_finite() && tolerance >= 0.0 && error.is_finite() && error <= tolerance
            {
                self.issue(path.to_string(), IssueKind::Approximated, description);
                return Ok(());
            }
        }
        Err(Error::requirement(path, description))
    }
}
