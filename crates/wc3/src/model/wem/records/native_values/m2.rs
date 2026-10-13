//! Native m2 selectors and flags preserve their encoded widths.
use crate::model::wem;
use bitfield::bitfield;

/// Native M2BlendingMode16 choices, retaining unrecognized codes.
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
#[wem(value=u16)]
pub enum M2BlendingMode16 {
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
    /// Unrecognized wire code.
    #[wem(unknown)]
    Unknown(u16),
}

/// Native M2TextureSource8 choices, retaining unrecognized codes.
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
#[wem(value=u8)]
pub enum M2TextureSource8 {
    #[default]
    /// File.
    #[wem(value = 0)]
    File,
    /// Skin.
    #[wem(value = 1)]
    Skin,
    /// ObjectSkin.
    #[wem(value = 2)]
    ObjectSkin,
    /// WeaponBlade.
    #[wem(value = 3)]
    WeaponBlade,
    /// WeaponHandle.
    #[wem(value = 4)]
    WeaponHandle,
    /// Environment.
    #[wem(value = 5)]
    Environment,
    /// Hair.
    #[wem(value = 6)]
    CharacterHair,
    /// FacialHair.
    #[wem(value = 7)]
    CharacterFacialHair,
    /// SkinExtra.
    #[wem(value = 8)]
    SkinExtra,
    /// Inventory-art skin.
    #[wem(value = 9)]
    UiSkin,
    /// Legacy tauren mane replacement.
    #[wem(value = 10)]
    TaurenMane,
    /// First creature or game-object skin.
    #[wem(value = 11)]
    MonsterSkin1,
    /// Second creature or game-object skin.
    #[wem(value = 12)]
    MonsterSkin2,
    /// Third creature or game-object skin.
    #[wem(value = 13)]
    MonsterSkin3,
    /// Inventory item icon.
    #[wem(value = 14)]
    ItemIcon,
    /// Guild background color replacement.
    #[wem(value = 15)]
    GuildBackgroundColor,
    /// Guild emblem color replacement.
    #[wem(value = 16)]
    GuildEmblemColor,
    /// Guild border color replacement.
    #[wem(value = 17)]
    GuildBorderColor,
    /// Guild emblem replacement.
    #[wem(value = 18)]
    GuildEmblem,
    /// Unrecognized wire code.
    #[wem(unknown)]
    Unknown(u8),
}

