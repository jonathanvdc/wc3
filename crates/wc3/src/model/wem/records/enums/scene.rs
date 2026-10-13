//! Named scene choices and flags retain unrecognized wire values.
use crate::model::wem;
use bitfield::bitfield;

/// Named NodePresence choices; unrecognized codes remain representable.
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
pub enum NodePresence {
    #[default]
    /// Present.
    #[wem(value = 0)]
    Present,
    /// Placement.
    #[wem(value = 1)]
    Placement,
    /// Absent.
    #[wem(value = 2)]
    Absent,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named NodeKind choices; unrecognized codes remain representable.
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
pub enum NodeKind {
    #[default]
    /// Helper.
    #[wem(value = 0)]
    Helper,
    /// Bone.
    #[wem(value = 1)]
    Bone,
    /// Attachment.
    #[wem(value = 2)]
    Attachment,
    /// Light.
    #[wem(value = 3)]
    Light,
    /// Camera.
    #[wem(value = 4)]
    Camera,
    /// ParticleEmitter.
    #[wem(value = 5)]
    ParticleEmitter,
    /// RibbonEmitter.
    #[wem(value = 6)]
    RibbonEmitter,
    /// Event.
    #[wem(value = 7)]
    Event,
    /// CollisionShape.
    #[wem(value = 8)]
    CollisionShape,
    /// Wc3ParticleEmitter1.
    #[wem(value = 9)]
    Wc3ParticleEmitter1,
    /// Wc3ParticleEmitter2.
    #[wem(value = 10)]
    Wc3ParticleEmitter2,
    /// Wc3RibbonEmitter.
    #[wem(value = 11)]
    Wc3RibbonEmitter,
    /// Sc2ParticleEmitter.
    #[wem(value = 12)]
    Sc2ParticleEmitter,
    /// Sc2RibbonEmitter.
    #[wem(value = 13)]
    Sc2RibbonEmitter,
    /// Wc3CornEmitter.
    #[wem(value = 14)]
    Wc3CornEmitter,
    /// M2ParticleEmitter.
    #[wem(value = 15)]
    M2ParticleEmitter,
    /// ForceField.
    #[wem(value = 16)]
    ForceField,
    /// VertexWarp.
    #[wem(value = 17)]
    VertexWarp,
    /// Wc3FaceFx.
    #[wem(value = 18)]
    Wc3FaceFx,
    /// Count.
    #[wem(value = 19)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

bitfield! {
    /// NodeFlags bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct NodeFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the Hidden bit.
    pub hidden, set_hidden: 0;
    /// Returns or changes the DontInheritTranslation bit.
    pub dont_inherit_translation, set_dont_inherit_translation: 1;
    /// Returns or changes the DontInheritRotation bit.
    pub dont_inherit_rotation, set_dont_inherit_rotation: 2;
    /// Returns or changes the DontInheritScale bit.
    pub dont_inherit_scale, set_dont_inherit_scale: 3;
    /// Returns or changes the Billboarded bit.
    pub billboarded, set_billboarded: 4;
    /// Returns or changes the BillboardLockX bit.
    pub billboard_lock_x, set_billboard_lock_x: 5;
    /// Returns or changes the BillboardLockY bit.
    pub billboard_lock_y, set_billboard_lock_y: 6;
    /// Returns or changes the BillboardLockZ bit.
    pub billboard_lock_z, set_billboard_lock_z: 7;
    /// Returns or changes the ModelSpace bit.
    pub model_space, set_model_space: 8;
}

impl NodeFlags {
    /// Retains every stored bit, including flags without known names.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
    /// Bit mask for None.
    pub const NONE: Self = Self(0);
    /// Bit mask for Hidden.
    pub const HIDDEN: Self = Self(1);
    /// Bit mask for DontInheritTranslation.
    pub const DONT_INHERIT_TRANSLATION: Self = Self(2);
    /// Bit mask for DontInheritRotation.
    pub const DONT_INHERIT_ROTATION: Self = Self(4);
    /// Bit mask for DontInheritScale.
    pub const DONT_INHERIT_SCALE: Self = Self(8);
    /// Bit mask for Billboarded.
    pub const BILLBOARDED: Self = Self(16);
    /// Bit mask for BillboardLockX.
    pub const BILLBOARD_LOCK_X: Self = Self(32);
    /// Bit mask for BillboardLockY.
    pub const BILLBOARD_LOCK_Y: Self = Self(64);
    /// Bit mask for BillboardLockZ.
    pub const BILLBOARD_LOCK_Z: Self = Self(128);
    /// Bit mask for ModelSpace.
    pub const MODEL_SPACE: Self = Self(256);
}
impl From<u32> for NodeFlags {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<NodeFlags> for u32 {
    fn from(value: NodeFlags) -> Self {
        value.bits()
    }
}

/// Named LightKind choices; unrecognized codes remain representable.
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
pub enum LightKind {
    #[default]
    /// Omni.
    #[wem(value = 0)]
    Omni,
    /// Directional.
    #[wem(value = 1)]
    Directional,
    /// Spot.
    #[wem(value = 2)]
    Spot,
    /// Ambient.
    #[wem(value = 3)]
    Ambient,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named CollisionShapeKind choices; unrecognized codes remain representable.
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
pub enum CollisionShapeKind {
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
    /// Plane.
    #[wem(value = 3)]
    Plane,
    /// Cylinder.
    #[wem(value = 4)]
    Cylinder,
    /// Hull.
    #[wem(value = 5)]
    Hull,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named EmitterLink choices; unrecognized codes remain representable.
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
pub enum EmitterLink {
    #[default]
    /// CopySource.
    #[wem(value = 0)]
    CopySource,
    /// CollisionSpawn.
    #[wem(value = 1)]
    CollisionSpawn,
    /// Trail.
    #[wem(value = 2)]
    Trail,
    /// BounceRibbon.
    #[wem(value = 3)]
    BounceRibbon,
    /// SplineBone.
    #[wem(value = 4)]
    SplineBone,
    /// SkinMirror.
    #[wem(value = 5)]
    SkinMirror,
    /// RigRidesWith.
    #[wem(value = 6)]
    RigRidesWith,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Wc3Particle1Property choices; unrecognized codes remain representable.
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
pub enum Wc3Particle1Property {
    #[default]
    /// EmissionRate.
    #[wem(value = 0)]
    EmissionRate,
    /// Gravity.
    #[wem(value = 1)]
    Gravity,
    /// Longitude.
    #[wem(value = 2)]
    Longitude,
    /// Latitude.
    #[wem(value = 3)]
    Latitude,
    /// Lifespan.
    #[wem(value = 4)]
    Lifespan,
    /// Speed.
    #[wem(value = 5)]
    Speed,
    /// Count.
    #[wem(value = 6)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Wc3Particle2Property choices; unrecognized codes remain representable.
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
pub enum Wc3Particle2Property {
    #[default]
    /// Speed.
    #[wem(value = 0)]
    Speed,
    /// Variation.
    #[wem(value = 1)]
    Variation,
    /// Latitude.
    #[wem(value = 2)]
    Latitude,
    /// Gravity.
    #[wem(value = 3)]
    Gravity,
    /// EmissionRate.
    #[wem(value = 4)]
    EmissionRate,
    /// Width.
    #[wem(value = 5)]
    Width,
    /// Length.
    #[wem(value = 6)]
    Length,
    /// Count.
    #[wem(value = 7)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Wc3RibbonProperty choices; unrecognized codes remain representable.
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
pub enum Wc3RibbonProperty {
    #[default]
    /// HeightAbove.
    #[wem(value = 0)]
    HeightAbove,
    /// HeightBelow.
    #[wem(value = 1)]
    HeightBelow,
    /// Count.
    #[wem(value = 2)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Wc3CornProperty choices; unrecognized codes remain representable.
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
pub enum Wc3CornProperty {
    #[default]
    /// Lifespan.
    #[wem(value = 0)]
    Lifespan,
    /// EmissionRate.
    #[wem(value = 1)]
    EmissionRate,
    /// Speed.
    #[wem(value = 2)]
    Speed,
    /// Count.
    #[wem(value = 3)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Sc2ParticleProperty choices; unrecognized codes remain representable.
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
pub enum Sc2ParticleProperty {
    #[default]
    /// InitialSpeed.
    #[wem(value = 0)]
    InitialSpeed,
    /// InitialSpeedRandom.
    #[wem(value = 1)]
    InitialSpeedRandom,
    /// InitialYaw.
    #[wem(value = 2)]
    InitialYaw,
    /// InitialPitch.
    #[wem(value = 3)]
    InitialPitch,
    /// InitialHorizontal.
    #[wem(value = 4)]
    InitialHorizontal,
    /// InitialVertical.
    #[wem(value = 5)]
    InitialVertical,
    /// Lifetime.
    #[wem(value = 6)]
    Lifetime,
    /// LifetimeRandom.
    #[wem(value = 7)]
    LifetimeRandom,
    /// SizeAnimation.
    #[wem(value = 8)]
    SizeAnimation,
    /// RotationAnimation.
    #[wem(value = 9)]
    RotationAnimation,
    /// ColorStart.
    #[wem(value = 10)]
    ColorStart,
    /// ColorMid.
    #[wem(value = 11)]
    ColorMid,
    /// ColorEnd.
    #[wem(value = 12)]
    ColorEnd,
    /// EmissionRate.
    #[wem(value = 13)]
    EmissionRate,
    /// ShapeOuter.
    #[wem(value = 14)]
    ShapeOuter,
    /// ShapeInner.
    #[wem(value = 15)]
    ShapeInner,
    /// OuterRadius.
    #[wem(value = 16)]
    OuterRadius,
    /// InnerRadius.
    #[wem(value = 17)]
    InnerRadius,
    /// SizeRandomAnimation.
    #[wem(value = 18)]
    SizeRandomAnimation,
    /// RotationRandomAnimation.
    #[wem(value = 19)]
    RotationRandomAnimation,
    /// ColorStartRandom.
    #[wem(value = 20)]
    ColorStartRandom,
    /// ColorMidRandom.
    #[wem(value = 21)]
    ColorMidRandom,
    /// ColorEndRandom.
    #[wem(value = 22)]
    ColorEndRandom,
    /// SquirtAmount.
    #[wem(value = 23)]
    SquirtAmount,
    /// PitchAmplitude.
    #[wem(value = 24)]
    PitchAmplitude,
    /// PitchFrequency.
    #[wem(value = 25)]
    PitchFrequency,
    /// YawAmplitude.
    #[wem(value = 26)]
    YawAmplitude,
    /// YawFrequency.
    #[wem(value = 27)]
    YawFrequency,
    /// SpeedAmplitude.
    #[wem(value = 28)]
    SpeedAmplitude,
    /// SpeedFrequency.
    #[wem(value = 29)]
    SpeedFrequency,
    /// SizeAmplitude.
    #[wem(value = 30)]
    SizeAmplitude,
    /// SizeFrequency.
    #[wem(value = 31)]
    SizeFrequency,
    /// AlphaAmplitude.
    #[wem(value = 32)]
    AlphaAmplitude,
    /// AlphaFrequency.
    #[wem(value = 33)]
    AlphaFrequency,
    /// ColorAmplitude.
    #[wem(value = 34)]
    ColorAmplitude,
    /// ColorFrequency.
    #[wem(value = 35)]
    ColorFrequency,
    /// RotationAmplitude.
    #[wem(value = 36)]
    RotationAmplitude,
    /// RotationFrequency.
    #[wem(value = 37)]
    RotationFrequency,
    /// HorizontalAmplitude.
    #[wem(value = 38)]
    HorizontalAmplitude,
    /// HorizontalFrequency.
    #[wem(value = 39)]
    HorizontalFrequency,
    /// VerticalAmplitude.
    #[wem(value = 40)]
    VerticalAmplitude,
    /// VerticalFrequency.
    #[wem(value = 41)]
    VerticalFrequency,
    /// ParticleVelocity.
    #[wem(value = 42)]
    ParticleVelocity,
    /// PhaseShift.
    #[wem(value = 43)]
    PhaseShift,
    /// AlphaThreshold.
    #[wem(value = 44)]
    AlphaThreshold,
    /// UvOffset.
    #[wem(value = 45)]
    UvOffset,
    /// UvAngle.
    #[wem(value = 46)]
    UvAngle,
    /// UvTiling.
    #[wem(value = 47)]
    UvTiling,
    /// SplinePoint.
    #[wem(value = 48)]
    SplinePoint,
    /// LowerBound.
    #[wem(value = 49)]
    LowerBound,
    /// UpperBound.
    #[wem(value = 50)]
    UpperBound,
    /// TrailEmissionRate.
    #[wem(value = 51)]
    TrailEmissionRate,
    /// Count.
    #[wem(value = 52)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Sc2RibbonProperty choices; unrecognized codes remain representable.
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
pub enum Sc2RibbonProperty {
    #[default]
    /// InitialSpeed.
    #[wem(value = 0)]
    InitialSpeed,
    /// InitialSpeedRandom.
    #[wem(value = 1)]
    InitialSpeedRandom,
    /// InitialYaw.
    #[wem(value = 2)]
    InitialYaw,
    /// InitialPitch.
    #[wem(value = 3)]
    InitialPitch,
    /// InitialHorizontal.
    #[wem(value = 4)]
    InitialHorizontal,
    /// InitialVertical.
    #[wem(value = 5)]
    InitialVertical,
    /// Lifetime.
    #[wem(value = 6)]
    Lifetime,
    /// LifetimeRandom.
    #[wem(value = 7)]
    LifetimeRandom,
    /// SizeAnimation.
    #[wem(value = 8)]
    SizeAnimation,
    /// RotationAnimation.
    #[wem(value = 9)]
    RotationAnimation,
    /// ColorStart.
    #[wem(value = 10)]
    ColorStart,
    /// ColorMid.
    #[wem(value = 11)]
    ColorMid,
    /// ColorEnd.
    #[wem(value = 12)]
    ColorEnd,
    /// MaxLength.
    #[wem(value = 13)]
    MaxLength,
    /// Active.
    #[wem(value = 14)]
    Active,
    /// YawAmplitude.
    #[wem(value = 15)]
    YawAmplitude,
    /// YawFrequency.
    #[wem(value = 16)]
    YawFrequency,
    /// PitchAmplitude.
    #[wem(value = 17)]
    PitchAmplitude,
    /// PitchFrequency.
    #[wem(value = 18)]
    PitchFrequency,
    /// SpeedAmplitude.
    #[wem(value = 19)]
    SpeedAmplitude,
    /// SpeedFrequency.
    #[wem(value = 20)]
    SpeedFrequency,
    /// SizeAmplitude.
    #[wem(value = 21)]
    SizeAmplitude,
    /// SizeFrequency.
    #[wem(value = 22)]
    SizeFrequency,
    /// AlphaAmplitude.
    #[wem(value = 23)]
    AlphaAmplitude,
    /// AlphaFrequency.
    #[wem(value = 24)]
    AlphaFrequency,
    /// ParticleVelocity.
    #[wem(value = 25)]
    ParticleVelocity,
    /// Overlay.
    #[wem(value = 26)]
    Overlay,
    /// SplineVelocity.
    #[wem(value = 27)]
    SplineVelocity,
    /// SplineVelocityBaseFactor.
    #[wem(value = 28)]
    SplineVelocityBaseFactor,
    /// SplineVelocityEndFactor.
    #[wem(value = 29)]
    SplineVelocityEndFactor,
    /// SplineYawAmplitude.
    #[wem(value = 30)]
    SplineYawAmplitude,
    /// SplineYawFrequency.
    #[wem(value = 31)]
    SplineYawFrequency,
    /// SplinePitchAmplitude.
    #[wem(value = 32)]
    SplinePitchAmplitude,
    /// SplinePitchFrequency.
    #[wem(value = 33)]
    SplinePitchFrequency,
    /// SplineVelocityAmplitude.
    #[wem(value = 34)]
    SplineVelocityAmplitude,
    /// SplineVelocityFrequency.
    #[wem(value = 35)]
    SplineVelocityFrequency,
    /// SplineYaw.
    #[wem(value = 36)]
    SplineYaw,
    /// SplinePitch.
    #[wem(value = 37)]
    SplinePitch,
    /// Count.
    #[wem(value = 38)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Wc3ParticleFilter choices; unrecognized codes remain representable.
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
pub enum Wc3ParticleFilter {
    #[default]
    /// Blend.
    #[wem(value = 0)]
    Blend,
    /// Additive.
    #[wem(value = 1)]
    Additive,
    /// Modulate.
    #[wem(value = 2)]
    Modulate,
    /// Modulate2x.
    #[wem(value = 3)]
    Modulate2x,
    /// AlphaKey.
    #[wem(value = 4)]
    AlphaKey,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Wc3ParticleHeadOrTail choices; unrecognized codes remain representable.
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
pub enum Wc3ParticleHeadOrTail {
    #[default]
    /// Head.
    #[wem(value = 0)]
    Head,
    /// Tail.
    #[wem(value = 1)]
    Tail,
    /// Both.
    #[wem(value = 2)]
    Both,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Sc2EmitterShape choices; unrecognized codes remain representable.
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
pub enum Sc2EmitterShape {
    #[default]
    /// Point.
    #[wem(value = 0)]
    Point,
    /// Plane.
    #[wem(value = 1)]
    Plane,
    /// Sphere.
    #[wem(value = 2)]
    Sphere,
    /// Box.
    #[wem(value = 3)]
    Box,
    /// Cylinder.
    #[wem(value = 4)]
    Cylinder,
    /// Disc.
    #[wem(value = 5)]
    Disc,
    /// Spline.
    #[wem(value = 6)]
    Spline,
    /// Mesh.
    #[wem(value = 7)]
    Mesh,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Sc2ParticleInstance choices; unrecognized codes remain representable.
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
pub enum Sc2ParticleInstance {
    #[default]
    /// Billboard.
    #[wem(value = 0)]
    Billboard,
    /// Tail.
    #[wem(value = 1)]
    Tail,
    /// FaceTravelDir.
    #[wem(value = 2)]
    FaceTravelDir,
    /// FaceWorldDir.
    #[wem(value = 3)]
    FaceWorldDir,
    /// SingleAxis.
    #[wem(value = 4)]
    SingleAxis,
    /// TerrainOriented.
    #[wem(value = 5)]
    TerrainOriented,
    /// TerrainDirOriented.
    #[wem(value = 6)]
    TerrainDirOriented,
    /// EmitterOriented.
    #[wem(value = 7)]
    EmitterOriented,
    /// PhysicsOriented.
    #[wem(value = 8)]
    PhysicsOriented,
    /// Pinned.
    #[wem(value = 9)]
    Pinned,
    /// Trail.
    #[wem(value = 10)]
    Trail,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Sc2Smoothing choices; unrecognized codes remain representable.
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
pub enum Sc2Smoothing {
    #[default]
    /// Linear.
    #[wem(value = 0)]
    Linear,
    /// LinearSmooth.
    #[wem(value = 1)]
    LinearSmooth,
    /// Bezier.
    #[wem(value = 2)]
    Bezier,
    /// LinearWithHold.
    #[wem(value = 3)]
    LinearWithHold,
    /// BezierWithHold.
    #[wem(value = 4)]
    BezierWithHold,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named Sc2RibbonType choices; unrecognized codes remain representable.
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
pub enum Sc2RibbonType {
    #[default]
    /// Billboard.
    #[wem(value = 0)]
    Billboard,
    /// Planar.
    #[wem(value = 1)]
    Planar,
    /// Cylinder.
    #[wem(value = 2)]
    Cylinder,
    /// Star.
    #[wem(value = 3)]
    Star,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named NodeReference choices; unrecognized codes remain representable.
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
pub enum NodeReference {
    #[default]
    /// Influence.
    #[wem(value = 0)]
    Influence,
    /// RigidNode.
    #[wem(value = 1)]
    RigidNode,
    /// VisibilityGate.
    #[wem(value = 2)]
    VisibilityGate,
    /// Channel.
    #[wem(value = 3)]
    Channel,
    /// StageDriven.
    #[wem(value = 4)]
    StageDriven,
    /// StageTarget.
    #[wem(value = 5)]
    StageTarget,
    /// StageSource.
    #[wem(value = 6)]
    StageSource,
    /// StageUp.
    #[wem(value = 7)]
    StageUp,
    /// PhysicsBody.
    #[wem(value = 8)]
    PhysicsBody,
    /// ClothCollider.
    #[wem(value = 9)]
    ClothCollider,
    /// ClothBone.
    #[wem(value = 10)]
    ClothBone,
    /// FractureHelper.
    #[wem(value = 11)]
    FractureHelper,
    /// FractureField.
    #[wem(value = 12)]
    FractureField,
    /// FractureWholeGate.
    #[wem(value = 13)]
    FractureWholeGate,
    /// FracturePiecesGate.
    #[wem(value = 14)]
    FracturePiecesGate,
    /// FractureBone.
    #[wem(value = 15)]
    FractureBone,
    /// FractureSkinNode.
    #[wem(value = 16)]
    FractureSkinNode,
    /// NodeLink.
    #[wem(value = 17)]
    NodeLink,
    /// ClipEvent.
    #[wem(value = 18)]
    ClipEvent,
    /// ClipWorldCentre.
    #[wem(value = 19)]
    ClipWorldCentre,
    /// SoftCarrier.
    #[wem(value = 20)]
    SoftCarrier,
    /// SoftNode.
    #[wem(value = 21)]
    SoftNode,
    /// SoftChain.
    #[wem(value = 22)]
    SoftChain,
    /// SoftAxisBone.
    #[wem(value = 23)]
    SoftAxisBone,
    /// SoftStrand.
    #[wem(value = 24)]
    SoftStrand,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}
