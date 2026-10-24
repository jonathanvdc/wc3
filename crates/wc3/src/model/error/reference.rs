//! Typed references and relationship constraints in checked semantic models.
use super::Owner;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A reference or relationship that cannot be satisfied by its owning graph.
#[derive(Clone, Debug)]
pub enum ReferenceError {
    /// The exact referenced identity has no definition in this domain.
    Unknown {
        /// Identity that could not be resolved.
        target: Owner,
    },
    /// The referenced identity belongs to another owning graph.
    Foreign {
        /// Foreign identity that cannot be selected here.
        target: Owner,
    },
    /// A relationship uses an identity more than once where uniqueness is required.
    Duplicate {
        /// Repeated identity.
        target: Owner,
    },
    /// Following parent references returns to this identity.
    Cycle {
        /// Identity repeated in the ancestor chain.
        target: Owner,
    },
    /// Traversal reported an invalid authored value without more specific evidence.
    /// The `ValidateLinks` contract currently carries no constraint or value payload.
    InvalidValue,
}
impl Display for ReferenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown { target } => write!(f, "identity {target:?} has no definition"),
            Self::Foreign { target } => write!(f, "identity {target:?} belongs to another graph"),
            Self::Duplicate { target } => {
                write!(f, "identity {target:?} is selected more than once")
            }
            Self::Cycle { target } => {
                write!(f, "parent relationships contain a cycle through {target:?}")
            }
            Self::InvalidValue => {
                f.write_str("relationship traversal reported an invalid authored value")
            }
        }
    }
}
impl StdError for ReferenceError {}
