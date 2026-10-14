//! Native physics selectors and bit masks.
use crate::model::m3;
use bitfield::bitfield;

/// Native PhysicsShapeKind selector.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
    m3::Value,
)]
#[m3(value = u8)]
pub enum PhysicsShapeKind {
    /// Selects Box.
    #[m3(value = 0)]
    Box,
    /// Selects Sphere.
    #[m3(value = 1)]
    Sphere,
    /// Selects Capsule.
    #[m3(value = 2)]
    Capsule,
    /// Selects Cylinder.
    #[m3(value = 3)]
    Cylinder,
    /// Selects ConvexHull.
    #[m3(value = 4)]
    ConvexHull,
    /// Selects TriangleMesh.
    #[m3(value = 5)]
    TriangleMesh,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u8),
}
impl Default for PhysicsShapeKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native JointKind selector.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
    m3::Value,
)]
#[m3(value = u32)]
pub enum JointKind {
    /// Selects Spherical.
    #[m3(value = 0)]
    Spherical,
    /// Selects Revolute.
    #[m3(value = 1)]
    Revolute,
    /// Selects Cone.
    #[m3(value = 2)]
    Cone,
    /// Selects Weld.
    #[m3(value = 3)]
    Weld,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for JointKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native PhysicalMaterial selector.
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Hash,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
    m3::Value,
)]
#[m3(value = u32)]
pub enum PhysicalMaterial {
    /// Selects Bone.
    #[m3(value = 0)]
    Bone,
    /// Selects HeavyCloth.
    #[m3(value = 1)]
    HeavyCloth,
    /// Selects LightCloth.
    #[m3(value = 2)]
    LightCloth,
    /// Selects Creep.
    #[m3(value = 3)]
    Creep,
    /// Selects Dirt.
    #[m3(value = 4)]
    Dirt,
    /// Selects EnergyShield.
    #[m3(value = 5)]
    EnergyShield,
    /// Selects Flesh.
    #[m3(value = 6)]
    Flesh,
    /// Selects Hair.
    #[m3(value = 7)]
    Hair,
    /// Selects Ice.
    #[m3(value = 8)]
    Ice,
    /// Selects Lava.
    #[m3(value = 9)]
    Lava,
    /// Selects LightArmor.
    #[m3(value = 10)]
    LightArmor,
    /// Selects HeavyMetal.
    #[m3(value = 11)]
    HeavyMetal,
    /// Selects LightMetal.
    #[m3(value = 12)]
    LightMetal,
    /// Selects ProtossMetal.
    #[m3(value = 13)]
    ProtossMetal,
    /// Selects Paper.
    #[m3(value = 14)]
    Paper,
    /// Selects Plastic.
    #[m3(value = 15)]
    Plastic,
    /// Selects Rock.
    #[m3(value = 16)]
    Rock,
    /// Selects Rubber.
    #[m3(value = 17)]
    Rubber,
    /// Selects Sand.
    #[m3(value = 18)]
    Sand,
    /// Selects Snow.
    #[m3(value = 19)]
    Snow,
    /// Selects Water.
    #[m3(value = 20)]
    Water,
    /// Selects Wood.
    #[m3(value = 21)]
    Wood,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for PhysicalMaterial {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// Native RigidBodyFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
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
    /// Native ConstraintFlags control mask; every unknown bit is preserved.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
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
