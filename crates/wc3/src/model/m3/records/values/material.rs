//! Native material selectors and bit masks.
use crate::model::m3;
use bitfield::bitfield;

/// Native MaterialKind selector.
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
pub enum MaterialKind {
    /// Selects Standard.
    #[m3(value = 1)]
    Standard,
    /// Selects Displacement.
    #[m3(value = 2)]
    Displacement,
    /// Selects Composite.
    #[m3(value = 3)]
    Composite,
    /// Selects Terrain.
    #[m3(value = 4)]
    Terrain,
    /// Selects Volume.
    #[m3(value = 5)]
    Volume,
    /// Selects VolumeNoise.
    #[m3(value = 6)]
    VolumeNoise,
    /// Selects Creep.
    #[m3(value = 7)]
    Creep,
    /// Selects Hair.
    #[m3(value = 8)]
    Hair,
    /// Selects SplatTerrainBake.
    #[m3(value = 9)]
    SplatTerrainBake,
    /// Selects Reflection.
    #[m3(value = 10)]
    Reflection,
    /// Selects LensFlare.
    #[m3(value = 11)]
    LensFlare,
    /// Selects DataDriven.
    #[m3(value = 12)]
    DataDriven,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for MaterialKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native BlendMode selector.
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
pub enum BlendMode {
    /// Selects Opaque.
    #[m3(value = 0)]
    Opaque,
    /// Selects AlphaBlend.
    #[m3(value = 1)]
    AlphaBlend,
    /// Selects Add.
    #[m3(value = 2)]
    Add,
    /// Selects AlphaAdd.
    #[m3(value = 3)]
    AlphaAdd,
    /// Selects Mod.
    #[m3(value = 4)]
    Mod,
    /// Selects Mod2x.
    #[m3(value = 5)]
    Mod2x,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for BlendMode {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native UvMappingMode selector.
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
pub enum UvMappingMode {
    /// Selects ExplicitUV0.
    #[m3(value = 0)]
    ExplicitUV0,
    /// Selects ExplicitUV1.
    #[m3(value = 1)]
    ExplicitUV1,
    /// Selects ReflectCubicEnvio.
    #[m3(value = 2)]
    ReflectCubicEnvio,
    /// Selects ReflectSphericalEnvio.
    #[m3(value = 3)]
    ReflectSphericalEnvio,
    /// Selects PlanarLocalZ.
    #[m3(value = 4)]
    PlanarLocalZ,
    /// Selects PlanarWorldZ.
    #[m3(value = 5)]
    PlanarWorldZ,
    /// Selects ParticleFlipbook.
    #[m3(value = 6)]
    ParticleFlipbook,
    /// Selects CubicEnvio.
    #[m3(value = 7)]
    CubicEnvio,
    /// Selects SphericalEnvio.
    #[m3(value = 8)]
    SphericalEnvio,
    /// Selects ExplicitUV2.
    #[m3(value = 9)]
    ExplicitUV2,
    /// Selects ExplicitUV3.
    #[m3(value = 10)]
    ExplicitUV3,
    /// Selects PlanarLocalX.
    #[m3(value = 11)]
    PlanarLocalX,
    /// Selects PlanarLocalY.
    #[m3(value = 12)]
    PlanarLocalY,
    /// Selects PlanarWorldX.
    #[m3(value = 13)]
    PlanarWorldX,
    /// Selects PlanarWorldY.
    #[m3(value = 14)]
    PlanarWorldY,
    /// Selects ScreenSpace.
    #[m3(value = 15)]
    ScreenSpace,
    /// Selects TriPlanarLocal.
    #[m3(value = 16)]
    TriPlanarLocal,
    /// Selects TriPlanarWorld.
    #[m3(value = 17)]
    TriPlanarWorld,
    /// Selects TriPlanarWorldLocalZ.
    #[m3(value = 18)]
    TriPlanarWorldLocalZ,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for UvMappingMode {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native ColorChannel selector.
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
pub enum ColorChannel {
    /// Selects RGB channels.
    #[m3(value = 0)]
    Rgb,
    /// Selects RGBA channels.
    #[m3(value = 1)]
    Rgba,
    /// Selects Alpha.
    #[m3(value = 2)]
    Alpha,
    /// Selects Red.
    #[m3(value = 3)]
    Red,
    /// Selects Green.
    #[m3(value = 4)]
    Green,
    /// Selects Blue.
    #[m3(value = 5)]
    Blue,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for ColorChannel {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native FresnelMode selector.
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
pub enum FresnelMode {
    /// Selects None.
    #[m3(value = 0)]
    None,
    /// Selects Standard.
    #[m3(value = 1)]
    Standard,
    /// Selects Inverted.
    #[m3(value = 2)]
    Inverted,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for FresnelMode {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native MaterialClass selector.
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
pub enum MaterialClass {
    /// Selects Unit.
    #[m3(value = 0)]
    Unit,
    /// Selects Building.
    #[m3(value = 1)]
    Building,
    /// Selects Doodad.
    #[m3(value = 2)]
    Doodad,
    /// Selects SpecialFX.
    #[m3(value = 3)]
    SpecialFX,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for MaterialClass {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native LayerBlendOp selector.
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
pub enum LayerBlendOp {
    /// Selects Mod.
    #[m3(value = 0)]
    Mod,
    /// Selects Mod2x.
    #[m3(value = 1)]
    Mod2x,
    /// Selects Add.
    #[m3(value = 2)]
    Add,
    /// Selects Lerp.
    #[m3(value = 3)]
    Lerp,
    /// Selects TeamColorEmissiveAdd.
    #[m3(value = 4)]
    TeamColorEmissiveAdd,
    /// Selects TeamColorDiffuseAdd.
    #[m3(value = 5)]
    TeamColorDiffuseAdd,
    /// Selects AddNoAlpha.
    #[m3(value = 6)]
    AddNoAlpha,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for LayerBlendOp {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native SpecularMode selector.
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
pub enum SpecularMode {
    /// Selects RGB channels.
    #[m3(value = 0)]
    Rgb,
    /// Selects AlphaOnly.
    #[m3(value = 1)]
    AlphaOnly,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for SpecularMode {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native VolumeFalloff selector.
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
pub enum VolumeFalloff {
    /// Selects Linear.
    #[m3(value = 0)]
    Linear,
    /// Selects Exponential.
    #[m3(value = 1)]
    Exponential,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for VolumeFalloff {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native VolumeCameraMode selector.
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
pub enum VolumeCameraMode {
    /// Selects Outside.
    #[m3(value = 0)]
    Outside,
    /// Selects Inside.
    #[m3(value = 1)]
    Inside,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for VolumeCameraMode {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native ShaderType selector.
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
pub enum ShaderType {
    /// Selects Material.
    #[m3(value = 0)]
    Material,
    /// Selects MaterialMedium.
    #[m3(value = 1)]
    MaterialMedium,
    /// Selects MaterialSimple.
    #[m3(value = 2)]
    MaterialSimple,
    /// Selects MaterialParticle.
    #[m3(value = 3)]
    MaterialParticle,
    /// Selects MaterialSplat.
    #[m3(value = 4)]
    MaterialSplat,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u8),
}
impl Default for ShaderType {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// Native LayerFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct LayerFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the uv wrap x control.
    pub uv_wrap_x, set_uv_wrap_x: 2;
    /// Returns or changes the uv wrap y control.
    pub uv_wrap_y, set_uv_wrap_y: 3;
    /// Returns or changes the color invert control.
    pub color_invert, set_color_invert: 4;
    /// Returns or changes the color clamp control.
    pub color_clamp, set_color_clamp: 5;
    /// Returns or changes the color add control.
    pub color_add, set_color_add: 6;
    /// Returns or changes the color mult control.
    pub color_mult, set_color_mult: 7;
    /// Returns or changes the particle uv flipbook control.
    pub particle_uv_flipbook, set_particle_uv_flipbook: 8;
    /// Returns or changes the video control.
    pub video, set_video: 9;
    /// Returns or changes the color control.
    pub color, set_color: 10;
    /// Returns or changes the fresnel transform control.
    pub fresnel_transform, set_fresnel_transform: 14;
    /// Returns or changes the fresnel normalize control.
    pub fresnel_normalize, set_fresnel_normalize: 15;
    /// Returns or changes the fresnel local transform control.
    pub fresnel_local_transform, set_fresnel_local_transform: 16;
    /// Returns or changes the fresnel do not mirror control.
    pub fresnel_do_not_mirror, set_fresnel_do_not_mirror: 17;
}
impl From<u32> for LayerFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<LayerFlags> for u32 {
    fn from(value: LayerFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native MaterialFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct MaterialFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the vertex color control.
    pub vertex_color, set_vertex_color: 0;
    /// Returns or changes the vertex alpha control.
    pub vertex_alpha, set_vertex_alpha: 1;
    /// Returns or changes the unfogged control.
    pub unfogged, set_unfogged: 2;
    /// Returns or changes the two sided control.
    pub two_sided, set_two_sided: 3;
    /// Returns or changes the unshaded control.
    pub unshaded, set_unshaded: 4;
    /// Returns or changes the no shadows cast control.
    pub no_shadows_cast, set_no_shadows_cast: 5;
    /// Returns or changes the no hittest control.
    pub no_hittest, set_no_hittest: 6;
    /// Returns or changes the no shadows receive control.
    pub no_shadows_receive, set_no_shadows_receive: 7;
    /// Returns or changes the depth prepass control.
    pub depth_prepass, set_depth_prepass: 8;
    /// Returns or changes the terrain hdr control.
    pub terrain_hdr, set_terrain_hdr: 9;
    /// Returns or changes the simulate roughness control.
    pub simulate_roughness, set_simulate_roughness: 11;
    /// Returns or changes the pixel forward lighting control.
    pub pixel_forward_lighting, set_pixel_forward_lighting: 12;
    /// Returns or changes the depth fog control.
    pub depth_fog, set_depth_fog: 13;
    /// Returns or changes the transparent shadows control.
    pub transparent_shadows, set_transparent_shadows: 14;
    /// Returns or changes the decal lighting control.
    pub decal_lighting, set_decal_lighting: 15;
    /// Returns or changes the transparent depth effects control.
    pub transparent_depth_effects, set_transparent_depth_effects: 16;
    /// Returns or changes the transparent local lights control.
    pub transparent_local_lights, set_transparent_local_lights: 17;
    /// Returns or changes the disable soft control.
    pub disable_soft, set_disable_soft: 18;
    /// Returns or changes the double lambert control.
    pub double_lambert, set_double_lambert: 19;
    /// Returns or changes the hair layer sorting control.
    pub hair_layer_sorting, set_hair_layer_sorting: 20;
    /// Returns or changes the accept splats control.
    pub accept_splats, set_accept_splats: 21;
    /// Returns or changes the decal low required control.
    pub decal_low_required, set_decal_low_required: 22;
    /// Returns or changes the emis low required control.
    pub emis_low_required, set_emis_low_required: 23;
    /// Returns or changes the spec low required control.
    pub spec_low_required, set_spec_low_required: 24;
    /// Returns or changes the accept splats only control.
    pub accept_splats_only, set_accept_splats_only: 25;
    /// Returns or changes the background object control.
    pub background_object, set_background_object: 26;
    /// Returns or changes the depth prepass low required control.
    pub depth_prepass_low_required, set_depth_prepass_low_required: 28;
    /// Returns or changes the no highlighting control.
    pub no_highlighting, set_no_highlighting: 29;
    /// Returns or changes the clamp output control.
    pub clamp_output, set_clamp_output: 30;
    /// Returns or changes the geometry visible control.
    pub geometry_visible, set_geometry_visible: 31;
}
impl From<u32> for MaterialFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<MaterialFlags> for u32 {
    fn from(value: MaterialFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native AdditionalMaterialFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct AdditionalMaterialFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the depth blend falloff control.
    pub depth_blend_falloff, set_depth_blend_falloff: 0;
    /// Returns or changes the vertex color control.
    pub vertex_color, set_vertex_color: 2;
    /// Returns or changes the vertex alpha control.
    pub vertex_alpha, set_vertex_alpha: 3;
}
impl From<u32> for AdditionalMaterialFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<AdditionalMaterialFlags> for u32 {
    fn from(value: AdditionalMaterialFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native DisplacementFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct DisplacementFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
}
impl From<u32> for DisplacementFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<DisplacementFlags> for u32 {
    fn from(value: DisplacementFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native VolumeFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct VolumeFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
}
impl From<u32> for VolumeFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<VolumeFlags> for u32 {
    fn from(value: VolumeFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native VolumeNoiseFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct VolumeNoiseFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the draw after transparency control.
    pub draw_after_transparency, set_draw_after_transparency: 0;
}
impl From<u32> for VolumeNoiseFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<VolumeNoiseFlags> for u32 {
    fn from(value: VolumeNoiseFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native ReflectionFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct ReflectionFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the use layer norm control.
    pub use_layer_norm, set_use_layer_norm: 0;
    /// Returns or changes the use layer strength control.
    pub use_layer_strength, set_use_layer_strength: 1;
    /// Returns or changes the render in transparent pass control.
    pub render_in_transparent_pass, set_render_in_transparent_pass: 2;
    /// Returns or changes the blurring control.
    pub blurring, set_blurring: 3;
}
impl From<u32> for ReflectionFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ReflectionFlags> for u32 {
    fn from(value: ReflectionFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native ShaderFresnelFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct ShaderFresnelFlags(u8);
    /// Returns the complete stored bit mask.
    pub bits, _: 7, 0;
}
impl From<u8> for ShaderFresnelFlags {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<ShaderFresnelFlags> for u8 {
    fn from(value: ShaderFresnelFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native RttChannels mask, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct RttChannels(u32);
    /// Complete native bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes render target 0.
    pub channel_0, set_channel_0: 0;
    /// Returns or changes render target 1.
    pub channel_1, set_channel_1: 1;
    /// Returns or changes render target 2.
    pub channel_2, set_channel_2: 2;
    /// Returns or changes render target 3.
    pub channel_3, set_channel_3: 3;
    /// Returns or changes render target 4.
    pub channel_4, set_channel_4: 4;
    /// Returns or changes render target 5.
    pub channel_5, set_channel_5: 5;
    /// Returns or changes render target 6.
    pub channel_6, set_channel_6: 6;
}
impl From<u32> for RttChannels {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<RttChannels> for u32 {
    fn from(value: RttChannels) -> Self {
        value.bits()
    }
}
