//! Physics failures in checked semantic models.
use crate::model::{Mat4, Vec3};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A physics constraint that prevents checked model use.
#[derive(Clone, Debug)]
pub enum PhysicsError {
    /// A physical coefficient is not finite.
    NonFiniteValue {
        #[doc = "Rejected numeric input."]
        value: f64,
    },
    /// A nonnegative physical coefficient is negative.
    NegativeValue {
        #[doc = "Rejected numeric input."]
        value: f64,
    },
    /// A collision transform contains a nonfinite component.
    NonFiniteMatrix {
        #[doc = "Rejected authored transform."]
        matrix: Mat4,
    },
    /// A collision position contains a nonfinite component.
    NonFinitePosition {
        #[doc = "Rejected collision position."]
        position: Vec3,
    },
    /// A collision triangle references a position outside its stream.
    TriangleVertexOutOfBounds {
        #[doc = "Out-of-bounds vertex index."]
        vertex: u32,
        #[doc = "Number of authored collision positions."]
        positions: usize,
    },
}
impl Display for PhysicsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteValue { value } => {
                write!(f, "physical coefficient must be finite, found {value}")
            }
            Self::NegativeValue { value } => {
                write!(f, "physical coefficient must be nonnegative, found {value}")
            }
            Self::NonFiniteMatrix { .. } => f.write_str("collision transform must be finite"),
            Self::NonFinitePosition { position } => {
                write!(f, "collision position must be finite, found {position:?}")
            }
            Self::TriangleVertexOutOfBounds { vertex, positions } => write!(
                f,
                "collision vertex index {vertex} exceeds the {positions} positions"
            ),
        }
    }
}
impl StdError for PhysicsError {}
