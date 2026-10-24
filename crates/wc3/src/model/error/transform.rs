//! Transform failures in checked semantic models.
use crate::model::Mat4;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A transform constraint that prevents checked model use.
#[derive(Clone, Debug)]
pub enum TransformError {
    /// A transform matrix contains a nonfinite component.
    NonFiniteMatrix {
        #[doc = "Rejected authored transform."]
        matrix: Mat4,
    },
    /// An explicit inverse-bind stream has the wrong cardinality.
    BindMatrixCount {
        #[doc = "Required inverse-bind matrix count."]
        expected: usize,
        #[doc = "Authored inverse-bind matrix count."]
        actual: usize,
    },
}
impl Display for TransformError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteMatrix { .. } => f.write_str("transform matrix must be finite"),
            Self::BindMatrixCount { expected, actual } => write!(
                f,
                "expected {expected} inverse bind matrices, found {actual}"
            ),
        }
    }
}
impl StdError for TransformError {}
