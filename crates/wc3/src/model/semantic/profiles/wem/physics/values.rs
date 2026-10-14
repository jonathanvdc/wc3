//! Collision and constraint interpretations admitted by WEM.

/// ShapeKind interpretation, retaining unnamed authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShapeKind {
    #[default]
    /// Box.
    Box,
    /// Sphere.
    Sphere,
    /// Capsule.
    Capsule,
    /// Cylinder.
    Cylinder,
    /// ConvexHull.
    ConvexHull,
    /// TriangleMesh.
    TriangleMesh,
    /// Count.
    Count,
    /// Unrecognized authored interpretation.
    Unknown(u8),
}

/// Motion interpretation, retaining unnamed authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Motion {
    #[default]
    /// Dynamic.
    Dynamic,
    /// Kinematic.
    Kinematic,
    /// Static.
    Static,
    /// Count.
    Count,
    /// Unrecognized authored interpretation.
    Unknown(u8),
}

/// JointKind interpretation, retaining unnamed authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JointKind {
    #[default]
    /// Spherical.
    Spherical,
    /// Revolute.
    Revolute,
    /// ConeTwist.
    ConeTwist,
    /// Weld.
    Weld,
    /// Prismatic.
    Prismatic,
    /// Distance.
    Distance,
    /// Count.
    Count,
    /// Unrecognized authored interpretation.
    Unknown(u8),
}

/// Friction interpretation, retaining unnamed authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Friction {
    #[default]
    /// None.
    None,
    /// Torque.
    Torque,
    /// GravityHold.
    GravityHold,
    /// Count.
    Count,
    /// Unrecognized authored interpretation.
    Unknown(u8),
}

/// Motor interpretation, retaining unnamed authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Motor {
    #[default]
    /// Off.
    Off,
    /// Position.
    Position,
    /// Velocity.
    Velocity,
    /// Count.
    Count,
    /// Unrecognized authored interpretation.
    Unknown(u8),
}

/// ClothColliderKind authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClothColliderKind {
    #[default]
    /// Capsule.
    Capsule,
    /// Plane.
    Plane,
    /// Count.
    Count,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// ClothCageKind authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClothCageKind {
    #[default]
    /// Auto.
    Auto,
    /// AsModelled.
    AsModelled,
    /// Grid.
    Grid,
    /// Strip.
    Strip,
    /// Reduced.
    Reduced,
    /// FromFaces.
    FromFaces,
    /// Count.
    Count,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}

/// ClothBakeInto authoring convention.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ClothBakeInto {
    #[default]
    /// Bones.
    Bones,
    /// FullDetail.
    FullDetail,
    /// Count.
    Count,
    /// Unrecognized authored convention retained without interpretation.
    Unknown(u8),
}
