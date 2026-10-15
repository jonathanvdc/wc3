//! M3 physics selectors and bit masks.
use bitfield::bitfield;

/// M3 PhysicsShapeKind selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PhysicsShapeKind {
    /// Selects Box.
    Box,
    /// Selects Sphere.
    Sphere,
    /// Selects Capsule.
    Capsule,
    /// Selects Cylinder.
    Cylinder,
    /// Selects ConvexHull.
    ConvexHull,
    /// Selects TriangleMesh.
    TriangleMesh,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u8),
}
impl PhysicsShapeKind {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u8 {
        match self {
            Self::Box => 0,
            Self::Sphere => 1,
            Self::Capsule => 2,
            Self::Cylinder => 3,
            Self::ConvexHull => 4,
            Self::TriangleMesh => 5,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Box,
            1 => Self::Sphere,
            2 => Self::Capsule,
            3 => Self::Cylinder,
            4 => Self::ConvexHull,
            5 => Self::TriangleMesh,
            value => Self::Unknown(value),
        }
    }
}

impl Default for PhysicsShapeKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 JointKind selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum JointKind {
    /// Selects Spherical.
    Spherical,
    /// Selects Revolute.
    Revolute,
    /// Selects Cone.
    Cone,
    /// Selects Weld.
    Weld,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl JointKind {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Spherical => 0,
            Self::Revolute => 1,
            Self::Cone => 2,
            Self::Weld => 3,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Spherical,
            1 => Self::Revolute,
            2 => Self::Cone,
            3 => Self::Weld,
            value => Self::Unknown(value),
        }
    }
}

impl Default for JointKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 PhysicalMaterial selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum PhysicalMaterial {
    /// Selects Bone.
    Bone,
    /// Selects HeavyCloth.
    HeavyCloth,
    /// Selects LightCloth.
    LightCloth,
    /// Selects Creep.
    Creep,
    /// Selects Dirt.
    Dirt,
    /// Selects EnergyShield.
    EnergyShield,
    /// Selects Flesh.
    Flesh,
    /// Selects Hair.
    Hair,
    /// Selects Ice.
    Ice,
    /// Selects Lava.
    Lava,
    /// Selects LightArmor.
    LightArmor,
    /// Selects HeavyMetal.
    HeavyMetal,
    /// Selects LightMetal.
    LightMetal,
    /// Selects ProtossMetal.
    ProtossMetal,
    /// Selects Paper.
    Paper,
    /// Selects Plastic.
    Plastic,
    /// Selects Rock.
    Rock,
    /// Selects Rubber.
    Rubber,
    /// Selects Sand.
    Sand,
    /// Selects Snow.
    Snow,
    /// Selects Water.
    Water,
    /// Selects Wood.
    Wood,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl PhysicalMaterial {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Bone => 0,
            Self::HeavyCloth => 1,
            Self::LightCloth => 2,
            Self::Creep => 3,
            Self::Dirt => 4,
            Self::EnergyShield => 5,
            Self::Flesh => 6,
            Self::Hair => 7,
            Self::Ice => 8,
            Self::Lava => 9,
            Self::LightArmor => 10,
            Self::HeavyMetal => 11,
            Self::LightMetal => 12,
            Self::ProtossMetal => 13,
            Self::Paper => 14,
            Self::Plastic => 15,
            Self::Rock => 16,
            Self::Rubber => 17,
            Self::Sand => 18,
            Self::Snow => 19,
            Self::Water => 20,
            Self::Wood => 21,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Bone,
            1 => Self::HeavyCloth,
            2 => Self::LightCloth,
            3 => Self::Creep,
            4 => Self::Dirt,
            5 => Self::EnergyShield,
            6 => Self::Flesh,
            7 => Self::Hair,
            8 => Self::Ice,
            9 => Self::Lava,
            10 => Self::LightArmor,
            11 => Self::HeavyMetal,
            12 => Self::LightMetal,
            13 => Self::ProtossMetal,
            14 => Self::Paper,
            15 => Self::Plastic,
            16 => Self::Rock,
            17 => Self::Rubber,
            18 => Self::Sand,
            19 => Self::Snow,
            20 => Self::Water,
            21 => Self::Wood,
            value => Self::Unknown(value),
        }
    }
}

impl Default for PhysicalMaterial {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// M3 RigidBodyFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct RigidBodyFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the collidable control.
    pub collidable, set_collidable: 0;
    /// Returns or changes the walkable control.
    pub walkable, set_walkable: 1;
    /// Returns or changes the stackable control.
    pub stackable, set_stackable: 2;
    /// Returns or changes the simulate collision control.
    pub simulate_collision, set_simulate_collision: 3;
    /// Returns or changes the ignore local bodies control.
    pub ignore_local_bodies, set_ignore_local_bodies: 4;
    /// Returns or changes the always exists control.
    pub always_exists, set_always_exists: 5;
    /// Returns or changes the no simulation control.
    pub no_simulation, set_no_simulation: 7;
}
impl From<u32> for RigidBodyFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<RigidBodyFlags> for u32 {
    fn from(value: RigidBodyFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// M3 ConstraintFlags control mask; every unknown bit is preserved.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct ConstraintFlags(u32);
    /// Returns the complete encoded mask.
    pub bits, _: 31, 0;
}
impl From<u32> for ConstraintFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ConstraintFlags> for u32 {
    fn from(value: ConstraintFlags) -> Self {
        value.bits()
    }
}
