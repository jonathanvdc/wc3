//! Preparation choices; unknown producer values remain explicit.
/// Authored RigStart behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RigStart {
    #[default]
    /// Animated.
    Animated,
    /// OnDeath.
    OnDeath,
    /// Always.
    Always,
    /// Never.
    Never,
    /// Count.
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}
/// Authored WowPhysicsKind behavior.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WowPhysicsKind {
    #[default]
    /// WornItem.
    WornItem,
    /// Vegetation.
    Vegetation,
    /// Ragdoll.
    Ragdoll,
    /// PrivateWorld.
    PrivateWorld,
    /// Count.
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}
/// Angular freedom assigned to ragdoll joints.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum JointRange {
    /// Restricted motion.
    #[default]
    Stiff,
    /// Normal motion limits.
    Normal,
    /// Expanded motion limits.
    Loose,
    /// Unrecognized authored choice, preserved without interpreting its behavior.
    Unknown(u8),
}
/// Preferred orientation when generating fracture pieces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GrainAxis {
    /// No preferred fracture axis.
    #[default]
    None,
    /// Grain follows the X axis.
    X,
    /// Grain follows the Y axis.
    Y,
    /// Grain follows the Z axis.
    Z,
    /// Unrecognized authored choice, preserved without interpreting its behavior.
    Unknown(u8),
}
/// Treatment of source surfaces with open boundaries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OpenParts {
    /// Treat open geometry as a closed solid.
    #[default]
    ClosedSolid,
    /// Thicken open geometry before cutting.
    Thicken,
    /// Unrecognized authored choice, preserved without interpreting its behavior.
    Unknown(u8),
}
/// Procedure for generating piece geometry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FractureMethod {
    /// Generate fracture pieces.
    #[default]
    Fracture,
    /// Cut pieces using slicing planes.
    Slices,
    /// Unrecognized authored choice, preserved without interpreting its behavior.
    Unknown(u8),
}
/// Force applied when fracture pieces activate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PushMode {
    /// Push pieces away from the activation center.
    #[default]
    Blast,
    /// Pull pieces toward the activation center.
    Implode,
    /// Apply no push force.
    None,
    /// Unrecognized authored choice, preserved without interpreting its behavior.
    Unknown(u8),
}
/// Order in which fracture pieces become active.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ReleaseOrder {
    /// Activate every piece together.
    #[default]
    Together,
    /// Activate lower pieces first.
    BottomUp,
    /// Activate upper pieces first.
    TopDown,
    /// Activate pieces from the center outward.
    Outward,
    /// Unrecognized authored choice, preserved without interpreting its behavior.
    Unknown(u8),
}
/// Texture-coordinate layout on generated interior surfaces.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InteriorUvs {
    /// Tile the interior texture.
    #[default]
    Tiled,
    /// Lay out interiors in a square texture map.
    Square,
    /// Unrecognized authored choice, preserved without interpreting its behavior.
    Unknown(u8),
}
/// Geometry visible before fracture activation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BeforeActivation {
    /// Choose the pre-activation geometry automatically.
    #[default]
    Auto,
    /// Show the unfractured geometry.
    Whole,
    /// Show the generated pieces.
    Pieces,
    /// Unrecognized authored choice, preserved without interpreting its behavior.
    Unknown(u8),
}
/// Orientation used by a set of slicing planes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliceAlong {
    /// Slice perpendicular to the X axis.
    #[default]
    X,
    /// Slice perpendicular to the Y axis.
    Y,
    /// Slice perpendicular to the Z axis.
    Z,
    /// Use the explicitly authored direction vector.
    Direction,
    /// Unrecognized authored choice, preserved without interpreting its behavior.
    Unknown(u8),
}
