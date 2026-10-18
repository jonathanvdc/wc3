//! Native materials wire codes and flags.
use crate::model::materials::state::BlendOperations;
use crate::model::{Link, ValidateLinks};
use bitfield::bitfield;

/// Native BlendMode code; unknown values retain their exact wire representation.
pub type BlendMode = BlendOperations<ModelBlend, (), (), u16>;

/// Native TextureSource code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextureSource {
    #[default]
    /// Loads the texture through its filename or file-data ID.
    File,
    /// Uses the runtime body-and-clothing skin replacement.
    Skin,
    /// Uses the runtime object-skin replacement.
    ObjectSkin,
    /// Uses the weapon-blade replacement slot.
    WeaponBlade,
    /// Uses the weapon-handle replacement slot.
    WeaponHandle,
    /// Uses the legacy environment-map replacement.
    Environment,
    /// Uses the character-hair replacement.
    CharacterHair,
    /// Uses the legacy character-facial-hair replacement.
    CharacterFacialHair,
    /// Uses the extra body-skin replacement.
    SkinExtra,
    /// Inventory-art skin.
    UiSkin,
    /// Legacy tauren mane replacement.
    TaurenMane,
    /// First creature or game-object skin.
    MonsterSkin1,
    /// Second creature or game-object skin.
    MonsterSkin2,
    /// Third creature or game-object skin.
    MonsterSkin3,
    /// Inventory item icon.
    ItemIcon,
    /// Guild background color replacement.
    GuildBackgroundColor,
    /// Guild emblem color replacement.
    GuildEmblemColor,
    /// Guild border color replacement.
    GuildBorderColor,
    /// Guild emblem replacement.
    GuildEmblem,
    /// Unrecognized native code.
    Unknown(u32),
}

bitfield! {
    /// Native MaterialFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct MaterialFlags(u16);
    /// Exact stored bits.
    pub bits, _: 15, 0;
    /// Returns or changes the unlit bit.
    pub unlit, set_unlit: 0;
    /// Returns or changes the unfogged bit.
    pub unfogged, set_unfogged: 1;
    /// Returns or changes the two sided bit.
    pub two_sided, set_two_sided: 2;
    /// Returns or changes the no depth test bit.
    pub no_depth_test, set_no_depth_test: 3;
    /// Returns or changes the no depth write bit.
    pub no_depth_write, set_no_depth_write: 4;
    /// Returns or changes the shadow batch 1 bit.
    pub shadow_batch_1, set_shadow_batch_1: 6;
    /// Returns or changes the shadow batch 2 bit.
    pub shadow_batch_2, set_shadow_batch_2: 7;
    /// Returns or changes the no alpha composite bit.
    pub no_alpha_composite, set_no_alpha_composite: 11;
}
impl MaterialFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<u16> for MaterialFlags {
    fn from(bits: u16) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<MaterialFlags> for u16 {
    fn from(flags: MaterialFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native BatchFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct BatchFlags(u8);
    /// Exact stored bits.
    pub bits, _: 7, 0;
    /// Returns or changes the projected texture bit.
    pub projected_texture, set_projected_texture: 2;
}
impl BatchFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u8) -> Self {
        Self(bits)
    }
}
impl From<u8> for BatchFlags {
    fn from(bits: u8) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<BatchFlags> for u8 {
    fn from(flags: BatchFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native ShadowBatchFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct ShadowBatchFlags(u8);
    /// Exact stored bits.
    pub bits, _: 7, 0;
}
impl ShadowBatchFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u8) -> Self {
        Self(bits)
    }
}
impl From<u8> for ShadowBatchFlags {
    fn from(bits: u8) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<ShadowBatchFlags> for u8 {
    fn from(flags: ShadowBatchFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native ShadowBatchExtraFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct ShadowBatchExtraFlags(u8);
    /// Exact stored bits.
    pub bits, _: 7, 0;
    /// Returns or changes the edge fade bit.
    pub edge_fade, set_edge_fade: 3;
}
impl ShadowBatchExtraFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u8) -> Self {
        Self(bits)
    }
}
impl From<u8> for ShadowBatchExtraFlags {
    fn from(bits: u8) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<ShadowBatchExtraFlags> for u8 {
    fn from(flags: ShadowBatchExtraFlags) -> Self {
        flags.bits()
    }
}

impl BlendMode {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Opaque,
            1 => Self::AlphaKey(()),
            2 => Self::Alpha,
            3 => Self::Additive,
            4 => Self::AdditiveAlpha(()),
            5 => Self::Modulate,
            6 => Self::Modulate2x,
            7 => Self::Extension(ModelBlend::BlendAdd),
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Opaque => 0,
            Self::AlphaKey(()) => 1,
            Self::Alpha => 2,
            Self::Additive => 3,
            Self::AdditiveAlpha(()) => 4,
            Self::Modulate => 5,
            Self::Modulate2x => 6,
            Self::Extension(ModelBlend::BlendAdd) => 7,
            Self::Unknown(value) => value,
        }
    }
}

impl TextureSource {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u32) -> Self {
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
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u32 {
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

/// Additional model-shader color operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelBlend {
    /// Combines alpha blending and an additive contribution.
    BlendAdd,
}
impl ValidateLinks for ModelBlend {
    fn visit_links(&self, _: &mut dyn FnMut(Link)) {}
}
