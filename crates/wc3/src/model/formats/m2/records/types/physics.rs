//! Native physics wire codes and flags.
use super::*;

/// Native PhysicsBodyKind code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u16)]
pub enum PhysicsBodyKind {
    #[default]
    /// Follows the model animation rather than dynamic simulation.
    #[m2(value = 0)]
    Kinematic,
    /// Moves under physics simulation.
    #[m2(value = 1)]
    Dynamic,
    /// Remains fixed in the simulation.
    #[m2(value = 2)]
    Static,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u16),
}

/// Native PhysicsShapeKind code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u16)]
pub enum PhysicsShapeKind {
    #[default]
    /// Selects geometry from the box table.
    #[m2(value = 0)]
    Box,
    /// Selects geometry from the capsule table.
    #[m2(value = 1)]
    Capsule,
    /// Selects geometry from the sphere table.
    #[m2(value = 2)]
    Sphere,
    /// Selects geometry from the convex-polytope table.
    #[m2(value = 3)]
    ConvexHull,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u16),
}

/// Native PhysicsJointKind code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u16)]
pub enum PhysicsJointKind {
    #[default]
    /// Constrains bodies through a spherical joint.
    #[m2(value = 0)]
    Spherical,
    /// Constrains cone and twist rotation.
    #[m2(value = 1)]
    Shoulder,
    /// Fixes bodies relative to each other.
    #[m2(value = 2)]
    Weld,
    /// Constrains rotation around a hinge axis.
    #[m2(value = 3)]
    Revolute,
    /// Constrains translation along a sliding axis.
    #[m2(value = 4)]
    Prismatic,
    /// Constrains the distance between two anchors.
    #[m2(value = 5)]
    Distance,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u16),
}

bitfield! {
    /// Native PhysicsGameFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout, m2::Validate, m2::VisitArrays)]
    pub struct PhysicsGameFlags(u16);
    /// Exact stored bits.
    pub bits, _: 15, 0;
}
impl PhysicsGameFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<u16> for PhysicsGameFlags {
    fn from(bits: u16) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<PhysicsGameFlags> for u16 {
    fn from(flags: PhysicsGameFlags) -> Self {
        flags.bits()
    }
}
wire_ule!(PhysicsBodyKind, u16, raw, from_raw);
wire_ule!(PhysicsShapeKind, u16, raw, from_raw);
wire_ule!(PhysicsJointKind, u16, raw, from_raw);
wire_ule!(PhysicsGameFlags, u16, bits, from_bits_retain);

bitfield! {
    /// Native PhysicsAttachment; unknown bits are retained.
    #[derive(Clone,Copy,Debug,Default,Eq,PartialEq,Ownership,m2::Read,m2::Write,m2::RecordLayout,m2::Validate,m2::VisitArrays)]
    pub struct PhysicsAttachment(u16);
    /// Exact stored bits.
    pub bits, _: 15,0;
    /// Parent body identity in the low fourteen bits.
    pub parent_body, set_parent_body: 13, 0;
    /// Attached body host.
    pub attachment_host, set_attachment_host: 15;
    /// Ragdoll root marker.
    pub ragdoll_root, set_ragdoll_root: 14;
}
impl PhysicsAttachment {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
wire_ule!(PhysicsAttachment, u16, bits, from_bits_retain);

/// Native PHYT simulation domain.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value=u32)]
pub enum PhysicsKind {
    /// Clothing and equipment simulation.
    #[default]
    #[m2(value = 0)]
    WornItem,
    /// Vegetation simulation.
    #[m2(value = 1)]
    Vegetation,
    /// Ragdoll simulation.
    #[m2(value = 2)]
    Ragdoll,
    /// Private world simulation.
    #[m2(value = 3)]
    PrivateWorld,
    /// Unrecognized native domain.
    #[m2(unknown)]
    Unknown(u32),
}
wire_ule!(PhysicsKind, u32, raw, from_raw);

/// Native physics joint motor selection.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value=u32)]
pub enum PhysicsMotorMode {
    /// Motor disabled.
    #[default]
    #[m2(value = 0)]
    Off,
    /// Position control.
    #[m2(value = 1)]
    Position,
    /// Velocity control.
    #[m2(value = 2)]
    Velocity,
    /// Unrecognized native motor code.
    #[m2(unknown)]
    Unknown(u32),
}
wire_ule!(PhysicsMotorMode, u32, raw, from_raw);
