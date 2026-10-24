//! Typed references and relationship constraints in checked semantic models.
use super::Owner;
use crate::model::{
    DrawId, FeatureId, FeatureKind, GeometryKind, InstanceId, LookId, MeshId, Quat, Texture,
    ValueSlot,
};
use glam::{Vec2, Vec3, Vec4};
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
    /// A declared forward identity still lacks its required definition.
    UnresolvedDefinition {
        /// Reserved identity.
        target: Owner,
    },
    /// A feature exists, but its payload does not satisfy the consuming relationship.
    FeatureKindMismatch {
        /// Selected feature.
        target: FeatureId,
        /// Accepted behavior categories.
        expected: Vec<FeatureKind>,
        /// Actual behavior category.
        actual: FeatureKind,
    },
    /// A mesh exists, but its geometry purpose does not satisfy the relationship.
    GeometryKindMismatch {
        /// Selected mesh.
        target: MeshId,
        /// Required geometry purpose.
        expected: GeometryKind,
        /// Actual geometry purpose.
        actual: GeometryKind,
    },
    /// An animated slot has no semantic definition that consumes its output.
    UnconsumedSlot {
        /// Unused animated slot.
        slot: ValueSlot,
    },
    /// An invalid rest constant retains its original typed value.
    InvalidConstant {
        /// Rejected authored constant.
        value: ConstantValue,
    },
    /// The same slot identity carries different constants in two inputs.
    ConflictingConstants {
        /// Previously registered value.
        existing: ConstantValue,
        /// Incoming value for the same identity.
        incoming: ConstantValue,
    },
    /// The same derived scalar slot refers to two different boolean source slots.
    ConflictingSources {
        /// Previously registered source.
        existing: ValueSlot,
        /// Incoming source.
        incoming: ValueSlot,
    },
    /// A single-look vocabulary cannot add a second look.
    SingleLookRequired,
    /// A single-look vocabulary cannot select another look or clear its default.
    SoleLookDefault {
        /// Required default.
        expected: LookId,
        /// Requested replacement.
        actual: Option<LookId>,
    },
    /// A model containing draws has no complete look to select.
    RequiredLook,
    /// A particular draw lacks a material assignment in the selected look.
    MissingDrawAssignment {
        /// Placement containing the draw.
        instance: InstanceId,
        /// Unassigned draw.
        draw: DrawId,
    },
    /// A complete look omits these placement/draw pairs.
    UnassignedDraws {
        /// Draws that require assignments.
        draws: Vec<(InstanceId, DrawId)>,
    },
    /// Placement construction received a different number of materials and draws.
    DrawMaterialCount {
        /// Placement draw count.
        draws: usize,
        /// Supplied material count.
        materials: usize,
    },
    /// Geometry with positive joint weights has no selected skin.
    RequiredSkin,
    /// An authored joint index exceeds the selected skin palette.
    JointOutOfBounds {
        /// Rejected palette index.
        joint: usize,
        /// Available joint count.
        joints: usize,
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
            Self::UnresolvedDefinition { target } => {
                write!(f, "reserved identity {target:?} has no definition")
            }
            Self::FeatureKindMismatch {
                target,
                expected,
                actual,
            } => write!(
                f,
                "feature {target:?} has kind {actual:?}; expected one of {expected:?}"
            ),
            Self::GeometryKindMismatch {
                target,
                expected,
                actual,
            } => write!(
                f,
                "mesh {target:?} has kind {actual:?}; expected {expected:?}"
            ),
            Self::UnconsumedSlot { slot } => {
                write!(f, "animated slot {slot:?} has no consuming semantic field")
            }
            Self::InvalidConstant { value } => write!(f, "invalid rest constant {value:?}"),
            Self::ConflictingConstants { existing, incoming } => {
                write!(f, "slot constants conflict: {existing:?} and {incoming:?}")
            }
            Self::ConflictingSources { existing, incoming } => write!(
                f,
                "converted sources conflict: {existing:?} and {incoming:?}"
            ),
            Self::SingleLookRequired => {
                f.write_str("this appearance vocabulary requires exactly one look")
            }
            Self::SoleLookDefault { expected, actual } => write!(
                f,
                "the sole look {expected:?} must remain the default; requested {actual:?}"
            ),
            Self::RequiredLook => f.write_str("draws require a complete material look"),
            Self::MissingDrawAssignment { instance, draw } => write!(
                f,
                "draw {draw:?} in placement {instance:?} has no material assignment"
            ),
            Self::UnassignedDraws { draws } => write!(f, "look omits draw assignments {draws:?}"),
            Self::DrawMaterialCount { draws, materials } => write!(
                f,
                "placement has {draws} draws but received {materials} materials"
            ),
            Self::RequiredSkin => f.write_str("positive joint influences require a selected skin"),
            Self::JointOutOfBounds { joint, joints } => {
                write!(f, "joint {joint} exceeds skin palette of {joints} joints")
            }
            Self::InvalidValue => {
                f.write_str("relationship traversal reported an invalid authored value")
            }
        }
    }
}
impl StdError for ReferenceError {}

/// An owned snapshot of a property constant rejected by semantic validation.
#[derive(Clone, Copy, Debug)]
pub enum ConstantValue {
    /// A value from the `Scalar` slot domain.
    Scalar(f32),
    /// A value from the `Vector2` slot domain.
    Vector2(Vec2),
    /// A value from the `Vector3` slot domain.
    Vector3(Vec3),
    /// A value from the `Vector4` slot domain.
    Vector4(Vec4),
    /// A value from the `Rotation` slot domain.
    Rotation(Quat),
    /// A value from the `Unsigned` slot domain.
    Unsigned(u32),
    /// A value from the `Signed` slot domain.
    Signed(i32),
    /// A value from the `Unsigned16` slot domain.
    Unsigned16(u16),
    /// A value from the `Byte` slot domain.
    Byte(u8),
    /// A value from the `Boolean` slot domain.
    Boolean(bool),
    /// A value from the `Unsigned4` slot domain.
    Unsigned4([u32; 4]),
    /// A value from the `Bytes4` slot domain.
    Bytes4([u8; 4]),
    /// A value from the `Texture` slot domain.
    Texture(Texture),
    /// A value from the `OptionalTexture` slot domain.
    OptionalTexture(Option<Texture>),
}
