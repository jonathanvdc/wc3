//! Native materials wire codes and flags.
use super::*;

/// Native BlendMode code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u16)]
pub enum BlendMode {
    #[default]
    /// Writes the source color without blending.
    #[m2(value = 0)]
    Opaque,
    /// Uses an alpha-test cutout.
    #[m2(value = 1)]
    AlphaKey,
    /// Blends using source alpha.
    #[m2(value = 2)]
    Alpha,
    /// Adds source color without source-alpha modulation.
    #[m2(value = 3)]
    NoAlphaAdd,
    /// Adds source color weighted by source alpha.
    #[m2(value = 4)]
    Add,
    /// Multiplies the destination color by the source color.
    #[m2(value = 5)]
    Modulate,
    /// Doubles the modulated color contribution.
    #[m2(value = 6)]
    Modulate2x,
    /// Combines alpha blending and an additive contribution.
    #[m2(value = 7)]
    BlendAdd,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u16),
}

/// Native TextureSource code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Value,
    m2::Validate,
    m2::VisitArrays,
)]
#[m2(value = u32)]
pub enum TextureSource {
    #[default]
    /// Loads the texture through its filename or file-data ID.
    #[m2(value = 0)]
    File,
    /// Uses the runtime body-and-clothing skin replacement.
    #[m2(value = 1)]
    Skin,
    /// Uses the runtime object-skin replacement.
    #[m2(value = 2)]
    ObjectSkin,
    /// Uses the weapon-blade replacement slot.
    #[m2(value = 3)]
    WeaponBlade,
    /// Uses the weapon-handle replacement slot.
    #[m2(value = 4)]
    WeaponHandle,
    /// Uses the legacy environment-map replacement.
    #[m2(value = 5)]
    Environment,
    /// Uses the character-hair replacement.
    #[m2(value = 6)]
    CharacterHair,
    /// Uses the legacy character-facial-hair replacement.
    #[m2(value = 7)]
    CharacterFacialHair,
    /// Uses the extra body-skin replacement.
    #[m2(value = 8)]
    SkinExtra,
    /// Inventory-art skin.
    #[m2(value = 9)]
    UiSkin,
    /// Legacy tauren mane replacement.
    #[m2(value = 10)]
    TaurenMane,
    /// First creature or game-object skin.
    #[m2(value = 11)]
    MonsterSkin1,
    /// Second creature or game-object skin.
    #[m2(value = 12)]
    MonsterSkin2,
    /// Third creature or game-object skin.
    #[m2(value = 13)]
    MonsterSkin3,
    /// Inventory item icon.
    #[m2(value = 14)]
    ItemIcon,
    /// Guild background color replacement.
    #[m2(value = 15)]
    GuildBackgroundColor,
    /// Guild emblem color replacement.
    #[m2(value = 16)]
    GuildEmblemColor,
    /// Guild border color replacement.
    #[m2(value = 17)]
    GuildBorderColor,
    /// Guild emblem replacement.
    #[m2(value = 18)]
    GuildEmblem,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u32),
}

bitfield! {
    /// Native MaterialFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout, m2::Validate, m2::VisitArrays)]
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
    /// Native TextureFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout, m2::Validate, m2::VisitArrays)]
    pub struct TextureFlags(u32);
    /// Exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the wrap u bit.
    pub wrap_u, set_wrap_u: 0;
    /// Returns or changes the wrap v bit.
    pub wrap_v, set_wrap_v: 1;
}
impl TextureFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for TextureFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<TextureFlags> for u32 {
    fn from(flags: TextureFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native BatchFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout, m2::Validate, m2::VisitArrays)]
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
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout, m2::Validate, m2::VisitArrays)]
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
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout, m2::Validate, m2::VisitArrays)]
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
wire_ule!(BlendMode, u16, raw, from_raw);
wire_ule!(TextureSource, u32, raw, from_raw);
wire_ule!(MaterialFlags, u16, bits, from_bits_retain);
wire_ule!(TextureFlags, u32, bits, from_bits_retain);
wire_ule!(BatchFlags, u8, bits, from_bits_retain);
wire_ule!(ShadowBatchFlags, u8, bits, from_bits_retain);
wire_ule!(ShadowBatchExtraFlags, u8, bits, from_bits_retain);
