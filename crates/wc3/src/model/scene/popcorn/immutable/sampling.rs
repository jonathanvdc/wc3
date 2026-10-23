//! Validation errors for one-pass public particle field reads.
use crate::model::scene::popcorn::EffectFieldId;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A field request is validated completely before its output buffer changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldSamplingError {
    /// Exactly one result is required for each requested field.
    OutputLength {
        /// Number of requested fields.
        expected: usize,
        /// Supplied output length.
        actual: usize,
    },
    /// The field ID is outside the particle's owning layer.
    UnknownField(#[doc = "Requested field ID."] EffectFieldId),
    /// The native field has no checked public value layout.
    UntypedField(#[doc = "Requested field ID."] EffectFieldId),
}
impl Display for FieldSamplingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::OutputLength { expected, actual } => write!(
                f,
                "particle field sampling needs {expected} outputs, received {actual}"
            ),
            Self::UnknownField(field) => {
                write!(f, "field {} is outside the particle layer", field.0)
            }
            Self::UntypedField(field) => write!(f, "field {} has no checked value layout", field.0),
        }
    }
}
impl Error for FieldSamplingError {}
