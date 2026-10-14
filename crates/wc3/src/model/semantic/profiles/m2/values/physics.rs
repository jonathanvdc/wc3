//! Native physics wire codes and flags.
use bitfield::bitfield;

/// Native PhysicsBodyKind code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PhysicsBodyKind {
    #[default]
    /// Follows the model animation rather than dynamic simulation.
    Kinematic,
    /// Moves under physics simulation.
    Dynamic,
    /// Remains fixed in the simulation.
    Static,
    /// Unrecognized native code.
    Unknown(u16),
}

/// Native PhysicsShapeKind code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PhysicsShapeKind {
    #[default]
    /// Selects geometry from the box table.
    Box,
    /// Selects geometry from the capsule table.
    Capsule,
    /// Selects geometry from the sphere table.
    Sphere,
    /// Selects geometry from the convex-polytope table.
    ConvexHull,
    /// Unrecognized native code.
    Unknown(u16),
}

/// Native PhysicsJointKind code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PhysicsJointKind {
    #[default]
    /// Constrains bodies through a spherical joint.
    Spherical,
    /// Constrains cone and twist rotation.
    Shoulder,
    /// Fixes bodies relative to each other.
    Weld,
    /// Constrains rotation around a hinge axis.
    Revolute,
    /// Constrains translation along a sliding axis.
    Prismatic,
    /// Constrains the distance between two anchors.
    Distance,
    /// Unrecognized native code.
    Unknown(u16),
}

bitfield! {
    /// Native PhysicsGameFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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

bitfield! {
    /// Native PhysicsAttachment; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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

/// Native PHYT simulation domain.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PhysicsKind {
    /// Clothing and equipment simulation.
    #[default]
    WornItem,
    /// Vegetation simulation.
    Vegetation,
    /// Ragdoll simulation.
    Ragdoll,
    /// Private world simulation.
    PrivateWorld,
    /// Unrecognized native domain.
    Unknown(u32),
}

/// Native physics joint motor selection.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PhysicsMotorMode {
    /// Motor disabled.
    #[default]
    Off,
    /// Position control.
    Position,
    /// Velocity control.
    Velocity,
    /// Unrecognized native motor code.
    Unknown(u32),
}

impl PhysicsBodyKind {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Kinematic,
            1 => Self::Dynamic,
            2 => Self::Static,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Kinematic => 0,
            Self::Dynamic => 1,
            Self::Static => 2,
            Self::Unknown(value) => value,
        }
    }
}

impl PhysicsShapeKind {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Box,
            1 => Self::Capsule,
            2 => Self::Sphere,
            3 => Self::ConvexHull,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Box => 0,
            Self::Capsule => 1,
            Self::Sphere => 2,
            Self::ConvexHull => 3,
            Self::Unknown(value) => value,
        }
    }
}

impl PhysicsJointKind {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Spherical,
            1 => Self::Shoulder,
            2 => Self::Weld,
            3 => Self::Revolute,
            4 => Self::Prismatic,
            5 => Self::Distance,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Spherical => 0,
            Self::Shoulder => 1,
            Self::Weld => 2,
            Self::Revolute => 3,
            Self::Prismatic => 4,
            Self::Distance => 5,
            Self::Unknown(value) => value,
        }
    }
}

impl PhysicsKind {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::WornItem,
            1 => Self::Vegetation,
            2 => Self::Ragdoll,
            3 => Self::PrivateWorld,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u32 {
        match self {
            Self::WornItem => 0,
            Self::Vegetation => 1,
            Self::Ragdoll => 2,
            Self::PrivateWorld => 3,
            Self::Unknown(value) => value,
        }
    }
}

impl PhysicsMotorMode {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Off,
            1 => Self::Position,
            2 => Self::Velocity,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Off => 0,
            Self::Position => 1,
            Self::Velocity => 2,
            Self::Unknown(value) => value,
        }
    }
}
