//! Named physics choices and flags retain unrecognized wire values.
use crate::model::wem;

/// Named RigConvention choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RigConvention {
    #[default]
    /// PivotRelative.
    #[wem(value = 0)]
    PivotRelative,
    /// ExplicitBind.
    #[wem(value = 1)]
    ExplicitBind,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named JointMotion choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum JointMotion {
    #[default]
    /// Bend.
    #[wem(value = 0)]
    Bend,
    /// Twist.
    #[wem(value = 1)]
    Twist,
    /// Spin.
    #[wem(value = 2)]
    Spin,
    /// Slide.
    #[wem(value = 3)]
    Slide,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named JointAxis choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum JointAxis {
    #[default]
    /// X.
    #[wem(value = 0)]
    X,
    /// Y.
    #[wem(value = 1)]
    Y,
    /// Z.
    #[wem(value = 2)]
    Z,
    /// Along.
    #[wem(value = 3)]
    Along,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named TPoseSource choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum TPoseSource {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Donor.
    #[wem(value = 1)]
    Donor,
    /// Record.
    #[wem(value = 2)]
    Record,
    /// Clips.
    #[wem(value = 3)]
    Clips,
    /// BindFrame.
    #[wem(value = 4)]
    BindFrame,
    /// You.
    #[wem(value = 5)]
    You,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftWeights choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftWeights {
    #[default]
    /// Float.
    #[wem(value = 0)]
    Float,
    /// Bytes.
    #[wem(value = 1)]
    Bytes,
    /// EqualShares.
    #[wem(value = 2)]
    EqualShares,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftEffectKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftEffectKind {
    #[default]
    /// Inflate.
    #[wem(value = 0)]
    Inflate,
    /// Bulge.
    #[wem(value = 1)]
    Bulge,
    /// Jiggle.
    #[wem(value = 2)]
    Jiggle,
    /// Wobble.
    #[wem(value = 3)]
    Wobble,
    /// Blobify.
    #[wem(value = 4)]
    Blobify,
    /// Drip.
    #[wem(value = 5)]
    Drip,
    /// SquashStretch.
    #[wem(value = 6)]
    SquashStretch,
    /// Melt.
    #[wem(value = 7)]
    Melt,
    /// Ripple.
    #[wem(value = 8)]
    Ripple,
    /// Count.
    #[wem(value = 9)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftLumpKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftLumpKind {
    #[default]
    /// Bone.
    #[wem(value = 0)]
    Bone,
    /// Copy.
    #[wem(value = 1)]
    Copy,
    /// Surface.
    #[wem(value = 2)]
    Surface,
    /// Count.
    #[wem(value = 3)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftAlong choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftAlong {
    #[default]
    /// Surface.
    #[wem(value = 0)]
    Surface,
    /// Bones.
    #[wem(value = 1)]
    Bones,
    /// Count.
    #[wem(value = 2)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftAxis choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftAxis {
    #[default]
    /// Auto.
    #[wem(value = 0)]
    Auto,
    /// Up.
    #[wem(value = 1)]
    Up,
    /// X.
    #[wem(value = 2)]
    X,
    /// Y.
    #[wem(value = 3)]
    Y,
    /// Z.
    #[wem(value = 4)]
    Z,
    /// Bone.
    #[wem(value = 5)]
    Bone,
    /// Motion.
    #[wem(value = 6)]
    Motion,
    /// Count.
    #[wem(value = 7)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftComesFrom choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftComesFrom {
    #[default]
    /// Spans.
    #[wem(value = 0)]
    Spans,
    /// Times.
    #[wem(value = 1)]
    Times,
    /// Events.
    #[wem(value = 2)]
    Events,
    /// Contact.
    #[wem(value = 3)]
    Contact,
    /// Count.
    #[wem(value = 4)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftPulse choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftPulse {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Breathe.
    #[wem(value = 1)]
    Breathe,
    /// Heartbeat.
    #[wem(value = 2)]
    Heartbeat,
    /// Rate.
    #[wem(value = 3)]
    Rate,
    /// Count.
    #[wem(value = 4)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftBulgeShape choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftBulgeShape {
    #[default]
    /// Round.
    #[wem(value = 0)]
    Round,
    /// Ring.
    #[wem(value = 1)]
    Ring,
    /// Count.
    #[wem(value = 2)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftJigglePreset choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftJigglePreset {
    #[default]
    /// Custom.
    #[wem(value = 0)]
    Custom,
    /// Jelly.
    #[wem(value = 1)]
    Jelly,
    /// Fat.
    #[wem(value = 2)]
    Fat,
    /// Muscle.
    #[wem(value = 3)]
    Muscle,
    /// Rubber.
    #[wem(value = 4)]
    Rubber,
    /// WaterBalloon.
    #[wem(value = 5)]
    WaterBalloon,
    /// Water.
    #[wem(value = 6)]
    Water,
    /// Slime.
    #[wem(value = 7)]
    Slime,
    /// Count.
    #[wem(value = 8)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftWobblePattern choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftWobblePattern {
    #[default]
    /// Jelly.
    #[wem(value = 0)]
    Jelly,
    /// Sway.
    #[wem(value = 1)]
    Sway,
    /// Roll.
    #[wem(value = 2)]
    Roll,
    /// Random.
    #[wem(value = 3)]
    Random,
    /// Count.
    #[wem(value = 4)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftDripPreset choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftDripPreset {
    #[default]
    /// Custom.
    #[wem(value = 0)]
    Custom,
    /// Wax.
    #[wem(value = 1)]
    Wax,
    /// Slime.
    #[wem(value = 2)]
    Slime,
    /// Blood.
    #[wem(value = 3)]
    Blood,
    /// Paint.
    #[wem(value = 4)]
    Paint,
    /// Chocolate.
    #[wem(value = 5)]
    Chocolate,
    /// Honey.
    #[wem(value = 6)]
    Honey,
    /// Water.
    #[wem(value = 7)]
    Water,
    /// Count.
    #[wem(value = 8)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftNeckLaw choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftNeckLaw {
    #[default]
    /// Linear.
    #[wem(value = 0)]
    Linear,
    /// Snap.
    #[wem(value = 1)]
    Snap,
    /// Thread.
    #[wem(value = 2)]
    Thread,
    /// Sets.
    #[wem(value = 3)]
    Sets,
    /// Count.
    #[wem(value = 4)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftSquashMode choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftSquashMode {
    #[default]
    /// Auto.
    #[wem(value = 0)]
    Auto,
    /// Keyed.
    #[wem(value = 1)]
    Keyed,
    /// Count.
    #[wem(value = 2)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftAnchor choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftAnchor {
    #[default]
    /// Bottom.
    #[wem(value = 0)]
    Bottom,
    /// Centre.
    #[wem(value = 1)]
    Centre,
    /// Pivot.
    #[wem(value = 2)]
    Pivot,
    /// Count.
    #[wem(value = 3)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftMeltOrder choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftMeltOrder {
    #[default]
    /// Top.
    #[wem(value = 0)]
    Top,
    /// Bottom.
    #[wem(value = 1)]
    Bottom,
    /// AllAtOnce.
    #[wem(value = 2)]
    AllAtOnce,
    /// Count.
    #[wem(value = 3)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftRipplePlays choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftRipplePlays {
    #[default]
    /// Once.
    #[wem(value = 0)]
    Once,
    /// Loops.
    #[wem(value = 1)]
    Loops,
    /// Count.
    #[wem(value = 2)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftThrough choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftThrough {
    #[default]
    /// Skin.
    #[wem(value = 0)]
    Skin,
    /// Body.
    #[wem(value = 1)]
    Body,
    /// Count.
    #[wem(value = 2)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftCeiling choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftCeiling {
    #[default]
    /// Swing.
    #[wem(value = 0)]
    Swing,
    /// Clip.
    #[wem(value = 1)]
    Clip,
    /// None.
    #[wem(value = 2)]
    None,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named PhysicsShapeKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum PhysicsShapeKind {
    #[default]
    /// Box.
    #[wem(value = 0)]
    Box,
    /// Sphere.
    #[wem(value = 1)]
    Sphere,
    /// Capsule.
    #[wem(value = 2)]
    Capsule,
    /// Cylinder.
    #[wem(value = 3)]
    Cylinder,
    /// ConvexHull.
    #[wem(value = 4)]
    ConvexHull,
    /// TriangleMesh.
    #[wem(value = 5)]
    TriangleMesh,
    /// Count.
    #[wem(value = 6)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named BodyMotion choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum BodyMotion {
    #[default]
    /// Dynamic.
    #[wem(value = 0)]
    Dynamic,
    /// Kinematic.
    #[wem(value = 1)]
    Kinematic,
    /// Static.
    #[wem(value = 2)]
    Static,
    /// Count.
    #[wem(value = 3)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named JointKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum JointKind {
    #[default]
    /// Spherical.
    #[wem(value = 0)]
    Spherical,
    /// Revolute.
    #[wem(value = 1)]
    Revolute,
    /// ConeTwist.
    #[wem(value = 2)]
    ConeTwist,
    /// Weld.
    #[wem(value = 3)]
    Weld,
    /// Prismatic.
    #[wem(value = 4)]
    Prismatic,
    /// Distance.
    #[wem(value = 5)]
    Distance,
    /// Count.
    #[wem(value = 6)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named JointFriction choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum JointFriction {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Torque.
    #[wem(value = 1)]
    Torque,
    /// GravityHold.
    #[wem(value = 2)]
    GravityHold,
    /// Count.
    #[wem(value = 3)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named JointMotorMode choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum JointMotorMode {
    #[default]
    /// Off.
    #[wem(value = 0)]
    Off,
    /// Position.
    #[wem(value = 1)]
    Position,
    /// Velocity.
    #[wem(value = 2)]
    Velocity,
    /// Count.
    #[wem(value = 3)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ClothColliderKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ClothColliderKind {
    #[default]
    /// Capsule.
    #[wem(value = 0)]
    Capsule,
    /// Plane.
    #[wem(value = 1)]
    Plane,
    /// Count.
    #[wem(value = 2)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ClothCageKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ClothCageKind {
    #[default]
    /// Auto.
    #[wem(value = 0)]
    Auto,
    /// AsModelled.
    #[wem(value = 1)]
    AsModelled,
    /// Grid.
    #[wem(value = 2)]
    Grid,
    /// Strip.
    #[wem(value = 3)]
    Strip,
    /// Reduced.
    #[wem(value = 4)]
    Reduced,
    /// FromFaces.
    #[wem(value = 5)]
    FromFaces,
    /// Count.
    #[wem(value = 6)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ClothBakeInto choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ClothBakeInto {
    #[default]
    /// Bones.
    #[wem(value = 0)]
    Bones,
    /// FullDetail.
    #[wem(value = 1)]
    FullDetail,
    /// Count.
    #[wem(value = 2)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RigStart choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RigStart {
    #[default]
    /// Animated.
    #[wem(value = 0)]
    Animated,
    /// OnDeath.
    #[wem(value = 1)]
    OnDeath,
    /// Always.
    #[wem(value = 2)]
    Always,
    /// Never.
    #[wem(value = 3)]
    Never,
    /// Count.
    #[wem(value = 4)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named WowPhysicsKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum WowPhysicsKind {
    #[default]
    /// WornItem.
    #[wem(value = 0)]
    WornItem,
    /// Vegetation.
    #[wem(value = 1)]
    Vegetation,
    /// Ragdoll.
    #[wem(value = 2)]
    Ragdoll,
    /// PrivateWorld.
    #[wem(value = 3)]
    PrivateWorld,
    /// Count.
    #[wem(value = 4)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SoftFitMethod choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum SoftFitMethod {
    #[default]
    /// Fast.
    #[wem(value = 0)]
    Fast,
    /// Dense.
    #[wem(value = 1)]
    Dense,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named HingeSource choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum HingeSource {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Joint.
    #[wem(value = 1)]
    Joint,
    /// Clips.
    #[wem(value = 2)]
    Clips,
    /// Fallback.
    #[wem(value = 3)]
    Fallback,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named TPoseRefusal choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum TPoseRefusal {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// NoChain.
    #[wem(value = 1)]
    NoChain,
    /// ZeroLength.
    #[wem(value = 2)]
    ZeroLength,
    /// NoHinge.
    #[wem(value = 3)]
    NoHinge,
    /// JointLimits.
    #[wem(value = 4)]
    JointLimits,
    /// Unreachable.
    #[wem(value = 5)]
    Unreachable,
    /// Overlaps.
    #[wem(value = 6)]
    Overlaps,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named TPoseLanding choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum TPoseLanding {
    #[default]
    /// Refused.
    #[wem(value = 0)]
    Refused,
    /// OnTarget.
    #[wem(value = 1)]
    OnTarget,
    /// Off.
    #[wem(value = 2)]
    Off,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named TPoseScoreKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum TPoseScoreKind {
    #[default]
    /// Aim.
    #[wem(value = 0)]
    Aim,
    /// Heading.
    #[wem(value = 1)]
    Heading,
    /// Level.
    #[wem(value = 2)]
    Level,
    /// Roll.
    #[wem(value = 3)]
    Roll,
    /// Mirror.
    #[wem(value = 4)]
    Mirror,
    /// AnkleHeight.
    #[wem(value = 5)]
    AnkleHeight,
    /// Head.
    #[wem(value = 6)]
    Head,
    /// Hips.
    #[wem(value = 7)]
    Hips,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RebindCase choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RebindCase {
    #[default]
    /// NonUniformScale.
    #[wem(value = 0)]
    NonUniformScale,
    /// DontInherit.
    #[wem(value = 1)]
    DontInherit,
    /// Billboard.
    #[wem(value = 2)]
    Billboard,
    /// CollisionShape.
    #[wem(value = 3)]
    CollisionShape,
    /// GlobalSequence.
    #[wem(value = 4)]
    GlobalSequence,
    /// Count.
    #[wem(value = 5)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RigRole choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RigRole {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Body.
    #[wem(value = 1)]
    Body,
    /// Spine.
    #[wem(value = 2)]
    Spine,
    /// Neck.
    #[wem(value = 3)]
    Neck,
    /// Head.
    #[wem(value = 4)]
    Head,
    /// Clavicle.
    #[wem(value = 5)]
    Clavicle,
    /// Upper.
    #[wem(value = 6)]
    Upper,
    /// Lower.
    #[wem(value = 7)]
    Lower,
    /// Hock.
    #[wem(value = 8)]
    Hock,
    /// End.
    #[wem(value = 9)]
    End,
    /// Toe.
    #[wem(value = 10)]
    Toe,
    /// Twist.
    #[wem(value = 11)]
    Twist,
    /// Pad.
    #[wem(value = 12)]
    Pad,
    /// Prop.
    #[wem(value = 13)]
    Prop,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RigSide choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RigSide {
    #[default]
    /// Centre.
    #[wem(value = 0)]
    Centre,
    /// Left.
    #[wem(value = 1)]
    Left,
    /// Right.
    #[wem(value = 2)]
    Right,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RigLimb choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RigLimb {
    #[default]
    /// Other.
    #[wem(value = 0)]
    Other,
    /// Arm.
    #[wem(value = 1)]
    Arm,
    /// Leg.
    #[wem(value = 2)]
    Leg,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RigPlant choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RigPlant {
    #[default]
    /// ByRole.
    #[wem(value = 0)]
    ByRole,
    /// OnGround.
    #[wem(value = 1)]
    OnGround,
    /// Always.
    #[wem(value = 2)]
    Always,
    /// Free.
    #[wem(value = 3)]
    Free,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named RigSource choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum RigSource {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// File.
    #[wem(value = 1)]
    File,
    /// Name.
    #[wem(value = 2)]
    Name,
    /// Attachment.
    #[wem(value = 3)]
    Attachment,
    /// Shape.
    #[wem(value = 4)]
    Shape,
    /// You.
    #[wem(value = 5)]
    You,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named EnvelopeFalloff choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum EnvelopeFalloff {
    #[default]
    /// Hard.
    #[wem(value = 0)]
    Hard,
    /// Linear.
    #[wem(value = 1)]
    Linear,
    /// Smooth.
    #[wem(value = 2)]
    Smooth,
    /// Gaussian.
    #[wem(value = 3)]
    Gaussian,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named EnvelopePreset choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum EnvelopePreset {
    #[default]
    /// Mechanical.
    #[wem(value = 0)]
    Mechanical,
    /// Organic.
    #[wem(value = 1)]
    Organic,
    /// Hybrid.
    #[wem(value = 2)]
    Hybrid,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ForceKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ForceKind {
    #[default]
    /// Directional.
    #[wem(value = 0)]
    Directional,
    /// Radial.
    #[wem(value = 1)]
    Radial,
    /// Drag.
    #[wem(value = 2)]
    Drag,
    /// Vortex.
    #[wem(value = 3)]
    Vortex,
    /// Count.
    #[wem(value = 4)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ForceVolume choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u8)]
pub enum ForceVolume {
    #[default]
    /// Sphere.
    #[wem(value = 0)]
    Sphere,
    /// Cylinder.
    #[wem(value = 1)]
    Cylinder,
    /// Box.
    #[wem(value = 2)]
    Box,
    /// Hemisphere.
    #[wem(value = 3)]
    Hemisphere,
    /// Cone.
    #[wem(value = 4)]
    Cone,
    /// Count.
    #[wem(value = 5)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ForceFieldProperty choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = u32)]
pub enum ForceFieldProperty {
    #[default]
    /// Strength.
    #[wem(value = 0)]
    Strength,
    /// Width.
    #[wem(value = 1)]
    Width,
    /// Height.
    #[wem(value = 2)]
    Height,
    /// Length.
    #[wem(value = 3)]
    Length,
    /// Count.
    #[wem(value = 4)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}
