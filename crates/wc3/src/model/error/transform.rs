//! Transform, coordinate and prepared-pose failures in checked models.
use crate::model::{Axis, LengthUnit, Mat4, NodeId, PoseBehavior, Quat, Vec3, Vec4};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// A spatial constraint with the rejected value or relationship preserved.
#[derive(Clone, Debug)]
pub enum TransformError {
    /// A transform matrix contains a nonfinite component.
    NonFiniteMatrix {
        #[doc = "Rejected authored matrix."]
        matrix: Mat4,
    },
    /// An explicit inverse-bind stream has the wrong cardinality.
    BindMatrixCount {
        #[doc = "Required stream length."]
        expected: usize,
        #[doc = "Authored stream length."]
        actual: usize,
    },
    /// A transform vector contains a nonfinite component.
    NonFiniteVector {
        #[doc = "Rejected authored vector."]
        vector: Vec3,
    },
    /// A rotation is nonfinite or has zero length.
    InvalidRotation {
        #[doc = "Rejected authored rotation."]
        rotation: Quat,
    },
    /// Up and forward lie on the same coordinate dimension.
    DependentBasisAxes {
        #[doc = "Requested upward axis."]
        up: Axis,
        #[doc = "Requested forward axis."]
        forward: Axis,
    },
    /// A physical unit is not finite and positive.
    InvalidMetersPerUnit {
        #[doc = "Rejected physical unit scale."]
        value: f64,
    },
    /// A unit conversion cannot be represented by a positive finite f32 scale.
    UnrepresentableScale {
        #[doc = "Authored distance unit."]
        source: LengthUnit,
        #[doc = "Requested output distance unit."]
        target: LengthUnit,
        #[doc = "Computed ratio before f32 representation."]
        scale: f64,
    },
    /// A prepared hierarchy contains the same source identity more than once.
    DuplicatePoseNode {
        #[doc = "Duplicate prepared source identity."]
        node: NodeId,
    },
    /// A prepared parent index is outside the node array.
    ParentIndexOutOfBounds {
        #[doc = "Requested parent index."]
        parent: usize,
        #[doc = "Number of prepared nodes."]
        nodes: usize,
    },
    /// Cyclic ancestry prevents this node from receiving a traversal position.
    PoseHierarchyCycle {
        #[doc = "Identity whose ancestry cannot be ordered."]
        blocked: NodeId,
    },
    /// A corrected hierarchy uses behavior incompatible with affine correction.
    UnsupportedCorrectionBehavior {
        #[doc = "Behavior incompatible with correction."]
        behavior: PoseBehavior,
    },
    /// A transform has a projective final row instead of an affine one.
    NonAffineMatrix {
        #[doc = "Projective row that violates affine interpretation."]
        last_row: Vec4,
    },
    /// A driving camera has a singular linear component.
    SingularCamera {
        #[doc = "Computed camera linear determinant."]
        determinant: f32,
    },
    /// Evaluation output and its prepared hierarchy contain different node counts.
    PoseNodeCount {
        #[doc = "Required stream length."]
        expected: usize,
        #[doc = "Authored stream length."]
        actual: usize,
    },
    /// A skin palette selects an element outside the authored joint list.
    JointPaletteOutOfBounds {
        #[doc = "Requested joint or matrix index."]
        index: usize,
        #[doc = "Number of authored joints."]
        joints: usize,
    },
    /// An authored skin joint does not occur in this prepared hierarchy.
    JointOutsideHierarchy {
        #[doc = "Requested joint identity."]
        joint: NodeId,
    },
    /// A selected skin joint has no corresponding inverse-bind transform.
    MissingInverseBind {
        #[doc = "Requested joint or matrix index."]
        index: usize,
        #[doc = "Number of available inverse-bind matrices."]
        matrices: usize,
    },
}
impl Display for TransformError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteMatrix { .. } => f.write_str("transform matrix must be finite"),
            Self::BindMatrixCount { expected, actual } => write!(f, "expected {expected} inverse bind matrices, found {actual}"),
            Self::NonFiniteVector { vector } => write!(f, "transform vector must be finite, found {vector:?}"),
            Self::InvalidRotation { rotation } => write!(f, "rotation must be finite and nonzero, found {rotation:?}"),
            Self::DependentBasisAxes { up, forward } => write!(f, "up {up:?} and forward {forward:?} must be independent axes"),
            Self::InvalidMetersPerUnit { value } => write!(f, "meters per unit must be finite and positive, found {value}"),
            Self::UnrepresentableScale { source, target, scale } => write!(f, "scale {scale} from {source:?} to {target:?} cannot be represented by f32 transforms"),
            Self::DuplicatePoseNode { node } => write!(f, "prepared node {node:?} occurs more than once"),
            Self::ParentIndexOutOfBounds { parent, nodes } => write!(f, "parent index {parent} exceeds the {nodes} prepared nodes"),
            Self::PoseHierarchyCycle { blocked } => write!(f, "cyclic ancestry prevents traversal of node {blocked:?}"),
            Self::UnsupportedCorrectionBehavior { .. } => f.write_str("corrected hierarchies require ordinary affine inheritance without billboards or camera anchoring"),
            Self::NonAffineMatrix { last_row } => write!(f, "transform must be affine, found final row {last_row:?}"),
            Self::SingularCamera { determinant } => write!(f, "camera must be nonsingular, found determinant {determinant}"),
            Self::PoseNodeCount { expected, actual } => write!(f, "expected {expected} evaluated nodes, found {actual}"),
            Self::JointPaletteOutOfBounds { index, joints } => write!(f, "joint palette index {index} exceeds the {joints} authored joints"),
            Self::JointOutsideHierarchy { joint } => write!(f, "joint {joint:?} belongs to another hierarchy"),
            Self::MissingInverseBind { index, matrices } => write!(f, "joint {index} has no inverse bind in the {matrices} matrices"),
        }
    }
}
impl StdError for TransformError {}
