//! Physics failures in checked semantic models.
use crate::model::{Mat4, NodeId, Vec3, Vec4};
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
    /// The same body placement identity occurs more than once.
    DuplicateBody {
        #[doc = "Referenced body placement identity."]
        body: NodeId,
    },
    /// Body following revisits this identity in its ancestry.
    BodyCycle {
        #[doc = "Referenced body placement identity."]
        body: NodeId,
    },
    /// A required body placement does not belong to the physics system.
    MissingBody {
        #[doc = "Referenced body placement identity."]
        body: NodeId,
    },
    /// A collision transform contains a projective final row.
    NonAffineMatrix {
        #[doc = "Rejected projective final row."]
        last_row: Vec4,
    },
    /// The convex boundary has different face and plane counts.
    HullPlaneCount {
        #[doc = "Number of authored convex faces."]
        faces: usize,
        #[doc = "Number of authored convex planes."]
        planes: usize,
    },
    /// A convex face has fewer than three vertices.
    HullFaceVertexCount {
        #[doc = "Number of vertices in the rejected face."]
        actual: usize,
    },
    /// A convex face references a vertex outside the hull stream.
    HullVertexOutOfBounds {
        #[doc = "Referenced hull vertex index."]
        vertex: u32,
        #[doc = "Number of authored hull vertices."]
        vertices: usize,
    },
    /// A directed boundary edge repeats the same vertex.
    HullDegenerateEdge {
        #[doc = "Referenced hull vertex index."]
        vertex: u32,
    },
    /// A directed boundary edge occurs more than once.
    HullDuplicateEdge {
        #[doc = "Directed boundary edge origin."]
        from: u32,
        #[doc = "Directed boundary edge destination."]
        to: u32,
    },
    /// A directed boundary edge has no reverse edge closing the boundary.
    HullOpenEdge {
        #[doc = "Directed boundary edge origin."]
        from: u32,
        #[doc = "Directed boundary edge destination."]
        to: u32,
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
            Self::DuplicateBody { body } => {
                write!(f, "body placement {body:?} occurs more than once")
            }
            Self::BodyCycle { body } => write!(f, "body following revisits {body:?}"),
            Self::MissingBody { body } => {
                write!(f, "body {body:?} does not belong to this physics system")
            }
            Self::NonAffineMatrix { last_row } => write!(
                f,
                "collision transform must be affine, found final row {last_row:?}"
            ),
            Self::HullPlaneCount { faces, planes } => {
                write!(f, "expected {faces} convex face planes, found {planes}")
            }
            Self::HullFaceVertexCount { actual } => write!(
                f,
                "convex face requires at least three vertices, found {actual}"
            ),
            Self::HullVertexOutOfBounds { vertex, vertices } => write!(
                f,
                "convex face vertex index {vertex} exceeds the {vertices} hull vertices"
            ),
            Self::HullDegenerateEdge { vertex } => {
                write!(f, "convex boundary edge repeats vertex {vertex}")
            }
            Self::HullDuplicateEdge { from, to } => {
                write!(f, "convex boundary repeats directed edge {from}->{to}")
            }
            Self::HullOpenEdge { from, to } => {
                write!(f, "convex boundary edge {from}->{to} has no reverse edge")
            }
            Self::TriangleVertexOutOfBounds { vertex, positions } => write!(
                f,
                "collision vertex index {vertex} exceeds the {positions} positions"
            ),
        }
    }
}
impl StdError for PhysicsError {}
