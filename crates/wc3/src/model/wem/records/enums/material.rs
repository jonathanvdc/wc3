//! Named material choices and flags retain unrecognized wire values.
use crate::model::wem;
use bitfield::bitfield;

/// Named BlendMode choices; unrecognized codes remain representable.
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
pub enum BlendMode {
    #[default]
    /// Opaque.
    #[wem(value = 0)]
    Opaque,
    /// AlphaKey.
    #[wem(value = 1)]
    AlphaKey,
    /// AlphaBlend.
    #[wem(value = 2)]
    AlphaBlend,
    /// Additive.
    #[wem(value = 3)]
    Additive,
    /// AdditiveAlpha.
    #[wem(value = 4)]
    AdditiveAlpha,
    /// Modulate.
    #[wem(value = 5)]
    Modulate,
    /// Modulate2x.
    #[wem(value = 6)]
    Modulate2x,
    /// BlendAdd.
    #[wem(value = 7)]
    BlendAdd,
    /// Transparent.
    #[wem(value = 8)]
    Transparent,
    /// PremultipliedAlpha.
    #[wem(value = 9)]
    PremultipliedAlpha,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named MaterialKind choices; unrecognized codes remain representable.
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
pub enum MaterialKind {
    #[default]
    /// Composite.
    #[wem(value = 0)]
    Composite,
    /// Combiners.
    #[wem(value = 1)]
    Combiners,
    /// LegacyDeferred.
    #[wem(value = 2)]
    LegacyDeferred,
    /// PBRDeferred.
    #[wem(value = 3)]
    PBRDeferred,
    /// Count.
    #[wem(value = 4)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

bitfield! {
    /// MaterialFlags bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct MaterialFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the Unlit bit.
    pub unlit, set_unlit: 0;
    /// Returns or changes the Unfogged bit.
    pub unfogged, set_unfogged: 1;
    /// Returns or changes the NoShadowCast bit.
    pub no_shadow_cast, set_no_shadow_cast: 2;
    /// Returns or changes the NoShadowReceive bit.
    pub no_shadow_receive, set_no_shadow_receive: 3;
    /// Returns or changes the SortNearZ bit.
    pub sort_near_z, set_sort_near_z: 4;
    /// Returns or changes the SortFarZ bit.
    pub sort_far_z, set_sort_far_z: 5;
    /// Returns or changes the Invisible bit.
    pub invisible, set_invisible: 6;
}

impl MaterialFlags {
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
    /// Bit mask for Unlit.
    pub const UNLIT: Self = Self(1);
    /// Bit mask for Unfogged.
    pub const UNFOGGED: Self = Self(2);
    /// Bit mask for NoShadowCast.
    pub const NO_SHADOW_CAST: Self = Self(4);
    /// Bit mask for NoShadowReceive.
    pub const NO_SHADOW_RECEIVE: Self = Self(8);
    /// Bit mask for SortNearZ.
    pub const SORT_NEAR_Z: Self = Self(16);
    /// Bit mask for SortFarZ.
    pub const SORT_FAR_Z: Self = Self(32);
    /// Bit mask for Invisible.
    pub const INVISIBLE: Self = Self(64);
}
impl From<u32> for MaterialFlags {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<MaterialFlags> for u32 {
    fn from(value: MaterialFlags) -> Self {
        value.bits()
    }
}

/// Named UVMappingMode choices; unrecognized codes remain representable.
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
pub enum UVMappingMode {
    #[default]
    /// ExplicitUV.
    #[wem(value = 0)]
    ExplicitUV,
    /// EnvSphere.
    #[wem(value = 1)]
    EnvSphere,
    /// EnvCube.
    #[wem(value = 2)]
    EnvCube,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named LegacySlot choices; unrecognized codes remain representable.
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
pub enum LegacySlot {
    #[default]
    /// Diffuse.
    #[wem(value = 0)]
    Diffuse,
    /// Normal.
    #[wem(value = 1)]
    Normal,
    /// Specular.
    #[wem(value = 2)]
    Specular,
    /// Gloss.
    #[wem(value = 3)]
    Gloss,
    /// Emissive.
    #[wem(value = 4)]
    Emissive,
    /// Environment.
    #[wem(value = 5)]
    Environment,
    /// AmbientOcclusion.
    #[wem(value = 6)]
    AmbientOcclusion,
    /// Height.
    #[wem(value = 7)]
    Height,
    /// Lightmap.
    #[wem(value = 8)]
    Lightmap,
    /// Detail.
    #[wem(value = 9)]
    Detail,
    /// Count.
    #[wem(value = 10)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ColorSpace choices; unrecognized codes remain representable.
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
pub enum ColorSpace {
    #[default]
    /// Auto.
    #[wem(value = 0)]
    Auto,
    /// Linear.
    #[wem(value = 1)]
    Linear,
    /// Srgb.
    #[wem(value = 2)]
    Srgb,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named WrapMode choices; unrecognized codes remain representable.
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
pub enum WrapMode {
    #[default]
    /// Repeat.
    #[wem(value = 0)]
    Repeat,
    /// Clamp.
    #[wem(value = 1)]
    Clamp,
    /// Mirror.
    #[wem(value = 2)]
    Mirror,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named TextureKeyKind choices; unrecognized codes remain representable.
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
pub enum TextureKeyKind {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Path.
    #[wem(value = 1)]
    Path,
    /// FileDataId.
    #[wem(value = 2)]
    FileDataId,
    /// SnoId.
    #[wem(value = 3)]
    SnoId,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named CullMode choices; unrecognized codes remain representable.
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
pub enum CullMode {
    #[default]
    /// Back.
    #[wem(value = 0)]
    Back,
    /// Front.
    #[wem(value = 1)]
    Front,
    /// None.
    #[wem(value = 2)]
    None,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named ImageChannel choices; unrecognized codes remain representable.
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
pub enum ImageChannel {
    #[default]
    /// All.
    #[wem(value = 0)]
    All,
    /// Red.
    #[wem(value = 1)]
    Red,
    /// Green.
    #[wem(value = 2)]
    Green,
    /// Blue.
    #[wem(value = 3)]
    Blue,
    /// Alpha.
    #[wem(value = 4)]
    Alpha,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SurfaceRole choices; unrecognized codes remain representable.
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
pub enum SurfaceRole {
    #[default]
    /// BaseColor.
    #[wem(value = 0)]
    BaseColor,
    /// Normal.
    #[wem(value = 1)]
    Normal,
    /// Orm.
    #[wem(value = 2)]
    Orm,
    /// Occlusion.
    #[wem(value = 3)]
    Occlusion,
    /// Emissive.
    #[wem(value = 4)]
    Emissive,
    /// Specular.
    #[wem(value = 5)]
    Specular,
    /// Gloss.
    #[wem(value = 6)]
    Gloss,
    /// Height.
    #[wem(value = 7)]
    Height,
    /// Metallic.
    #[wem(value = 8)]
    Metallic,
    /// Roughness.
    #[wem(value = 9)]
    Roughness,
    /// Dropped.
    #[wem(value = 10)]
    Dropped,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named FeatureKind choices; unrecognized codes remain representable.
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
pub enum FeatureKind {
    #[default]
    /// Fresnel.
    #[wem(value = 0)]
    Fresnel,
    /// UvAnimation.
    #[wem(value = 1)]
    UvAnimation,
    /// LayerShading.
    #[wem(value = 2)]
    LayerShading,
    /// Count.
    #[wem(value = 3)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named NativeSync choices; unrecognized codes remain representable.
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
pub enum NativeSync {
    #[default]
    /// Absent.
    #[wem(value = 0)]
    Absent,
    /// InSync.
    #[wem(value = 1)]
    InSync,
    /// NativeAuthoritative.
    #[wem(value = 2)]
    NativeAuthoritative,
    /// CommonEdited.
    #[wem(value = 3)]
    CommonEdited,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named SurfaceChannel choices; unrecognized codes remain representable.
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
pub enum SurfaceChannel {
    #[default]
    /// Color.
    #[wem(value = 0)]
    Color,
    /// Emissive.
    #[wem(value = 1)]
    Emissive,
    /// Specular.
    #[wem(value = 2)]
    Specular,
    /// Normal.
    #[wem(value = 3)]
    Normal,
    /// AmbientOcclusion.
    #[wem(value = 4)]
    AmbientOcclusion,
    /// Environment.
    #[wem(value = 5)]
    Environment,
    /// Coverage.
    #[wem(value = 6)]
    Coverage,
    /// Gloss.
    #[wem(value = 7)]
    Gloss,
    /// Count.
    #[wem(value = 8)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named CompositeOp choices; unrecognized codes remain representable.
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
pub enum CompositeOp {
    #[default]
    /// Set.
    #[wem(value = 0)]
    Set,
    /// AlphaKey.
    #[wem(value = 1)]
    AlphaKey,
    /// AlphaBlend.
    #[wem(value = 2)]
    AlphaBlend,
    /// Add.
    #[wem(value = 3)]
    Add,
    /// AddAlpha.
    #[wem(value = 4)]
    AddAlpha,
    /// Modulate.
    #[wem(value = 5)]
    Modulate,
    /// Modulate2x.
    #[wem(value = 6)]
    Modulate2x,
    /// Count.
    #[wem(value = 7)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named CombinerOp choices; unrecognized codes remain representable.
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
pub enum CombinerOp {
    #[default]
    /// Opaque.
    #[wem(value = 0)]
    Opaque,
    /// Mod.
    #[wem(value = 1)]
    Mod,
    /// Mod2x.
    #[wem(value = 2)]
    Mod2x,
    /// Add.
    #[wem(value = 3)]
    Add,
    /// Decal.
    #[wem(value = 4)]
    Decal,
    /// Fade.
    #[wem(value = 5)]
    Fade,
    /// Pass.
    #[wem(value = 6)]
    Pass,
    /// AddAlpha.
    #[wem(value = 7)]
    AddAlpha,
    /// MaskedMod.
    #[wem(value = 8)]
    MaskedMod,
    /// MaskedMod2x.
    #[wem(value = 9)]
    MaskedMod2x,
    /// Count.
    #[wem(value = 10)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

/// Named PbrSlot choices; unrecognized codes remain representable.
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
pub enum PbrSlot {
    #[default]
    /// BaseColor.
    #[wem(value = 0)]
    BaseColor,
    /// Normal.
    #[wem(value = 1)]
    Normal,
    /// Orm.
    #[wem(value = 2)]
    Orm,
    /// Metallic.
    #[wem(value = 3)]
    Metallic,
    /// Roughness.
    #[wem(value = 4)]
    Roughness,
    /// AmbientOcclusion.
    #[wem(value = 5)]
    AmbientOcclusion,
    /// Emissive.
    #[wem(value = 6)]
    Emissive,
    /// Environment.
    #[wem(value = 7)]
    Environment,
    /// TeamColorMask.
    #[wem(value = 8)]
    TeamColorMask,
    /// Count.
    #[wem(value = 9)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}
