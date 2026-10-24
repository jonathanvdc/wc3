//! Distinct operation and publication failures for transactional model editing.
use super::Error;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A transaction rejected by its operation or by final semantic validation.
#[derive(Clone, Debug)]
pub enum EditError<E> {
    /// The application operation failed before publication was attempted.
    Operation(E),
    /// The operation completed, but the resulting graph could not be published.
    Validation(Error),
}
impl<E: Display> Display for EditError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Operation(cause) => write!(f, "model edit operation failed: {cause}"),
            Self::Validation(cause) => write!(f, "model edit validation failed: {cause}"),
        }
    }
}
impl<E: StdError + 'static> StdError for EditError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match self {
            Self::Operation(cause) => cause,
            Self::Validation(cause) => cause,
        })
    }
}
