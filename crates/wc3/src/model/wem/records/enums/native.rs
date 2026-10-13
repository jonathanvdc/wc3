//! Named native choices and flags retain unrecognized wire values.
use crate::model::wem;
use bitfield::bitfield;

/// Named M3MaterialKind choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
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
pub enum M3MaterialKind {
    /// Standard.
    #[wem(value = 1)]
    Standard,
    /// Displacement.
    #[wem(value = 2)]
    Displacement,
    /// Composite.
    #[wem(value = 3)]
    Composite,
    /// Terrain.
    #[wem(value = 4)]
    Terrain,
    /// Volume.
    #[wem(value = 5)]
    Volume,
    /// VolumeNoise.
    #[wem(value = 6)]
    VolumeNoise,
    /// Creep.
    #[wem(value = 7)]
    Creep,
    /// Hair.
    #[wem(value = 8)]
    Hair,
    /// SplatTerrainBake.
    #[wem(value = 9)]
    SplatTerrainBake,
    /// Reflection.
    #[wem(value = 10)]
    Reflection,
    /// LensFlare.
    #[wem(value = 11)]
    LensFlare,
    /// DataDriven.
    #[wem(value = 12)]
    DataDriven,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}
impl Default for M3MaterialKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// M3MaterialAdditionalFlag bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M3MaterialAdditionalFlag(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the DepthBlendFalloff bit.
    pub depth_blend_falloff, set_depth_blend_falloff: 0;
    /// Returns or changes the VertexColor bit.
    pub vertex_color, set_vertex_color: 2;
    /// Returns or changes the VertexAlpha bit.
    pub vertex_alpha, set_vertex_alpha: 3;
}

