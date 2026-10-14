//! Logical shading choices for authored WEM surfaces.

/// BlendMode behavior, retaining unrecognized authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlendMode {
    #[default]
    /// Opaque.
    Opaque,
    /// AlphaKey.
    AlphaKey,
    /// AlphaBlend.
    AlphaBlend,
    /// Additive.
    Additive,
    /// AdditiveAlpha.
    AdditiveAlpha,
    /// Modulate.
    Modulate,
    /// Modulate2x.
    Modulate2x,
    /// BlendAdd.
    BlendAdd,
    /// Transparent.
    Transparent,
    /// PremultipliedAlpha.
    PremultipliedAlpha,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u32),
}

/// CullMode behavior, retaining unrecognized authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CullMode {
    #[default]
    /// Back.
    Back,
    /// Front.
    Front,
    /// None.
    None,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}

/// UVMappingMode behavior, retaining unrecognized authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UVMappingMode {
    #[default]
    /// ExplicitUV.
    ExplicitUV,
    /// EnvSphere.
    EnvSphere,
    /// EnvCube.
    EnvCube,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}

/// LegacySlot behavior, retaining unrecognized authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LegacySlot {
    #[default]
    /// Diffuse.
    Diffuse,
    /// Normal.
    Normal,
    /// Specular.
    Specular,
    /// Gloss.
    Gloss,
    /// Emissive.
    Emissive,
    /// Environment.
    Environment,
    /// AmbientOcclusion.
    AmbientOcclusion,
    /// Height.
    Height,
    /// Lightmap.
    Lightmap,
    /// Detail.
    Detail,
    /// Count.
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}

/// SurfaceChannel behavior, retaining unrecognized authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SurfaceChannel {
    #[default]
    /// Color.
    Color,
    /// Emissive.
    Emissive,
    /// Specular.
    Specular,
    /// Normal.
    Normal,
    /// AmbientOcclusion.
    AmbientOcclusion,
    /// Environment.
    Environment,
    /// Coverage.
    Coverage,
    /// Gloss.
    Gloss,
    /// Count.
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}

/// CompositeOp behavior, retaining unrecognized authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompositeOp {
    #[default]
    /// Set.
    Set,
    /// AlphaKey.
    AlphaKey,
    /// AlphaBlend.
    AlphaBlend,
    /// Add.
    Add,
    /// AddAlpha.
    AddAlpha,
    /// Modulate.
    Modulate,
    /// Modulate2x.
    Modulate2x,
    /// Count.
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}

/// CombinerOp behavior, retaining unrecognized authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CombinerOp {
    #[default]
    /// Opaque.
    Opaque,
    /// Mod.
    Mod,
    /// Mod2x.
    Mod2x,
    /// Add.
    Add,
    /// Decal.
    Decal,
    /// Fade.
    Fade,
    /// Pass.
    Pass,
    /// AddAlpha.
    AddAlpha,
    /// MaskedMod.
    MaskedMod,
    /// MaskedMod2x.
    MaskedMod2x,
    /// Count.
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}

/// PbrSlot behavior, retaining unrecognized authoring choices.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PbrSlot {
    #[default]
    /// BaseColor.
    BaseColor,
    /// Normal.
    Normal,
    /// Orm.
    Orm,
    /// Metallic.
    Metallic,
    /// Roughness.
    Roughness,
    /// AmbientOcclusion.
    AmbientOcclusion,
    /// Emissive.
    Emissive,
    /// Environment.
    Environment,
    /// TeamColorMask.
    TeamColorMask,
    /// Count.
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    Unknown(u8),
}
