//! Native particles wire codes and flags.
use bitfield::bitfield;

/// Native ParticleBlend code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ParticleBlend {
    #[default]
    /// Draws without blending.
    Opaque,
    /// Discards fragments below the alpha-test threshold.
    AlphaKey,
    /// Blends particle color using source alpha.
    Alpha,
    /// Adds particle color without multiplying by source alpha.
    NoAlphaAdd,
    /// Adds particle color weighted by source alpha.
    Add,
    /// Multiplies the destination by particle color.
    Modulate,
    /// Doubles the modulation of destination and particle color.
    Modulate2x,
    /// Uses premultiplied source color with inverse-alpha destination blending.
    BlendAdd,
    /// Unrecognized native code.
    Unknown(u16),
}

/// Native PackedParticleBlend code; unknown values retain their exact wire representation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PackedParticleBlend {
    #[default]
    /// Draws without blending.
    Opaque,
    /// Discards fragments below the alpha-test threshold.
    AlphaKey,
    /// Blends particle color using source alpha.
    Alpha,
    /// Adds particle color without multiplying by source alpha.
    NoAlphaAdd,
    /// Adds particle color weighted by source alpha.
    Add,
    /// Multiplies the destination by particle color.
    Modulate,
    /// Doubles the modulation of destination and particle color.
    Modulate2x,
    /// Uses premultiplied source color with inverse-alpha destination blending.
    BlendAdd,
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

/// Particle geometry selection, independent of legacy bytes or modern style bits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ParticleHeadTail {
    #[default]
    /// Draws the particle head.
    Head,
    /// Draws the particle trail.
    Tail,
    /// Draws both the head and trail.
    Both,
    /// Simulates particles without drawing head or tail geometry.
    Disabled,
    /// Unrecognized legacy head/tail code.
    Unknown(u8),
}

bitfield! {
    /// Native ParticleFlags; unknown bits are retained.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct ParticleFlags(u32);
    /// Exact stored bits, including unresolved flags.
    pub bits, _: 31, 0;
    /// Selects the unlit particle shader rather than the lit shader.
    pub disable_lighting, set_disable_lighting: 0;
    /// Aligns particle heads with their projected velocity.
    pub velocity_oriented, set_velocity_oriented: 2;
    /// Disables fog for particle rendering.
    pub unfogged, set_unfogged: 3;
    /// Keeps particle simulation local to the moving emitter frame.
    pub attached, set_attached: 4;
    /// Multiplies rendered particle size by the inherited bone scale.
    pub inherit_bone_scale, set_inherit_bone_scale: 5;
    /// Adds inherited emitter velocity to new particles.
    pub inherit_velocity, set_inherit_velocity: 6;
    /// Discards sphere particles moving away from the emitter center.
    pub implosion_filter, set_implosion_filter: 7;
    /// Directs sphere-generator initial velocity along its positive Z axis.
    pub sphere_up, set_sphere_up: 8;
    /// Negates particle spin for one random seed parity.
    pub negate_spin_random, set_negate_spin_random: 9;
    /// Limits the tail time interval to the particle age.
    pub clamp_tail_to_age, set_clamp_tail_to_age: 10;
    /// Aligns the particle quad with the emitter XY plane.
    pub align_xy, set_align_xy: 12;
    /// Requests particle position projection onto the ground.
    pub ground_clamp, set_ground_clamp: 13;
    /// Applies the emitter position delta to existing particles.
    pub follow_position, set_follow_position: 14;
    /// Selects authored keyed burst emission behavior.
    pub squirt, set_squirt: 15;
    /// Selects a random particle texture frame.
    pub random_texture, set_random_texture: 16;
    /// Enables particle head geometry.
    pub head, set_head: 17;
    /// Enables particle tail geometry.
    pub tail, set_tail: 18;
    /// Uses independent random X and Y scale variation.
    pub independent_scale, set_independent_scale: 19;
    /// Selects the scene-refraction particle programme when multi-texture is inactive.
    pub refraction, set_refraction: 20;
    /// Randomizes the starting flipbook frame.
    pub random_flipbook, set_random_flipbook: 21;
    /// Stores gravity keys as signed direction components and a signed magnitude.
    pub packed_gravity, set_packed_gravity: 23;
    /// Selects the bone variant of the bone/joint generator.
    pub bone_generator, set_bone_generator: 24;
    /// Retains the native distance-throttling bypass request.
    pub no_throttle, set_no_throttle: 26;
    /// Offsets the particle head along the spin basis.
    pub offset_head_by_spin, set_offset_head_by_spin: 27;
    /// Uses three packed five-bit texture identifiers and multi-texture composition.
    pub multi_texture, set_multi_texture: 28;
    /// Selects fourfold alpha gain in the multi-texture programme.
    pub multi_texture_modulate4x, set_multi_texture_modulate4x: 29;
    /// Selects three color inputs in the multi-texture programme.
    pub multi_texture_three_colors, set_multi_texture_three_colors: 30;
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
    /// Interprets a native particle blend code.
    pub const fn from_raw(value: u16) -> Self {
        match value {
            0 => Self::Opaque,
            1 => Self::AlphaKey,
            2 => Self::Alpha,
            3 => Self::NoAlphaAdd,
            4 => Self::Add,
            5 => Self::Modulate,
            6 => Self::Modulate2x,
            7 => Self::BlendAdd,
            value => Self::Unknown(value),
        }
    }
    /// Returns the native particle blend code.
    pub const fn raw(self) -> u16 {
        match self {
            Self::Opaque => 0,
            Self::AlphaKey => 1,
            Self::Alpha => 2,
            Self::NoAlphaAdd => 3,
            Self::Add => 4,
            Self::Modulate => 5,
            Self::Modulate2x => 6,
            Self::BlendAdd => 7,
            Self::Unknown(value) => value,
        }
    }
}

impl PackedParticleBlend {
    /// Interprets a native particle blend code.
    pub const fn from_raw(value: u8) -> Self {
        match value {
            0 => Self::Opaque,
            1 => Self::AlphaKey,
            2 => Self::Alpha,
            3 => Self::NoAlphaAdd,
            4 => Self::Add,
            5 => Self::Modulate,
            6 => Self::Modulate2x,
            7 => Self::BlendAdd,
            value => Self::Unknown(value),
        }
    }
    /// Returns the native particle blend code.
    pub const fn raw(self) -> u8 {
        match self {
            Self::Opaque => 0,
            Self::AlphaKey => 1,
            Self::Alpha => 2,
            Self::NoAlphaAdd => 3,
            Self::Add => 4,
            Self::Modulate => 5,
            Self::Modulate2x => 6,
            Self::BlendAdd => 7,
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
    /// Interprets the legacy emitter's explicit head/tail byte.
    pub const fn from_legacy_code(value: u8) -> Self {
        match value {
            0 => Self::Head,
            1 => Self::Tail,
            2 => Self::Both,
            value => Self::Unknown(value),
        }
    }
    /// Returns the legacy head/tail byte when that representation admits this selection.
    pub const fn legacy_code(self) -> Option<u8> {
        match self {
            Self::Head => Some(0),
            Self::Tail => Some(1),
            Self::Both => Some(2),
            Self::Disabled => None,
            Self::Unknown(value) => Some(value),
        }
    }
}