bitfield! {
    /// Native M2RenderFlags16 bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M2RenderFlags16(u16);
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
impl M2RenderFlags16 {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u16> for M2RenderFlags16 {
    fn from(bits: u16) -> Self {
        Self(bits)
    }
}
impl From<M2RenderFlags16> for u16 {
    fn from(flags: M2RenderFlags16) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native M2BatchFlags8 bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M2BatchFlags8(u8);
    /// Returns the exact stored bits.
    pub bits, _: 7, 0;
    /// Returns or changes the projected texture control.
    pub projected_texture, set_projected_texture: 2;
}
impl M2BatchFlags8 {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u8) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u8> for M2BatchFlags8 {
    fn from(bits: u8) -> Self {
        Self(bits)
    }
}
impl From<M2BatchFlags8> for u8 {
    fn from(flags: M2BatchFlags8) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native M2ParticleFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct M2ParticleFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the packed gravity control.
    pub packed_gravity, set_packed_gravity: 23;
    /// Returns or changes the multi texture control.
    pub multi_texture, set_multi_texture: 28;
    /// Returns or changes the lit by light control.
    pub lit_by_light, set_lit_by_light: 0;
    /// Returns or changes the billboard control.
    pub billboard, set_billboard: 2;
    /// Returns or changes the travel up world space control.
    pub travel_up_world_space, set_travel_up_world_space: 3;
    /// Returns or changes the no trail control.
    pub no_trail, set_no_trail: 4;
    /// Returns or changes the unlit control.
    pub unlit, set_unlit: 5;
    /// Returns or changes the burst multi control.
    pub burst_multi, set_burst_multi: 6;
    /// Returns or changes the model space control.
    pub model_space, set_model_space: 7;
    /// Returns or changes the random spawn control.
    pub random_spawn, set_random_spawn: 9;
    /// Returns or changes the pinned control.
    pub pinned, set_pinned: 10;
    /// Returns or changes the align xy control.
    pub align_xy, set_align_xy: 12;
    /// Returns or changes the ground clamp control.
    pub ground_clamp, set_ground_clamp: 13;
    /// Returns or changes the random texture control.
    pub random_texture, set_random_texture: 16;
    /// Returns or changes the outward control.
    pub outward, set_outward: 17;
    /// Returns or changes the inward control.
    pub inward, set_inward: 18;
    /// Returns or changes the independent scale control.
    pub independent_scale, set_independent_scale: 19;
    /// Returns or changes the random flipbook control.
    pub random_flipbook, set_random_flipbook: 21;
    /// Returns or changes the bone generator control.
    pub bone_generator, set_bone_generator: 24;
    /// Returns or changes the no throttle control.
    pub no_throttle, set_no_throttle: 26;
}
impl M2ParticleFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u32> for M2ParticleFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<M2ParticleFlags> for u32 {
    fn from(flags: M2ParticleFlags) -> Self {
        flags.bits()
    }
}

/// Native M2TextureSource32 choices, retaining unrecognized codes.
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
#[wem(value=u32)]
pub enum M2TextureSource32 {
    #[default]
    /// File.
    #[wem(value = 0)]
    File,
    /// Skin.
    #[wem(value = 1)]
    Skin,
    /// ObjectSkin.
    #[wem(value = 2)]
    ObjectSkin,
    /// WeaponBlade.
    #[wem(value = 3)]
    WeaponBlade,
    /// WeaponHandle.
    #[wem(value = 4)]
    WeaponHandle,
    /// Environment.
    #[wem(value = 5)]
    Environment,
    /// Hair.
    #[wem(value = 6)]
    CharacterHair,
    /// FacialHair.
    #[wem(value = 7)]
    CharacterFacialHair,
    /// SkinExtra.
    #[wem(value = 8)]
    SkinExtra,
    /// Inventory-art skin.
    #[wem(value = 9)]
    UiSkin,
    /// Legacy tauren mane replacement.
    #[wem(value = 10)]
    TaurenMane,
    /// First creature or game-object skin.
    #[wem(value = 11)]
    MonsterSkin1,
    /// Second creature or game-object skin.
    #[wem(value = 12)]
    MonsterSkin2,
    /// Third creature or game-object skin.
    #[wem(value = 13)]
    MonsterSkin3,
    /// Inventory item icon.
    #[wem(value = 14)]
    ItemIcon,
    /// Guild background color replacement.
    #[wem(value = 15)]
    GuildBackgroundColor,
    /// Guild emblem color replacement.
    #[wem(value = 16)]
    GuildEmblemColor,
    /// Guild border color replacement.
    #[wem(value = 17)]
    GuildBorderColor,
    /// Guild emblem replacement.
    #[wem(value = 18)]
    GuildEmblem,
    /// Unrecognized wire code.
    #[wem(unknown)]
    Unknown(u32),
}

/// M2 emitter geometry selector.
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
#[wem(value=u32)]
pub enum M2EmitterShape {
    /// Emission across a plane.
    #[wem(value = 1)]
    Plane,
    /// Emission from a sphere.
    #[wem(value = 2)]
    Sphere,
    /// Emission along a spline.
    #[wem(value = 3)]
    Spline,
    /// Emission attached to a bone.
    #[wem(value = 4)]
    Bone,
    /// Unrecognized emitter selector.
    #[wem(unknown)]
    Unknown(u32),
}
impl Default for M2EmitterShape {
    fn default() -> Self {
        Self::Unknown(0)
    }
}

/// M2 particle blending codes, independent of material blending codes.
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
#[wem(value=u32)]
pub enum M2ParticleBlend32 {
    /// Draws opaque particles.
    #[default]
    #[wem(value = 0)]
    Opaque,
    /// Blends using particle alpha.
    #[wem(value = 1)]
    Alpha,
    /// Adds particle color.
    #[wem(value = 2)]
    Add,
    /// Tests particle alpha.
    #[wem(value = 3)]
    AlphaKey,
    /// Adds color with an alpha test.
    #[wem(value = 4)]
    AddAlphaKey,
    /// Unrecognized particle blend code.
    #[wem(unknown)]
    Unknown(u32),
}