impl M3MaterialAdditionalFlag {
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
    /// Bit mask for DepthBlendFalloff.
    pub const DEPTH_BLEND_FALLOFF: Self = Self(1);
    /// Bit mask for VertexColor.
    pub const VERTEX_COLOR: Self = Self(4);
    /// Bit mask for VertexAlpha.
    pub const VERTEX_ALPHA: Self = Self(8);
}
impl From<u32> for M3MaterialAdditionalFlag {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<M3MaterialAdditionalFlag> for u32 {
    fn from(value: M3MaterialAdditionalFlag) -> Self {
        value.bits()
    }
}

bitfield! {
    /// M3MaterialFlag bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M3MaterialFlag(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the VertexColor bit.
    pub vertex_color, set_vertex_color: 0;
    /// Returns or changes the VertexAlpha bit.
    pub vertex_alpha, set_vertex_alpha: 1;
    /// Disables fog on this material.
    pub unfogged, set_unfogged: 2;
    /// Returns or changes the TwoSided bit.
    pub two_sided, set_two_sided: 3;
    /// Returns or changes the Unshaded bit.
    pub unshaded, set_unshaded: 4;
    /// Returns or changes the NoShadowsCast bit.
    pub no_shadows_cast, set_no_shadows_cast: 5;
    /// Returns or changes the NoHitTest bit.
    pub no_hit_test, set_no_hit_test: 6;
    /// Returns or changes the NoShadowsReceive bit.
    pub no_shadows_receive, set_no_shadows_receive: 7;
    /// Returns or changes the DepthPrepass bit.
    pub depth_prepass, set_depth_prepass: 8;
    /// Returns or changes the TerrainHDR bit.
    pub terrain_hdr, set_terrain_hdr: 9;
    /// Returns or changes the SimulateRoughness bit.
    pub simulate_roughness, set_simulate_roughness: 11;
    /// Returns or changes the PixelForwardLighting bit.
    pub pixel_forward_lighting, set_pixel_forward_lighting: 12;
    /// Enables depth-dependent fog.
    pub depth_fog, set_depth_fog: 13;
    /// Returns or changes the TransparentShadows bit.
    pub transparent_shadows, set_transparent_shadows: 14;
    /// Returns or changes the DecalLighting bit.
    pub decal_lighting, set_decal_lighting: 15;
    /// Returns or changes the TransparentDepthEffects bit.
    pub transparent_depth_effects, set_transparent_depth_effects: 16;
    /// Returns or changes the TransparentLocalLights bit.
    pub transparent_local_lights, set_transparent_local_lights: 17;
    /// Returns or changes the DisableSoft bit.
    pub disable_soft, set_disable_soft: 18;
    /// Returns or changes the DoubleLambert bit.
    pub double_lambert, set_double_lambert: 19;
    /// Returns or changes the HairLayerSorting bit.
    pub hair_layer_sorting, set_hair_layer_sorting: 20;
    /// Returns or changes the AcceptSplats bit.
    pub accept_splats, set_accept_splats: 21;
    /// Returns or changes the DecalLowRequired bit.
    pub decal_low_required, set_decal_low_required: 22;
    /// Returns or changes the EmisLowRequired bit.
    pub emis_low_required, set_emis_low_required: 23;
    /// Returns or changes the SpecLowRequired bit.
    pub spec_low_required, set_spec_low_required: 24;
    /// Returns or changes the AcceptSplatsOnly bit.
    pub accept_splats_only, set_accept_splats_only: 25;
    /// Returns or changes the BackgroundObject bit.
    pub background_object, set_background_object: 26;
    /// Returns or changes the DepthPrepassLowRequired bit.
    pub depth_prepass_low_required, set_depth_prepass_low_required: 28;
    /// Returns or changes the NoHighlighting bit.
    pub no_highlighting, set_no_highlighting: 29;
    /// Returns or changes the ClampOutput bit.
    pub clamp_output, set_clamp_output: 30;
    /// Returns or changes the GeometryVisible bit.
    pub geometry_visible, set_geometry_visible: 31;
}

impl M3MaterialFlag {
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
    /// Bit mask for VertexColor.
    pub const VERTEX_COLOR: Self = Self(1);
    /// Bit mask for VertexAlpha.
    pub const VERTEX_ALPHA: Self = Self(2);
    /// Bit mask disabling fog.
    pub const UNFOGGED: Self = Self(4);
    /// Bit mask for TwoSided.
    pub const TWO_SIDED: Self = Self(8);
    /// Bit mask for Unshaded.
    pub const UNSHADED: Self = Self(16);
    /// Bit mask for NoShadowsCast.
    pub const NO_SHADOWS_CAST: Self = Self(32);
    /// Bit mask for NoHitTest.
    pub const NO_HIT_TEST: Self = Self(64);
    /// Bit mask for NoShadowsReceive.
    pub const NO_SHADOWS_RECEIVE: Self = Self(128);
    /// Bit mask for DepthPrepass.
    pub const DEPTH_PREPASS: Self = Self(256);
    /// Bit mask for TerrainHDR.
    pub const TERRAIN_HDR: Self = Self(512);
    /// Bit mask for SimulateRoughness.
    pub const SIMULATE_ROUGHNESS: Self = Self(2048);
    /// Bit mask for PixelForwardLighting.
    pub const PIXEL_FORWARD_LIGHTING: Self = Self(4096);
    /// Bit mask enabling depth fog.
    pub const DEPTH_FOG: Self = Self(8192);
    /// Bit mask for TransparentShadows.
    pub const TRANSPARENT_SHADOWS: Self = Self(16384);
    /// Bit mask for DecalLighting.
    pub const DECAL_LIGHTING: Self = Self(32768);
    /// Bit mask for TransparentDepthEffects.
    pub const TRANSPARENT_DEPTH_EFFECTS: Self = Self(65536);
    /// Bit mask for TransparentLocalLights.
    pub const TRANSPARENT_LOCAL_LIGHTS: Self = Self(131072);
    /// Bit mask for DisableSoft.
    pub const DISABLE_SOFT: Self = Self(262144);
    /// Bit mask for DoubleLambert.
    pub const DOUBLE_LAMBERT: Self = Self(524288);
    /// Bit mask for HairLayerSorting.
    pub const HAIR_LAYER_SORTING: Self = Self(1048576);
    /// Bit mask for AcceptSplats.
    pub const ACCEPT_SPLATS: Self = Self(2097152);
    /// Bit mask for DecalLowRequired.
    pub const DECAL_LOW_REQUIRED: Self = Self(4194304);
    /// Bit mask for EmisLowRequired.
    pub const EMIS_LOW_REQUIRED: Self = Self(8388608);
    /// Bit mask for SpecLowRequired.
    pub const SPEC_LOW_REQUIRED: Self = Self(16777216);
    /// Bit mask for AcceptSplatsOnly.
    pub const ACCEPT_SPLATS_ONLY: Self = Self(33554432);
    /// Bit mask for BackgroundObject.
    pub const BACKGROUND_OBJECT: Self = Self(67108864);
    /// Bit mask for DepthPrepassLowRequired.
    pub const DEPTH_PREPASS_LOW_REQUIRED: Self = Self(268435456);
    /// Bit mask for NoHighlighting.
    pub const NO_HIGHLIGHTING: Self = Self(536870912);
    /// Bit mask for ClampOutput.
    pub const CLAMP_OUTPUT: Self = Self(1073741824);
    /// Bit mask for GeometryVisible.
    pub const GEOMETRY_VISIBLE: Self = Self(2147483648);
}
impl From<u32> for M3MaterialFlag {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<M3MaterialFlag> for u32 {
    fn from(value: M3MaterialFlag) -> Self {
        value.bits()
    }
}

/// Named M3BlendMode choices; unrecognized codes remain representable.
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
pub enum M3BlendMode {
    #[default]
    /// Opaque.
    #[wem(value = 0)]
    Opaque,
    /// AlphaBlend.
    #[wem(value = 1)]
    AlphaBlend,
    /// Add.
    #[wem(value = 2)]
    Add,
    /// AlphaAdd.
    #[wem(value = 3)]
    AlphaAdd,
    /// Mod.
    #[wem(value = 4)]
    Mod,
    /// Mod2x.
    #[wem(value = 5)]
    Mod2x,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

bitfield! {
    /// M3TextureLayerFlag bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M3TextureLayerFlag(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the UVWrapX bit.
    pub uv_wrap_x, set_uv_wrap_x: 2;
    /// Returns or changes the UVWrapY bit.
    pub uv_wrap_y, set_uv_wrap_y: 3;
    /// Returns or changes the ColorInvert bit.
    pub color_invert, set_color_invert: 4;
    /// Returns or changes the ColorClamp bit.
    pub color_clamp, set_color_clamp: 5;
    /// Returns or changes the ColorAdd bit.
    pub color_add, set_color_add: 6;
    /// Returns or changes the ColorMultiply bit.
    pub color_multiply, set_color_multiply: 7;
    /// Returns or changes the ParticleUVFlipbook bit.
    pub particle_uv_flipbook, set_particle_uv_flipbook: 8;
    /// Returns or changes the Video bit.
    pub video, set_video: 9;
    /// Returns or changes the Color bit.
    pub color, set_color: 10;
    /// Returns or changes the ReplaceTextureSource bit.
    pub replace_texture_source, set_replace_texture_source: 11;
    /// Returns or changes the FresnelTransform bit.
    pub fresnel_transform, set_fresnel_transform: 14;
    /// Returns or changes the FresnelNormalize bit.
    pub fresnel_normalize, set_fresnel_normalize: 15;
}

impl M3TextureLayerFlag {
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
    /// Bit mask for UVWrapX.
    pub const UV_WRAP_X: Self = Self(4);
    /// Bit mask for UVWrapY.
    pub const UV_WRAP_Y: Self = Self(8);
    /// Bit mask for ColorInvert.
    pub const COLOR_INVERT: Self = Self(16);
    /// Bit mask for ColorClamp.
    pub const COLOR_CLAMP: Self = Self(32);
    /// Bit mask for ColorAdd.
    pub const COLOR_ADD: Self = Self(64);
    /// Bit mask for ColorMultiply.
    pub const COLOR_MULTIPLY: Self = Self(128);
    /// Bit mask for ParticleUVFlipbook.
    pub const PARTICLE_UV_FLIPBOOK: Self = Self(256);
    /// Bit mask for Video.
    pub const VIDEO: Self = Self(512);
    /// Bit mask for Color.
    pub const COLOR: Self = Self(1024);
    /// Bit mask for ReplaceTextureSource.
    pub const REPLACE_TEXTURE_SOURCE: Self = Self(2048);
    /// Bit mask for FresnelTransform.
    pub const FRESNEL_TRANSFORM: Self = Self(16384);
    /// Bit mask for FresnelNormalize.
    pub const FRESNEL_NORMALIZE: Self = Self(32768);
}
impl From<u32> for M3TextureLayerFlag {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<M3TextureLayerFlag> for u32 {
    fn from(value: M3TextureLayerFlag) -> Self {
        value.bits()
    }
}

/// Named M3UVMappingMode choices; unrecognized codes remain representable.
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
pub enum M3UVMappingMode {
    #[default]
    /// ExplicitUV0.
    #[wem(value = 0)]
    ExplicitUV0,
    /// ExplicitUV1.
    #[wem(value = 1)]
    ExplicitUV1,
    /// ReflectCubicEnvio.
    #[wem(value = 2)]
    ReflectCubicEnvio,
    /// ReflectSphericalEnvio.
    #[wem(value = 3)]
    ReflectSphericalEnvio,
    /// PlanarLocalZ.
    #[wem(value = 4)]
    PlanarLocalZ,
    /// PlanarWorldZ.
    #[wem(value = 5)]
    PlanarWorldZ,
    /// ParticleFlipbook.
    #[wem(value = 6)]
    ParticleFlipbook,
    /// CubicEnvio.
    #[wem(value = 7)]
    CubicEnvio,
    /// SphericalEnvio.
    #[wem(value = 8)]
    SphericalEnvio,
    /// ExplicitUV2.
    #[wem(value = 9)]
    ExplicitUV2,
    /// ExplicitUV3.
    #[wem(value = 10)]
    ExplicitUV3,
    /// PlanarLocalX.
    #[wem(value = 11)]
    PlanarLocalX,
    /// PlanarLocalY.
    #[wem(value = 12)]
    PlanarLocalY,
    /// PlanarWorldX.
    #[wem(value = 13)]
    PlanarWorldX,
    /// PlanarWorldY.
    #[wem(value = 14)]
    PlanarWorldY,
    /// ScreenSpace.
    #[wem(value = 15)]
    ScreenSpace,
    /// TriPlanarLocal.
    #[wem(value = 16)]
    TriPlanarLocal,
    /// TriPlanarWorld.
    #[wem(value = 17)]
    TriPlanarWorld,
    /// TriPlanarWorldLocalZ.
    #[wem(value = 18)]
    TriPlanarWorldLocalZ,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named M3ColorChannelSelect choices; unrecognized codes remain representable.
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
pub enum M3ColorChannelSelect {
    #[default]
    /// RGB.
    #[wem(value = 0)]
    RGB,
    /// RGBA.
    #[wem(value = 1)]
    RGBA,
    /// Alpha.
    #[wem(value = 2)]
    Alpha,
    /// Red.
    #[wem(value = 3)]
    Red,
    /// Green.
    #[wem(value = 4)]
    Green,
    /// Blue.
    #[wem(value = 5)]
    Blue,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named M3FresnelMode choices; unrecognized codes remain representable.
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
pub enum M3FresnelMode {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Standard.
    #[wem(value = 1)]
    Standard,
    /// Inverted.
    #[wem(value = 2)]
    Inverted,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named M3MaterialClass choices; unrecognized codes remain representable.
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
pub enum M3MaterialClass {
    #[default]
    /// Unit.
    #[wem(value = 0)]
    Unit,
    /// Building.
    #[wem(value = 1)]
    Building,
    /// Doodad.
    #[wem(value = 2)]
    Doodad,
    /// SpecialFX.
    #[wem(value = 3)]
    SpecialFX,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named M3LayerBlendOp choices; unrecognized codes remain representable.
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
pub enum M3LayerBlendOp {
    #[default]
    /// Mod.
    #[wem(value = 0)]
    Mod,
    /// Mod2x.
    #[wem(value = 1)]
    Mod2x,
    /// Add.
    #[wem(value = 2)]
    Add,
    /// Lerp.
    #[wem(value = 3)]
    Lerp,
    /// TeamColorEmissiveAdd.
    #[wem(value = 4)]
    TeamColorEmissiveAdd,
    /// TeamColorDiffuseAdd.
    #[wem(value = 5)]
    TeamColorDiffuseAdd,
    /// AddNoAlpha.
    #[wem(value = 6)]
    AddNoAlpha,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named M3SpecularMode choices; unrecognized codes remain representable.
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
pub enum M3SpecularMode {
    #[default]
    /// RGB.
    #[wem(value = 0)]
    RGB,
    /// AlphaOnly.
    #[wem(value = 1)]
    AlphaOnly,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named M3VolumeFalloffType choices; unrecognized codes remain representable.
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
pub enum M3VolumeFalloffType {
    #[default]
    /// Linear.
    #[wem(value = 0)]
    Linear,
    /// Exponential.
    #[wem(value = 1)]
    Exponential,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named M3VolumeNoiseCameraMode choices; unrecognized codes remain representable.
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
pub enum M3VolumeNoiseCameraMode {
    #[default]
    /// Outside.
    #[wem(value = 0)]
    Outside,
    /// Inside.
    #[wem(value = 1)]
    Inside,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

bitfield! {
    /// M3VolumeNoiseMaterialFlag bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M3VolumeNoiseMaterialFlag(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the DrawAfterTransparency bit.
    pub draw_after_transparency, set_draw_after_transparency: 0;
}

impl M3VolumeNoiseMaterialFlag {
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
    /// Bit mask for DrawAfterTransparency.
    pub const DRAW_AFTER_TRANSPARENCY: Self = Self(1);
}
impl From<u32> for M3VolumeNoiseMaterialFlag {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<M3VolumeNoiseMaterialFlag> for u32 {
    fn from(value: M3VolumeNoiseMaterialFlag) -> Self {
        value.bits()
    }
}

bitfield! {
    /// M3ReflectionMaterialFlag bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M3ReflectionMaterialFlag(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the UseReflectionMap bit.
    pub use_reflection_map, set_use_reflection_map: 0;
    /// Returns or changes the UseDisplacementMap bit.
    pub use_displacement_map, set_use_displacement_map: 1;
    /// Returns or changes the RenderInTransparentPass bit.
    pub render_in_transparent_pass, set_render_in_transparent_pass: 2;
    /// Returns or changes the Blurring bit.
    pub blurring, set_blurring: 3;
    /// Returns or changes the UseBlurMap bit.
    pub use_blur_map, set_use_blur_map: 4;
}

impl M3ReflectionMaterialFlag {
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
    /// Bit mask for UseReflectionMap.
    pub const USE_REFLECTION_MAP: Self = Self(1);
    /// Bit mask for UseDisplacementMap.
    pub const USE_DISPLACEMENT_MAP: Self = Self(2);
    /// Bit mask for RenderInTransparentPass.
    pub const RENDER_IN_TRANSPARENT_PASS: Self = Self(4);
    /// Bit mask for Blurring.
    pub const BLURRING: Self = Self(8);
    /// Bit mask for UseBlurMap.
    pub const USE_BLUR_MAP: Self = Self(16);
}
impl From<u32> for M3ReflectionMaterialFlag {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<M3ReflectionMaterialFlag> for u32 {
    fn from(value: M3ReflectionMaterialFlag) -> Self {
        value.bits()
    }
}

/// Named M3MaterialShaderType choices; unrecognized codes remain representable.
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
pub enum M3MaterialShaderType {
    #[default]
    /// Material.
    #[wem(value = 0)]
    Material,
    /// MaterialMedium.
    #[wem(value = 1)]
    MaterialMedium,
    /// MaterialSimple.
    #[wem(value = 2)]
    MaterialSimple,
    /// MaterialParticle.
    #[wem(value = 3)]
    MaterialParticle,
    /// MaterialSplat.
    #[wem(value = 4)]
    MaterialSplat,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u8),
}

bitfield! {
    /// MdxMaterialFlag bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct MdxMaterialFlag(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the ConstantColor bit.
    pub constant_color, set_constant_color: 0;
    /// Returns or changes the TwoSided bit.
    pub two_sided, set_two_sided: 1;
    /// Returns or changes the Unfogged bit.
    pub unfogged, set_unfogged: 2;
    /// Returns or changes the SortPrimsNearZ bit.
    pub sort_prims_near_z, set_sort_prims_near_z: 3;
    /// Returns or changes the SortPrimsFarZ bit.
    pub sort_prims_far_z, set_sort_prims_far_z: 4;
    /// Returns or changes the FullResolution bit.
    pub full_resolution, set_full_resolution: 5;
}

impl MdxMaterialFlag {
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
    /// Bit mask for ConstantColor.
    pub const CONSTANT_COLOR: Self = Self(1);
    /// Bit mask for TwoSided.
    pub const TWO_SIDED: Self = Self(2);
    /// Bit mask for Unfogged.
    pub const UNFOGGED: Self = Self(4);
    /// Bit mask for SortPrimsNearZ.
    pub const SORT_PRIMS_NEAR_Z: Self = Self(8);
    /// Bit mask for SortPrimsFarZ.
    pub const SORT_PRIMS_FAR_Z: Self = Self(16);
    /// Bit mask for FullResolution.
    pub const FULL_RESOLUTION: Self = Self(32);
}
impl From<u32> for MdxMaterialFlag {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<MdxMaterialFlag> for u32 {
    fn from(value: MdxMaterialFlag) -> Self {
        value.bits()
    }
}

/// Named MdxFilterMode choices; unrecognized codes remain representable.
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
pub enum MdxFilterMode {
    #[default]
    /// None.
    #[wem(value = 0)]
    None,
    /// Transparent.
    #[wem(value = 1)]
    Transparent,
    /// Blend.
    #[wem(value = 2)]
    Blend,
    /// Additive.
    #[wem(value = 3)]
    Additive,
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
    Unknown(u32),
}

bitfield! {
    /// MdxShadingFlag bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct MdxShadingFlag(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the Unshaded bit.
    pub unshaded, set_unshaded: 0;
    /// Returns or changes the SphereEnvMap bit.
    pub sphere_env_map, set_sphere_env_map: 1;
    /// Returns or changes the WrapWidth bit.
    pub wrap_width, set_wrap_width: 2;
    /// Returns or changes the WrapHeight bit.
    pub wrap_height, set_wrap_height: 3;
    /// Returns or changes the TwoSided bit.
    pub two_sided, set_two_sided: 4;
    /// Returns or changes the Unfogged bit.
    pub unfogged, set_unfogged: 5;
    /// Returns or changes the NoDepthTest bit.
    pub no_depth_test, set_no_depth_test: 6;
    /// Returns or changes the NoDepthSet bit.
    pub no_depth_set, set_no_depth_set: 7;
    /// Returns or changes the Unlit bit.
    pub unlit, set_unlit: 8;
    /// Returns or changes the BackFacesForShadows bit.
    pub back_faces_for_shadows, set_back_faces_for_shadows: 9;
    /// Returns or changes the AmbientOcclusion bit.
    pub ambient_occlusion, set_ambient_occlusion: 10;
}

impl MdxShadingFlag {
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
    /// Bit mask for Unshaded.
    pub const UNSHADED: Self = Self(1);
    /// Bit mask for SphereEnvMap.
    pub const SPHERE_ENV_MAP: Self = Self(2);
    /// Bit mask for WrapWidth.
    pub const WRAP_WIDTH: Self = Self(4);
    /// Bit mask for WrapHeight.
    pub const WRAP_HEIGHT: Self = Self(8);
    /// Bit mask for TwoSided.
    pub const TWO_SIDED: Self = Self(16);
    /// Bit mask for Unfogged.
    pub const UNFOGGED: Self = Self(32);
    /// Bit mask for NoDepthTest.
    pub const NO_DEPTH_TEST: Self = Self(64);
    /// Bit mask for NoDepthSet.
    pub const NO_DEPTH_SET: Self = Self(128);
    /// Bit mask for Unlit.
    pub const UNLIT: Self = Self(256);
    /// Bit mask for BackFacesForShadows.
    pub const BACK_FACES_FOR_SHADOWS: Self = Self(512);
    /// Bit mask for AmbientOcclusion.
    pub const AMBIENT_OCCLUSION: Self = Self(1024);
}
impl From<u32> for MdxShadingFlag {
    fn from(value: u32) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<MdxShadingFlag> for u32 {
    fn from(value: MdxShadingFlag) -> Self {
        value.bits()
    }
}

/// Named MdxShaderType choices; unrecognized codes remain representable.
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
pub enum MdxShaderType {
    #[default]
    /// SD.
    #[wem(value = 0)]
    SD,
    /// HD.
    #[wem(value = 1)]
    HD,
    /// SDOnHD.
    #[wem(value = 2)]
    SDOnHD,
    /// Terrain.
    #[wem(value = 3)]
    Terrain,
    /// Water.
    #[wem(value = 4)]
    Water,
    /// Fog.
    #[wem(value = 5)]
    Fog,
    /// Foliage.
    #[wem(value = 6)]
    Foliage,
    /// FoliagePush.
    #[wem(value = 7)]
    FoliagePush,
    /// Sprite.
    #[wem(value = 8)]
    Sprite,
    /// DebugTexture.
    #[wem(value = 9)]
    DebugTexture,
    /// DepthOfField.
    #[wem(value = 10)]
    DepthOfField,
    /// BloomCombine.
    #[wem(value = 11)]
    BloomCombine,
    /// BloomExtract.
    #[wem(value = 12)]
    BloomExtract,
    /// GaussianBlur.
    #[wem(value = 13)]
    GaussianBlur,
    /// Tonemap.
    #[wem(value = 14)]
    Tonemap,
    /// Movie.
    #[wem(value = 15)]
    Movie,
    /// FFXCMAAEdge0.
    #[wem(value = 16)]
    FFXCMAAEdge0,
    /// FFXCMAAEdge1.
    #[wem(value = 17)]
    FFXCMAAEdge1,
    /// FFXCMAAEdgeCombine.
    #[wem(value = 18)]
    FFXCMAAEdgeCombine,
    /// FFXCMAAProcessAndApply.
    #[wem(value = 19)]
    FFXCMAAProcessAndApply,
    /// PopcornFX.
    #[wem(value = 20)]
    PopcornFX,
    /// ConeIndicator.
    #[wem(value = 21)]
    ConeIndicator,
    /// CliffBlightMiscTerrain.
    #[wem(value = 22)]
    CliffBlightMiscTerrain,
    /// Distortion.
    #[wem(value = 23)]
    Distortion,
    /// Crystal.
    #[wem(value = 24)]
    Crystal,
    /// Imgui.
    #[wem(value = 25)]
    Imgui,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named MdxSlotType choices; unrecognized codes remain representable.
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
pub enum MdxSlotType {
    #[default]
    /// DiffuseMap.
    #[wem(value = 0)]
    DiffuseMap,
    /// NormalMap.
    #[wem(value = 1)]
    NormalMap,
    /// ORMMap.
    #[wem(value = 2)]
    ORMMap,
    /// EmissiveMap.
    #[wem(value = 3)]
    EmissiveMap,
    /// TeamColor.
    #[wem(value = 4)]
    TeamColor,
    /// EnvironmentMap.
    #[wem(value = 5)]
    EnvironmentMap,
    /// The format's unspecified choice.
    #[wem(value = 6)]
    Unspecified,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}

/// Named M2BlendingMode choices; unrecognized codes remain representable.
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
#[wem(value = u16)]
pub enum M2BlendingMode {
    #[default]
    /// Opaque.
    #[wem(value = 0)]
    Opaque,
    /// AlphaKey.
    #[wem(value = 1)]
    AlphaKey,
    /// Alpha.
    #[wem(value = 2)]
    Alpha,
    /// NoAlphaAdd.
    #[wem(value = 3)]
    NoAlphaAdd,
    /// Add.
    #[wem(value = 4)]
    Add,
    /// Mod.
    #[wem(value = 5)]
    Mod,
    /// Mod2x.
    #[wem(value = 6)]
    Mod2x,
    /// BlendAdd.
    #[wem(value = 7)]
    BlendAdd,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u16),
}

/// Named D3Group choices; unrecognized codes remain representable.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    wem::Read,
    wem::Write,
    wem::RecordLayout,
    wem::Validate,
    wem::VisitReferences,
    wem::Value,
)]
#[wem(value = i32)]
pub enum D3Group {
    /// The format's unspecified choice.
    #[wem(value = -1)]
    Unspecified,
    /// Actor.
    #[wem(value = 1)]
    Actor,
    /// Adventure.
    #[wem(value = 2)]
    Adventure,
    /// AIBehavior.
    #[wem(value = 3)]
    AIBehavior,
    /// AIState.
    #[wem(value = 4)]
    AIState,
    /// AmbientSound.
    #[wem(value = 5)]
    AmbientSound,
    /// Anim.
    #[wem(value = 6)]
    Anim,
    /// Anim2D.
    #[wem(value = 7)]
    Anim2D,
    /// AnimSet.
    #[wem(value = 8)]
    AnimSet,
    /// Appearance.
    #[wem(value = 9)]
    Appearance,
    /// Hero.
    #[wem(value = 10)]
    Hero,
    /// Cloth.
    #[wem(value = 11)]
    Cloth,
    /// Conversation.
    #[wem(value = 12)]
    Conversation,
    /// ConversationList.
    #[wem(value = 13)]
    ConversationList,
    /// EffectGroup.
    #[wem(value = 14)]
    EffectGroup,
    /// Encounter.
    #[wem(value = 15)]
    Encounter,
    /// Explosion.
    #[wem(value = 17)]
    Explosion,
    /// FlagSet.
    #[wem(value = 18)]
    FlagSet,
    /// Font.
    #[wem(value = 19)]
    Font,
    /// GameBalance.
    #[wem(value = 20)]
    GameBalance,
    /// Globals.
    #[wem(value = 21)]
    Globals,
    /// LevelArea.
    #[wem(value = 22)]
    LevelArea,
    /// Light.
    #[wem(value = 23)]
    Light,
    /// MarkerSet.
    #[wem(value = 24)]
    MarkerSet,
    /// Monster.
    #[wem(value = 25)]
    Monster,
    /// Observer.
    #[wem(value = 26)]
    Observer,
    /// Particle.
    #[wem(value = 27)]
    Particle,
    /// Physics.
    #[wem(value = 28)]
    Physics,
    /// Power.
    #[wem(value = 29)]
    Power,
    /// Quest.
    #[wem(value = 31)]
    Quest,
    /// Rope.
    #[wem(value = 32)]
    Rope,
    /// Scene.
    #[wem(value = 33)]
    Scene,
    /// SceneGroup.
    #[wem(value = 34)]
    SceneGroup,
    /// Script.
    #[wem(value = 35)]
    Script,
    /// ShaderMap.
    #[wem(value = 36)]
    ShaderMap,
    /// Shaders.
    #[wem(value = 37)]
    Shaders,
    /// Shakes.
    #[wem(value = 38)]
    Shakes,
    /// SkillKit.
    #[wem(value = 39)]
    SkillKit,
    /// Sound.
    #[wem(value = 40)]
    Sound,
    /// SoundBank.
    #[wem(value = 41)]
    SoundBank,
    /// StringList.
    #[wem(value = 42)]
    StringList,
    /// Surface.
    #[wem(value = 43)]
    Surface,
    /// Textures.
    #[wem(value = 44)]
    Textures,
    /// Trail.
    #[wem(value = 45)]
    Trail,
    /// UI.
    #[wem(value = 46)]
    UI,
    /// Weather.
    #[wem(value = 47)]
    Weather,
    /// Worlds.
    #[wem(value = 48)]
    Worlds,
    /// Recipe.
    #[wem(value = 49)]
    Recipe,
    /// Condition.
    #[wem(value = 51)]
    Condition,
    /// TreasureClass.
    #[wem(value = 52)]
    TreasureClass,
    /// Account.
    #[wem(value = 53)]
    Account,
    /// Conductor.
    #[wem(value = 54)]
    Conductor,
    /// TimedEvent.
    #[wem(value = 55)]
    TimedEvent,
    /// Act.
    #[wem(value = 56)]
    Act,
    /// Material.
    #[wem(value = 57)]
    Material,
    /// QuestRange.
    #[wem(value = 58)]
    QuestRange,
    /// Lore.
    #[wem(value = 59)]
    Lore,
    /// Reverb.
    #[wem(value = 60)]
    Reverb,
    /// PhysMesh.
    #[wem(value = 61)]
    PhysMesh,
    /// Music.
    #[wem(value = 62)]
    Music,
    /// Tutorial.
    #[wem(value = 63)]
    Tutorial,
    /// BossEncounter.
    #[wem(value = 64)]
    BossEncounter,
    /// ControlScheme.
    #[wem(value = 65)]
    ControlScheme,
    /// Accolade.
    #[wem(value = 66)]
    Accolade,
    /// AnimTree.
    #[wem(value = 67)]
    AnimTree,
    /// Vibrations.
    #[wem(value = 68)]
    Vibrations,
    /// DungeonFinder.
    #[wem(value = 69)]
    DungeonFinder,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(i32),
}
impl Default for D3Group {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// M2MaterialFlag bits, including unrecognized flags.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M2MaterialFlag(u16);
    /// Returns the exact stored bits.
    pub bits, _: 15, 0;
    /// Returns or changes the Unlit bit.
    pub unlit, set_unlit: 0;
    /// Returns or changes the Unfogged bit.
    pub unfogged, set_unfogged: 1;
    /// Returns or changes the TwoSided bit.
    pub two_sided, set_two_sided: 2;
    /// Returns or changes the DepthTest bit.
    pub depth_test, set_depth_test: 3;
    /// Returns or changes the DepthWrite bit.
    pub depth_write, set_depth_write: 4;
    /// Returns or changes the NoAlphaComposite bit.
    pub no_alpha_composite, set_no_alpha_composite: 11;
}

impl M2MaterialFlag {
    /// Retains every stored bit, including flags without known names.
    pub const fn from_bits_retain(bits: u16) -> Self {
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
    /// Bit mask for TwoSided.
    pub const TWO_SIDED: Self = Self(4);
    /// Bit mask for DepthTest.
    pub const DEPTH_TEST: Self = Self(8);
    /// Bit mask for DepthWrite.
    pub const DEPTH_WRITE: Self = Self(16);
    /// Bit mask for NoAlphaComposite.
    pub const NO_ALPHA_COMPOSITE: Self = Self(2048);
}
impl From<u16> for M2MaterialFlag {
    fn from(value: u16) -> Self {
        Self::from_bits_retain(value)
    }
}
impl From<M2MaterialFlag> for u16 {
    fn from(value: M2MaterialFlag) -> Self {
        value.bits()
    }
}

/// Named M2ParticleProperty choices; unrecognized codes remain representable.
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
pub enum M2ParticleProperty {
    #[default]
    /// Speed.
    #[wem(value = 0)]
    Speed,
    /// SpeedVariation.
    #[wem(value = 1)]
    SpeedVariation,
    /// VerticalRange.
    #[wem(value = 2)]
    VerticalRange,
    /// HorizontalRange.
    #[wem(value = 3)]
    HorizontalRange,
    /// Gravity.
    #[wem(value = 4)]
    Gravity,
    /// Lifespan.
    #[wem(value = 5)]
    Lifespan,
    /// EmissionRate.
    #[wem(value = 6)]
    EmissionRate,
    /// Width.
    #[wem(value = 7)]
    Width,
    /// Length.
    #[wem(value = 8)]
    Length,
    /// ZSource.
    #[wem(value = 9)]
    ZSource,
    /// Count.
    #[wem(value = 10)]
    Count,
    /// Unrecognized wire code, preserved without interpretation.
    #[wem(unknown)]
    Unknown(u32),
}
