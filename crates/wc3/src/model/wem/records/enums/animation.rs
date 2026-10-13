//! Named animation choices and flags retain unrecognized wire values.
use crate::model::wem;
use bitfield::bitfield;

/// Named Channel choices; unrecognized codes remain representable.
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
pub enum Channel {
    #[default]
    /// Translation.
    #[wem(value = 0)]
    Translation,
    /// Rotation.
    #[wem(value = 1)]
    Rotation,
    /// Scale.
    #[wem(value = 2)]
    Scale,
    /// Visibility.
    #[wem(value = 3)]
    Visibility,
    /// Color.
    #[wem(value = 4)]
    Color,
    /// Alpha.
    #[wem(value = 5)]
    Alpha,
    /// Intensity.
    #[wem(value = 6)]
    Intensity,
    /// AttenuationStart.
    #[wem(value = 7)]
    AttenuationStart,
    /// AttenuationEnd.
    #[wem(value = 8)]
    AttenuationEnd,
    /// UvTranslate.
    #[wem(value = 9)]
    UvTranslate,
    /// UvRotate.
    #[wem(value = 10)]
    UvRotate,
    /// UvScale.
    #[wem(value = 11)]
    UvScale,
    /// Weight.
    #[wem(value = 12)]
    Weight,
    /// TextureIndex.
    #[wem(value = 13)]
    TextureIndex,
    /// Emissive.
    #[wem(value = 14)]
    Emissive,
    /// EmitterProperty.
    #[wem(value = 15)]
    EmitterProperty,
    /// ShadowCastingStart.
    #[wem(value = 16)]
    ShadowCastingStart,
    /// ShadowCastingEnd.
    #[wem(value = 17)]
    ShadowCastingEnd,
    /// QuadraticFalloff.
    #[wem(value = 18)]
    QuadraticFalloff,
    /// LinearFalloff.
    #[wem(value = 19)]
    LinearFalloff,
    /// Damping.
    #[wem(value = 20)]
    Damping,
    /// StageWeight.
    #[wem(value = 21)]
    StageWeight,
    /// StageSourceWeight.
    #[wem(value = 22)]
    StageSourceWeight,
    /// StageSourceEnabled.
    #[wem(value = 23)]
    StageSourceEnabled,
    /// FocusDistance.
    #[wem(value = 24)]
    FocusDistance,
    /// FocalLength.
    #[wem(value = 25)]
    FocalLength,
    /// FStop.
    #[wem(value = 26)]
    FStop,
    /// Target.
    #[wem(value = 27)]
    Target,
    /// Roll.
    #[wem(value = 28)]
    Roll,
    /// PhysicsDynamic.
    #[wem(value = 29)]
    PhysicsDynamic,
    /// ClothActive.
    #[wem(value = 30)]
    ClothActive,
    /// PhysicsRagdoll.
    #[wem(value = 31)]
    PhysicsRagdoll,
    /// PhysicsBlend.
    #[wem(value = 32)]
    PhysicsBlend,
    /// ClothDriverTranslation.
    #[wem(value = 33)]
    ClothDriverTranslation,
    /// ClothDriverRotation.
    #[wem(value = 34)]
    ClothDriverRotation,
    /// NormalTextureIndex.
    #[wem(value = 35)]
    NormalTextureIndex,
    /// OrmTextureIndex.
    #[wem(value = 36)]
    OrmTextureIndex,
    /// EmissiveTextureIndex.
    #[wem(value = 37)]
    EmissiveTextureIndex,
    /// TeamColorTextureIndex.
    #[wem(value = 38)]
    TeamColorTextureIndex,
    /// EnvironmentTextureIndex.
    #[wem(value = 39)]
    EnvironmentTextureIndex,
    /// SoftAmount.
    #[wem(value = 40)]
    SoftAmount,
    /// SoftLumpTranslation.
    #[wem(value = 41)]
    SoftLumpTranslation,
    /// SoftLumpRotation.
    #[wem(value = 42)]
    SoftLumpRotation,
    /// SoftLumpScale.
    #[wem(value = 43)]
    SoftLumpScale,
    /// Count.
    #[wem(value = 44)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named TrackTargetKind choices; unrecognized codes remain representable.
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
pub enum TrackTargetKind {
    #[default]
    /// Node.
    #[wem(value = 0)]
    Node,
    /// MaterialLayer.
    #[wem(value = 1)]
    MaterialLayer,
    /// MaterialFeature.
    #[wem(value = 2)]
    MaterialFeature,
    /// Section.
    #[wem(value = 3)]
    Section,
    /// Physics.
    #[wem(value = 4)]
    Physics,
    /// Count.
    #[wem(value = 5)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named Interpolation choices; unrecognized codes remain representable.
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
pub enum Interpolation {
    #[default]
    /// Step.
    #[wem(value = 0)]
    Step,
    /// Linear.
    #[wem(value = 1)]
    Linear,
    /// Hermite.
    #[wem(value = 2)]
    Hermite,
    /// Bezier.
    #[wem(value = 3)]
    Bezier,
    /// Slerp.
    #[wem(value = 4)]
    Slerp,
    /// Count.
    #[wem(value = 5)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

bitfield! {
    /// ClipFlags bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct ClipFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the AutoPlay bit.
    pub auto_play, set_auto_play: 0;
    /// Returns or changes the Persistent bit.
    pub persistent, set_persistent: 1;
    /// Returns or changes the WorldClocked bit.
    pub world_clocked, set_world_clocked: 2;
}

impl ClipFlags {
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
    /// Bit mask for AutoPlay.
    pub const AUTO_PLAY: Self = Self(1);
    /// Bit mask for Persistent.
    pub const PERSISTENT: Self = Self(2);
    /// Bit mask for WorldClocked.
    pub const WORLD_CLOCKED: Self = Self(4);
}
impl From<u32> for ClipFlags {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<ClipFlags> for u32 {
    fn from(value: ClipFlags) -> Self {
        value.bits()
    }
}

/// Named MirrorAxis choices; unrecognized codes remain representable.
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
pub enum MirrorAxis {
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
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named MirrorSource choices; unrecognized codes remain representable.
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
pub enum MirrorSource {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Override.
    #[wem(value = 1)]
    Override,
    /// Name.
    #[wem(value = 2)]
    Name,
    /// Position.
    #[wem(value = 3)]
    Position,
    /// Self.
    #[wem(value = 4)]
    SelfTarget,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named EventCategory choices; unrecognized codes remain representable.
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
pub enum EventCategory {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Sound.
    #[wem(value = 1)]
    Sound,
    /// Splat.
    #[wem(value = 2)]
    Splat,
    /// Footprint.
    #[wem(value = 3)]
    Footprint,
    /// UberSplat.
    #[wem(value = 4)]
    UberSplat,
    /// SpawnModel.
    #[wem(value = 5)]
    SpawnModel,
    /// MorphShow.
    #[wem(value = 6)]
    MorphShow,
    /// MorphHide.
    #[wem(value = 7)]
    MorphHide,
    /// Count.
    #[wem(value = 8)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}
