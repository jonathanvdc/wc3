//! Native particles wire codes and flags.
use bitfield::bitfield;

/// Native ParticleBlend code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ParticleBlend {
    #[default]
    /// Draws without blending.
    Opaque,
    /// Blends particles using source alpha.
    Alpha,
    /// Adds the particle color to the destination.
    Add,
    /// Cuts out pixels below the alpha-test threshold.
    AlphaKey,
    /// Combines additive drawing with an alpha-test cutout.
    AddAlphaKey,
    /// Unrecognized native code.
    Unknown(u16),
}

/// Native PackedParticleBlend code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PackedParticleBlend {
    #[default]
    /// Draws without blending.
    Opaque,
    /// Blends particles using source alpha.
    Alpha,
    /// Adds the particle color to the destination.
    Add,
    /// Cuts out pixels below the alpha-test threshold.
    AlphaKey,
    /// Combines additive drawing with an alpha-test cutout.
    AddAlphaKey,
    /// Unrecognized native code.
    Unknown(u8),
}

/// Native ParticleGenerator code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParticleGenerator {
    /// Spawns particles across a planar emitter area.
    Plane,
    /// Spawns particles using a spherical emitter.
    Sphere,
    /// Spawns particles using a spline generator.
    Spline,
    /// Spawns particles using a bone generator.
    Bone,
    /// Unrecognized native code.
    Unknown(u16),
}

/// Native PackedParticleGenerator code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PackedParticleGenerator {
    /// Spawns particles across a planar emitter area.
    Plane,
    /// Spawns particles using a spherical emitter.
    Sphere,
    /// Spawns particles using a spline generator.
    Spline,
    /// Spawns particles using a bone generator.
    Bone,
    /// Unrecognized native code.
    Unknown(u8),
}

/// Native ParticleHeadTail code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ParticleHeadTail {
    #[default]
    /// Draws the particle head.
    Head,
    /// Draws the particle trail.
    Tail,
    /// Draws both the head and trail.
    Both,
    /// Unrecognized native code.
    Unknown(u8),
}

bitfield! {
    /// Native ParticleFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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

impl ParticleBlend {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Opaque,
            1 => Self::Alpha,
            2 => Self::Add,
            3 => Self::AlphaKey,
            4 => Self::AddAlphaKey,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Opaque => 0,
            Self::Alpha => 1,
            Self::Add => 2,
            Self::AlphaKey => 3,
            Self::AddAlphaKey => 4,
            Self::Unknown(value) => value,
        }
    }
}

impl PackedParticleBlend {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Opaque,
            1 => Self::Alpha,
            2 => Self::Add,
            3 => Self::AlphaKey,
            4 => Self::AddAlphaKey,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u8 {
        match self {
            Self::Opaque => 0,
            Self::Alpha => 1,
            Self::Add => 2,
            Self::AlphaKey => 3,
            Self::AddAlphaKey => 4,
            Self::Unknown(value) => value,
        }
    }
}

impl ParticleGenerator {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            1 => Self::Plane,
            2 => Self::Sphere,
            3 => Self::Spline,
            4 => Self::Bone,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Plane => 1,
            Self::Sphere => 2,
            Self::Spline => 3,
            Self::Bone => 4,
            Self::Unknown(value) => value,
        }
    }
}

impl PackedParticleGenerator {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u8) -> Self {
        match value {
            1 => Self::Plane,
            2 => Self::Sphere,
            3 => Self::Spline,
            4 => Self::Bone,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u8 {
        match self {
            Self::Plane => 1,
            Self::Sphere => 2,
            Self::Spline => 3,
            Self::Bone => 4,
            Self::Unknown(value) => value,
        }
    }
}

impl ParticleHeadTail {
    /// Interprets an external format choice without depending on its codec.
    pub const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Head,
            1 => Self::Tail,
            2 => Self::Both,
            value => Self::Unknown(value),
        }
    }
    /// Returns the external choice identifier.
    pub const fn raw(self) -> u8 {
        match self {
            Self::Head => 0,
            Self::Tail => 1,
            Self::Both => 2,
            Self::Unknown(value) => value,
        }
    }
}
