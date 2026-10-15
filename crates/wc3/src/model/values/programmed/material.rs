//! M3 material selectors and bit masks.
use crate::model::materials::state::BlendOperations;
use crate::model::profiles::Unavailable;
use bitfield::bitfield;

/// M3 MaterialKind selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MaterialKind {
    /// Selects Standard.
    Standard,
    /// Selects Displacement.
    Displacement,
    /// Selects Composite.
    Composite,
    /// Selects Terrain.
    Terrain,
    /// Selects Volume.
    Volume,
    /// Selects VolumeNoise.
    VolumeNoise,
    /// Selects Creep.
    Creep,
    /// Selects Hair.
    Hair,
    /// Selects SplatTerrainBake.
    SplatTerrainBake,
    /// Selects Reflection.
    Reflection,
    /// Selects LensFlare.
    LensFlare,
    /// Selects DataDriven.
    DataDriven,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl MaterialKind {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Standard => 1,
            Self::Displacement => 2,
            Self::Composite => 3,
            Self::Terrain => 4,
            Self::Volume => 5,
            Self::VolumeNoise => 6,
            Self::Creep => 7,
            Self::Hair => 8,
            Self::SplatTerrainBake => 9,
            Self::Reflection => 10,
            Self::LensFlare => 11,
            Self::DataDriven => 12,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::Standard,
            2 => Self::Displacement,
            3 => Self::Composite,
            4 => Self::Terrain,
            5 => Self::Volume,
            6 => Self::VolumeNoise,
            7 => Self::Creep,
            8 => Self::Hair,
            9 => Self::SplatTerrainBake,
            10 => Self::Reflection,
            11 => Self::LensFlare,
            12 => Self::DataDriven,
            value => Self::Unknown(value),
        }
    }
}

impl Default for MaterialKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 BlendMode selector.
pub type BlendMode = BlendOperations<Unavailable, Unavailable, (), u32>;
impl BlendMode {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Opaque => 0,
            Self::Alpha => 1,
            Self::Additive => 2,
            Self::AdditiveAlpha(()) => 3,
            Self::Modulate => 4,
            Self::Modulate2x => 5,
            Self::Unknown(value) => value,
            Self::AlphaKey(value) | Self::Extension(value) => match value {},
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Opaque,
            1 => Self::Alpha,
            2 => Self::Additive,
            3 => Self::AdditiveAlpha(()),
            4 => Self::Modulate,
            5 => Self::Modulate2x,
            value => Self::Unknown(value),
        }
    }
}

/// M3 UvMappingMode selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum UvMappingMode {
    /// Selects ExplicitUV0.
    ExplicitUV0,
    /// Selects ExplicitUV1.
    ExplicitUV1,
    /// Selects ReflectCubicEnvio.
    ReflectCubicEnvio,
    /// Selects ReflectSphericalEnvio.
    ReflectSphericalEnvio,
    /// Selects PlanarLocalZ.
    PlanarLocalZ,
    /// Selects PlanarWorldZ.
    PlanarWorldZ,
    /// Selects ParticleFlipbook.
    ParticleFlipbook,
    /// Selects CubicEnvio.
    CubicEnvio,
    /// Selects SphericalEnvio.
    SphericalEnvio,
    /// Selects ExplicitUV2.
    ExplicitUV2,
    /// Selects ExplicitUV3.
    ExplicitUV3,
    /// Selects PlanarLocalX.
    PlanarLocalX,
    /// Selects PlanarLocalY.
    PlanarLocalY,
    /// Selects PlanarWorldX.
    PlanarWorldX,
    /// Selects PlanarWorldY.
    PlanarWorldY,
    /// Selects ScreenSpace.
    ScreenSpace,
    /// Selects TriPlanarLocal.
    TriPlanarLocal,
    /// Selects TriPlanarWorld.
    TriPlanarWorld,
    /// Selects TriPlanarWorldLocalZ.
    TriPlanarWorldLocalZ,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl UvMappingMode {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::ExplicitUV0 => 0,
            Self::ExplicitUV1 => 1,
            Self::ReflectCubicEnvio => 2,
            Self::ReflectSphericalEnvio => 3,
            Self::PlanarLocalZ => 4,
            Self::PlanarWorldZ => 5,
            Self::ParticleFlipbook => 6,
            Self::CubicEnvio => 7,
            Self::SphericalEnvio => 8,
            Self::ExplicitUV2 => 9,
            Self::ExplicitUV3 => 10,
            Self::PlanarLocalX => 11,
            Self::PlanarLocalY => 12,
            Self::PlanarWorldX => 13,
            Self::PlanarWorldY => 14,
            Self::ScreenSpace => 15,
            Self::TriPlanarLocal => 16,
            Self::TriPlanarWorld => 17,
            Self::TriPlanarWorldLocalZ => 18,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::ExplicitUV0,
            1 => Self::ExplicitUV1,
            2 => Self::ReflectCubicEnvio,
            3 => Self::ReflectSphericalEnvio,
            4 => Self::PlanarLocalZ,
            5 => Self::PlanarWorldZ,
            6 => Self::ParticleFlipbook,
            7 => Self::CubicEnvio,
            8 => Self::SphericalEnvio,
            9 => Self::ExplicitUV2,
            10 => Self::ExplicitUV3,
            11 => Self::PlanarLocalX,
            12 => Self::PlanarLocalY,
            13 => Self::PlanarWorldX,
            14 => Self::PlanarWorldY,
            15 => Self::ScreenSpace,
            16 => Self::TriPlanarLocal,
            17 => Self::TriPlanarWorld,
            18 => Self::TriPlanarWorldLocalZ,
            value => Self::Unknown(value),
        }
    }
}

