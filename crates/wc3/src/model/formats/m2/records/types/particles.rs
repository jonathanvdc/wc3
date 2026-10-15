//! Native particles wire codes and flags.
use super::*;

/// Native ParticleBlend code; unknown values retain their exact wire representation.
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
pub enum ParticleBlend {
    #[default]
    /// Draws without blending.
    #[m2(value = 0)]
    Opaque,
    /// Blends particles using source alpha.
    #[m2(value = 1)]
    Alpha,
    /// Adds the particle color to the destination.
    #[m2(value = 2)]
    Add,
    /// Cuts out pixels below the alpha-test threshold.
    #[m2(value = 3)]
    AlphaKey,
    /// Combines additive drawing with an alpha-test cutout.
    #[m2(value = 4)]
    AddAlphaKey,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u16),
}

/// Native PackedParticleBlend code; unknown values retain their exact wire representation.
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
#[m2(value = u8)]
pub enum PackedParticleBlend {
    #[default]
    /// Draws without blending.
    #[m2(value = 0)]
    Opaque,
    /// Blends particles using source alpha.
    #[m2(value = 1)]
    Alpha,
    /// Adds the particle color to the destination.
    #[m2(value = 2)]
    Add,
    /// Cuts out pixels below the alpha-test threshold.
    #[m2(value = 3)]
    AlphaKey,
    /// Combines additive drawing with an alpha-test cutout.
    #[m2(value = 4)]
    AddAlphaKey,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u8),
}

/// Native ParticleGenerator code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
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
pub enum ParticleGenerator {
    /// Spawns particles across a planar emitter area.
    #[m2(value = 1)]
    Plane,
    /// Spawns particles using a spherical emitter.
    #[m2(value = 2)]
    Sphere,
    /// Spawns particles using a spline generator.
    #[m2(value = 3)]
    Spline,
    /// Spawns particles using a bone generator.
    #[m2(value = 4)]
    Bone,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u16),
}

/// Native PackedParticleGenerator code; unknown values retain their exact wire representation.
#[derive(
    Clone,
    Copy,
    Debug,
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
#[m2(value = u8)]
pub enum PackedParticleGenerator {
    /// Spawns particles across a planar emitter area.
    #[m2(value = 1)]
    Plane,
    /// Spawns particles using a spherical emitter.
    #[m2(value = 2)]
    Sphere,
    /// Spawns particles using a spline generator.
    #[m2(value = 3)]
    Spline,
    /// Spawns particles using a bone generator.
    #[m2(value = 4)]
    Bone,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u8),
}

/// Native ParticleHeadTail code; unknown values retain their exact wire representation.
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
#[m2(value = u8)]
pub enum ParticleHeadTail {
    #[default]
    /// Draws the particle head.
    #[m2(value = 0)]
    Head,
    /// Draws the particle trail.
    #[m2(value = 1)]
    Tail,
    /// Draws both the head and trail.
    #[m2(value = 2)]
    Both,
    /// Unrecognized native code.
    #[m2(unknown)]
    Unknown(u8),
}

bitfield! {
    /// Native ParticleFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, m2::Read, m2::Write, m2::RecordLayout, m2::Validate, m2::VisitArrays)]
    pub struct ParticleFlags(u32);
    /// Exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the packed gravity bit.
    pub packed_gravity, set_packed_gravity: 23;
    /// Uses the packed three-texture identifier representation.
    pub multi_texture, set_multi_texture: 28;
    /// Returns or changes the lit by light bit.
    pub lit_by_light, set_lit_by_light: 0;
    /// Returns or changes the billboard bit.
    pub billboard, set_billboard: 2;
    /// Returns or changes the travel up world space bit.
    pub travel_up_world_space, set_travel_up_world_space: 3;
    /// Returns or changes the no trail bit.
    pub no_trail, set_no_trail: 4;
    /// Returns or changes the unlit bit.
    pub unlit, set_unlit: 5;
    /// Returns or changes the burst multi bit.
    pub burst_multi, set_burst_multi: 6;
    /// Returns or changes the model space bit.
    pub model_space, set_model_space: 7;
    /// Returns or changes the random spawn bit.
    pub random_spawn, set_random_spawn: 9;
    /// Returns or changes the pinned bit.
    pub pinned, set_pinned: 10;
    /// Returns or changes the align xy bit.
    pub align_xy, set_align_xy: 12;
    /// Returns or changes the ground clamp bit.
    pub ground_clamp, set_ground_clamp: 13;
    /// Returns or changes the random texture bit.
    pub random_texture, set_random_texture: 16;
    /// Returns or changes the outward bit.
    pub outward, set_outward: 17;
    /// Returns or changes the inward bit.
    pub inward, set_inward: 18;
    /// Returns or changes the independent scale bit.
    pub independent_scale, set_independent_scale: 19;
    /// Returns or changes the random flipbook bit.
    pub random_flipbook, set_random_flipbook: 21;
    /// Returns or changes the bone generator bit.
    pub bone_generator, set_bone_generator: 24;
    /// Returns or changes the no throttle bit.
    pub no_throttle, set_no_throttle: 26;
}
impl ParticleFlags {
    /// Constructs flags without discarding unknown bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<u32> for ParticleFlags {
    fn from(bits: u32) -> Self {
        Self::from_bits_retain(bits)
    }
}
impl From<ParticleFlags> for u32 {
    fn from(flags: ParticleFlags) -> Self {
        flags.bits()
    }
}

impl Default for ParticleGenerator {
    fn default() -> Self {
        Self::Unknown(0)
    }
}

impl Default for PackedParticleGenerator {
    fn default() -> Self {
        Self::Unknown(0)
    }
}
wire_ule!(ParticleBlend, u16, raw, from_raw);
wire_ule!(PackedParticleBlend, u8, raw, from_raw);
wire_ule!(ParticleGenerator, u16, raw, from_raw);
wire_ule!(PackedParticleGenerator, u8, raw, from_raw);
wire_ule!(ParticleHeadTail, u8, raw, from_raw);
wire_ule!(ParticleFlags, u32, bits, from_bits_retain);
