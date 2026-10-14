//! Force and collision categories independent of native payload records.

/// ForceKind authoring category.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ForceKind {
    #[default]
    /// Directional.
    Directional,
    /// Radial.
    Radial,
    /// Drag.
    Drag,
    /// Vortex.
    Vortex,
    /// Count.
    Count,
    /// Unrecognized authored category retained without interpretation.
    Unknown(u8),
}

/// ForceVolume authoring category.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ForceVolume {
    #[default]
    /// Sphere.
    Sphere,
    /// Cylinder.
    Cylinder,
    /// Box.
    Box,
    /// Hemisphere.
    Hemisphere,
    /// Cone.
    Cone,
    /// Count.
    Count,
    /// Unrecognized authored category retained without interpretation.
    Unknown(u8),
}

/// CollisionShapeKind authoring category.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CollisionShapeKind {
    #[default]
    /// Box.
    Box,
    /// Sphere.
    Sphere,
    /// Capsule.
    Capsule,
    /// Plane.
    Plane,
    /// Cylinder.
    Cylinder,
    /// Hull.
    Hull,
    /// Unrecognized authored category retained without interpretation.
    Unknown(u8),
}