impl Default for UvMappingMode {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 ColorChannel selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ColorChannel {
    /// Selects RGB channels.
    Rgb,
    /// Selects RGBA channels.
    Rgba,
    /// Selects Alpha.
    Alpha,
    /// Selects Red.
    Red,
    /// Selects Green.
    Green,
    /// Selects Blue.
    Blue,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl ColorChannel {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Rgb => 0,
            Self::Rgba => 1,
            Self::Alpha => 2,
            Self::Red => 3,
            Self::Green => 4,
            Self::Blue => 5,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Rgb,
            1 => Self::Rgba,
            2 => Self::Alpha,
            3 => Self::Red,
            4 => Self::Green,
            5 => Self::Blue,
            value => Self::Unknown(value),
        }
    }
}

impl Default for ColorChannel {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 FresnelMode selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FresnelMode {
    /// Selects None.
    None,
    /// Selects Standard.
    Standard,
    /// Selects Inverted.
    Inverted,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl FresnelMode {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Standard => 1,
            Self::Inverted => 2,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Standard,
            2 => Self::Inverted,
            value => Self::Unknown(value),
        }
    }
}

impl Default for FresnelMode {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 MaterialClass selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum MaterialClass {
    /// Selects Unit.
    Unit,
    /// Selects Building.
    Building,
    /// Selects Doodad.
    Doodad,
    /// Selects SpecialFX.
    SpecialFX,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl MaterialClass {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Unit => 0,
            Self::Building => 1,
            Self::Doodad => 2,
            Self::SpecialFX => 3,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Unit,
            1 => Self::Building,
            2 => Self::Doodad,
            3 => Self::SpecialFX,
            value => Self::Unknown(value),
        }
    }
}

impl Default for MaterialClass {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 LayerBlendOp selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum LayerBlendOp {
    /// Selects Mod.
    Mod,
    /// Selects Mod2x.
    Mod2x,
    /// Selects Add.
    Add,
    /// Selects Lerp.
    Lerp,
    /// Selects TeamColorEmissiveAdd.
    TeamColorEmissiveAdd,
    /// Selects TeamColorDiffuseAdd.
    TeamColorDiffuseAdd,
    /// Selects AddNoAlpha.
    AddNoAlpha,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl LayerBlendOp {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Mod => 0,
            Self::Mod2x => 1,
            Self::Add => 2,
            Self::Lerp => 3,
            Self::TeamColorEmissiveAdd => 4,
            Self::TeamColorDiffuseAdd => 5,
            Self::AddNoAlpha => 6,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Mod,
            1 => Self::Mod2x,
            2 => Self::Add,
            3 => Self::Lerp,
            4 => Self::TeamColorEmissiveAdd,
            5 => Self::TeamColorDiffuseAdd,
            6 => Self::AddNoAlpha,
            value => Self::Unknown(value),
        }
    }
}

impl Default for LayerBlendOp {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 SpecularMode selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SpecularMode {
    /// Selects RGB channels.
    Rgb,
    /// Selects AlphaOnly.
    AlphaOnly,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl SpecularMode {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Rgb => 0,
            Self::AlphaOnly => 1,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Rgb,
            1 => Self::AlphaOnly,
            value => Self::Unknown(value),
        }
    }
}

impl Default for SpecularMode {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 VolumeFalloff selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VolumeFalloff {
    /// Selects Linear.
    Linear,
    /// Selects Exponential.
    Exponential,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl VolumeFalloff {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Linear => 0,
            Self::Exponential => 1,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Linear,
            1 => Self::Exponential,
            value => Self::Unknown(value),
        }
    }
}

impl Default for VolumeFalloff {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 VolumeCameraMode selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VolumeCameraMode {
    /// Selects Outside.
    Outside,
    /// Selects Inside.
    Inside,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl VolumeCameraMode {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Outside => 0,
            Self::Inside => 1,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Outside,
            1 => Self::Inside,
            value => Self::Unknown(value),
        }
    }
}

impl Default for VolumeCameraMode {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 ShaderType selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ShaderType {
    /// Selects Material.
    Material,
    /// Selects MaterialMedium.
    MaterialMedium,
    /// Selects MaterialSimple.
    MaterialSimple,
    /// Selects MaterialParticle.
    MaterialParticle,
    /// Selects MaterialSplat.
    MaterialSplat,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u8),
}
impl ShaderType {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u8 {
        match self {
            Self::Material => 0,
            Self::MaterialMedium => 1,
            Self::MaterialSimple => 2,
            Self::MaterialParticle => 3,
            Self::MaterialSplat => 4,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Material,
            1 => Self::MaterialMedium,
            2 => Self::MaterialSimple,
            3 => Self::MaterialParticle,
            4 => Self::MaterialSplat,
            value => Self::Unknown(value),
        }
    }
}

impl Default for ShaderType {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// M3 LayerFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 MaterialFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 AdditionalMaterialFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 DisplacementFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 VolumeFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 VolumeNoiseFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 ReflectionFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 ShaderFresnelFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
    /// M3 RttChannels mask, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
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
