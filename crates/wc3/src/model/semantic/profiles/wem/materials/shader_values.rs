//! Native-game shader selectors are authored independently of document texture resources.
use bitfield::bitfield;
/// Game-defined warcraft iii filter selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WarcraftFilter {
    /// None selector.
    #[default]
    None,
    /// Transparent selector.
    Transparent,
    /// Blend selector.
    Blend,
    /// Additive selector.
    Additive,
    /// Add Alpha selector.
    AddAlpha,
    /// Modulate selector.
    Modulate,
    /// Modulate2x selector.
    Modulate2x,
    /// Producer-defined selection.
    Unknown(u32),
}
impl WarcraftFilter {
    /// Retains game-defined selector codes without assigning new meaning.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Transparent,
            2 => Self::Blend,
            3 => Self::Additive,
            4 => Self::AddAlpha,
            5 => Self::Modulate,
            6 => Self::Modulate2x,
            value => Self::Unknown(value),
        }
    }
    /// Returns the game-defined selector value.
    pub const fn raw(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Transparent => 1,
            Self::Blend => 2,
            Self::Additive => 3,
            Self::AddAlpha => 4,
            Self::Modulate => 5,
            Self::Modulate2x => 6,
            Self::Unknown(value) => value,
        }
    }
}
/// Game-defined warcraft iii shader selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WarcraftShader {
    /// SD selector.
    #[default]
    SD,
    /// HD selector.
    HD,
    /// SDOn HD selector.
    SDOnHD,
    /// Terrain selector.
    Terrain,
    /// Water selector.
    Water,
    /// Fog selector.
    Fog,
    /// Foliage selector.
    Foliage,
    /// Foliage Push selector.
    FoliagePush,
    /// Sprite selector.
    Sprite,
    /// Debug Texture selector.
    DebugTexture,
    /// Depth Of Field selector.
    DepthOfField,
    /// Bloom Combine selector.
    BloomCombine,
    /// Bloom Extract selector.
    BloomExtract,
    /// Gaussian Blur selector.
    GaussianBlur,
    /// Tonemap selector.
    Tonemap,
    /// Movie selector.
    Movie,
    /// FFXCMAAEdge0 selector.
    FFXCMAAEdge0,
    /// FFXCMAAEdge1 selector.
    FFXCMAAEdge1,
    /// FFXCMAAEdge Combine selector.
    FFXCMAAEdgeCombine,
    /// FFXCMAAProcess And Apply selector.
    FFXCMAAProcessAndApply,
    /// Popcorn FX selector.
    PopcornFX,
    /// Cone Indicator selector.
    ConeIndicator,
    /// Cliff Blight Misc Terrain selector.
    CliffBlightMiscTerrain,
    /// Distortion selector.
    Distortion,
    /// Crystal selector.
    Crystal,
    /// Imgui selector.
    Imgui,
    /// Producer-defined selection.
    Unknown(u32),
}
impl WarcraftShader {
    /// Retains game-defined selector codes without assigning new meaning.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::SD,
            1 => Self::HD,
            2 => Self::SDOnHD,
            3 => Self::Terrain,
            4 => Self::Water,
            5 => Self::Fog,
            6 => Self::Foliage,
            7 => Self::FoliagePush,
            8 => Self::Sprite,
            9 => Self::DebugTexture,
            10 => Self::DepthOfField,
            11 => Self::BloomCombine,
            12 => Self::BloomExtract,
            13 => Self::GaussianBlur,
            14 => Self::Tonemap,
            15 => Self::Movie,
            16 => Self::FFXCMAAEdge0,
            17 => Self::FFXCMAAEdge1,
            18 => Self::FFXCMAAEdgeCombine,
            19 => Self::FFXCMAAProcessAndApply,
            20 => Self::PopcornFX,
            21 => Self::ConeIndicator,
            22 => Self::CliffBlightMiscTerrain,
            23 => Self::Distortion,
            24 => Self::Crystal,
            25 => Self::Imgui,
            value => Self::Unknown(value),
        }
    }
    /// Returns the game-defined selector value.
    pub const fn raw(self) -> u32 {
        match self {
            Self::SD => 0,
            Self::HD => 1,
            Self::SDOnHD => 2,
            Self::Terrain => 3,
            Self::Water => 4,
            Self::Fog => 5,
            Self::Foliage => 6,
            Self::FoliagePush => 7,
            Self::Sprite => 8,
            Self::DebugTexture => 9,
            Self::DepthOfField => 10,
            Self::BloomCombine => 11,
            Self::BloomExtract => 12,
            Self::GaussianBlur => 13,
            Self::Tonemap => 14,
            Self::Movie => 15,
            Self::FFXCMAAEdge0 => 16,
            Self::FFXCMAAEdge1 => 17,
            Self::FFXCMAAEdgeCombine => 18,
            Self::FFXCMAAProcessAndApply => 19,
            Self::PopcornFX => 20,
            Self::ConeIndicator => 21,
            Self::CliffBlightMiscTerrain => 22,
            Self::Distortion => 23,
            Self::Crystal => 24,
            Self::Imgui => 25,
            Self::Unknown(value) => value,
        }
    }
}
/// Game-defined warcraft iii textureslot selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WarcraftTextureSlot {
    /// Diffuse Map selector.
    #[default]
    DiffuseMap,
    /// Normal Map selector.
    NormalMap,
    /// ORMMap selector.
    ORMMap,
    /// Emissive Map selector.
    EmissiveMap,
    /// Team Color selector.
    TeamColor,
    /// Environment Map selector.
    EnvironmentMap,
    /// Unspecified selector.
    Unspecified,
    /// Producer-defined selection.
    Unknown(u32),
}
impl WarcraftTextureSlot {
    /// Retains game-defined selector codes without assigning new meaning.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::DiffuseMap,
            1 => Self::NormalMap,
            2 => Self::ORMMap,
            3 => Self::EmissiveMap,
            4 => Self::TeamColor,
            5 => Self::EnvironmentMap,
            6 => Self::Unspecified,
            value => Self::Unknown(value),
        }
    }
    /// Returns the game-defined selector value.
    pub const fn raw(self) -> u32 {
        match self {
            Self::DiffuseMap => 0,
            Self::NormalMap => 1,
            Self::ORMMap => 2,
            Self::EmissiveMap => 3,
            Self::TeamColor => 4,
            Self::EnvironmentMap => 5,
            Self::Unspecified => 6,
            Self::Unknown(value) => value,
        }
    }
}
/// Game-defined world of warcraft blend selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WowBlend {
    /// Opaque selector.
    #[default]
    Opaque,
    /// Alpha Key selector.
    AlphaKey,
    /// Alpha selector.
    Alpha,
    /// No Alpha Add selector.
    NoAlphaAdd,
    /// Add selector.
    Add,
    /// Mod selector.
    Mod,
    /// Mod2x selector.
    Mod2x,
    /// Blend Add selector.
    BlendAdd,
    /// Producer-defined selection.
    Unknown(u16),
}
impl WowBlend {
    /// Retains game-defined selector codes without assigning new meaning.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Opaque,
            1 => Self::AlphaKey,
            2 => Self::Alpha,
            3 => Self::NoAlphaAdd,
            4 => Self::Add,
            5 => Self::Mod,
            6 => Self::Mod2x,
            7 => Self::BlendAdd,
            value => Self::Unknown(value),
        }
    }
    /// Returns the game-defined selector value.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Opaque => 0,
            Self::AlphaKey => 1,
            Self::Alpha => 2,
            Self::NoAlphaAdd => 3,
            Self::Add => 4,
            Self::Mod => 5,
            Self::Mod2x => 6,
            Self::BlendAdd => 7,
            Self::Unknown(value) => value,
        }
    }
}
/// Game-defined world of warcraft texturesource selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WowTextureSource {
    /// File selector.
    #[default]
    File,
    /// Skin selector.
    Skin,
    /// Object Skin selector.
    ObjectSkin,
    /// Weapon Blade selector.
    WeaponBlade,
    /// Weapon Handle selector.
    WeaponHandle,
    /// Environment selector.
    Environment,
    /// Character Hair selector.
    CharacterHair,
    /// Character Facial Hair selector.
    CharacterFacialHair,
    /// Skin Extra selector.
    SkinExtra,
    /// Ui Skin selector.
    UiSkin,
    /// Tauren Mane selector.
    TaurenMane,
    /// Monster Skin1 selector.
    MonsterSkin1,
    /// Monster Skin2 selector.
    MonsterSkin2,
    /// Monster Skin3 selector.
    MonsterSkin3,
    /// Item Icon selector.
    ItemIcon,
    /// Guild Background Color selector.
    GuildBackgroundColor,
    /// Guild Emblem Color selector.
    GuildEmblemColor,
    /// Guild Border Color selector.
    GuildBorderColor,
    /// Guild Emblem selector.
    GuildEmblem,
    /// Producer-defined selection.
    Unknown(u8),
}
impl WowTextureSource {
    /// Retains game-defined selector codes without assigning new meaning.
    pub const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::File,
            1 => Self::Skin,
            2 => Self::ObjectSkin,
            3 => Self::WeaponBlade,
            4 => Self::WeaponHandle,
            5 => Self::Environment,
            6 => Self::CharacterHair,
            7 => Self::CharacterFacialHair,
            8 => Self::SkinExtra,
            9 => Self::UiSkin,
            10 => Self::TaurenMane,
            11 => Self::MonsterSkin1,
            12 => Self::MonsterSkin2,
            13 => Self::MonsterSkin3,
            14 => Self::ItemIcon,
            15 => Self::GuildBackgroundColor,
            16 => Self::GuildEmblemColor,
            17 => Self::GuildBorderColor,
            18 => Self::GuildEmblem,
            value => Self::Unknown(value),
        }
    }
    /// Returns the game-defined selector value.
    pub const fn raw(self) -> u8 {
        match self {
            Self::File => 0,
            Self::Skin => 1,
            Self::ObjectSkin => 2,
            Self::WeaponBlade => 3,
            Self::WeaponHandle => 4,
            Self::Environment => 5,
            Self::CharacterHair => 6,
            Self::CharacterFacialHair => 7,
            Self::SkinExtra => 8,
            Self::UiSkin => 9,
            Self::TaurenMane => 10,
            Self::MonsterSkin1 => 11,
            Self::MonsterSkin2 => 12,
            Self::MonsterSkin3 => 13,
            Self::ItemIcon => 14,
            Self::GuildBackgroundColor => 15,
            Self::GuildEmblemColor => 16,
            Self::GuildBorderColor => 17,
            Self::GuildEmblem => 18,
            Self::Unknown(value) => value,
        }
    }
}
bitfield! {
    /// Native-game WarcraftMaterialFlags options, including producer-defined bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct WarcraftMaterialFlags(u32);
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
impl WarcraftMaterialFlags {
    /// Retains known and producer-defined behavior bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// Native-game WarcraftShadingFlags options, including producer-defined bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct WarcraftShadingFlags(u32);
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
impl WarcraftShadingFlags {
    /// Retains known and producer-defined behavior bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// Native-game WowRenderFlags options, including producer-defined bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct WowRenderFlags(u16);
    /// Returns the exact stored bits.
    pub bits, _: 15, 0;
    /// Returns or changes Unlit.
    pub unlit, set_unlit: 0;
    /// Returns or changes Unfogged.
    pub unfogged, set_unfogged: 1;
    /// Returns or changes TwoSided.
    pub two_sided, set_two_sided: 2;
    /// Disables depth testing.
    pub no_depth_test, set_no_depth_test: 3;
    /// Disables depth-buffer writes.
    pub no_depth_write, set_no_depth_write: 4;
    /// Returns or changes NoAlphaComposite.
    pub no_alpha_composite, set_no_alpha_composite: 11;
}
impl WowRenderFlags {
    /// Retains known and producer-defined behavior bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// Native-game WowBatchFlags options, including producer-defined bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct WowBatchFlags(u8);
    /// Returns the exact stored bits.
    pub bits, _: 7, 0;
    /// Returns or changes the projected texture control.
    pub projected_texture, set_projected_texture: 2;
}
impl WowBatchFlags {
    /// Retains known and producer-defined behavior bits.
    pub const fn from_bits_retain(bits: u8) -> Self {
        Self(bits)
    }
}
