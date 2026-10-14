//! M3 particle selectors and bit masks.
use bitfield::bitfield;

/// M3 EmitterShape selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum EmitterShape {
    /// Selects Point.
    Point,
    /// Selects Plane.
    Plane,
    /// Selects Sphere.
    Sphere,
    /// Selects Box.
    Box,
    /// Selects Cylinder.
    Cylinder,
    /// Selects Disc.
    Disc,
    /// Selects Spline.
    Spline,
    /// Selects Mesh.
    Mesh,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl EmitterShape {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Point => 0,
            Self::Plane => 1,
            Self::Sphere => 2,
            Self::Box => 3,
            Self::Cylinder => 4,
            Self::Disc => 5,
            Self::Spline => 6,
            Self::Mesh => 7,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Point,
            1 => Self::Plane,
            2 => Self::Sphere,
            3 => Self::Box,
            4 => Self::Cylinder,
            5 => Self::Disc,
            6 => Self::Spline,
            7 => Self::Mesh,
            value => Self::Unknown(value),
        }
    }
}

impl Default for EmitterShape {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 ParticleInstance selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ParticleInstance {
    /// Selects Billboard.
    Billboard,
    /// Selects Tail.
    Tail,
    /// Selects FaceTravelDir.
    FaceTravelDir,
    /// Selects FaceWorldDir.
    FaceWorldDir,
    /// Selects SingleAxis.
    SingleAxis,
    /// Selects TerrainOriented.
    TerrainOriented,
    /// Selects TerrainDirOriented.
    TerrainDirOriented,
    /// Selects EmitterOriented.
    EmitterOriented,
    /// Selects PhysicsOriented.
    PhysicsOriented,
    /// Selects Pinned.
    Pinned,
    /// Selects Trail.
    Trail,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl ParticleInstance {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Billboard => 0,
            Self::Tail => 1,
            Self::FaceTravelDir => 2,
            Self::FaceWorldDir => 3,
            Self::SingleAxis => 4,
            Self::TerrainOriented => 5,
            Self::TerrainDirOriented => 6,
            Self::EmitterOriented => 7,
            Self::PhysicsOriented => 8,
            Self::Pinned => 9,
            Self::Trail => 10,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Billboard,
            1 => Self::Tail,
            2 => Self::FaceTravelDir,
            3 => Self::FaceWorldDir,
            4 => Self::SingleAxis,
            5 => Self::TerrainOriented,
            6 => Self::TerrainDirOriented,
            7 => Self::EmitterOriented,
            8 => Self::PhysicsOriented,
            9 => Self::Pinned,
            10 => Self::Trail,
            value => Self::Unknown(value),
        }
    }
}

impl Default for ParticleInstance {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 Smoothing selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Smoothing {
    /// Selects Linear.
    Linear,
    /// Selects LinearSmooth.
    LinearSmooth,
    /// Selects Bezier.
    Bezier,
    /// Selects LinearWithHold.
    LinearWithHold,
    /// Selects BezierWithHold.
    BezierWithHold,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl Smoothing {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Linear => 0,
            Self::LinearSmooth => 1,
            Self::Bezier => 2,
            Self::LinearWithHold => 3,
            Self::BezierWithHold => 4,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Linear,
            1 => Self::LinearSmooth,
            2 => Self::Bezier,
            3 => Self::LinearWithHold,
            4 => Self::BezierWithHold,
            value => Self::Unknown(value),
        }
    }
}

impl Default for Smoothing {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 Variation selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Variation {
    /// Selects None.
    None,
    /// Selects Sine.
    Sine,
    /// Selects Cosine.
    Cosine,
    /// Selects Saw.
    Saw,
    /// Selects Square.
    Square,
    /// Selects RandomNoise.
    RandomNoise,
    /// Selects ContinuousNoise.
    ContinuousNoise,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl Variation {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Sine => 1,
            Self::Cosine => 2,
            Self::Saw => 3,
            Self::Square => 4,
            Self::RandomNoise => 5,
            Self::ContinuousNoise => 6,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Sine,
            2 => Self::Cosine,
            3 => Self::Saw,
            4 => Self::Square,
            5 => Self::RandomNoise,
            6 => Self::ContinuousNoise,
            value => Self::Unknown(value),
        }
    }
}

impl Default for Variation {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// M3 VelocityKind selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum VelocityKind {
    /// Selects Constant.
    Constant,
    /// Selects Radial.
    Radial,
    /// Selects ZAxis.
    ZAxis,
    /// Selects Random.
    Random,
    /// Selects MeshNormal.
    MeshNormal,
    /// An unrecognized native code, retained without reinterpretation.
    Unknown(u32),
}
impl VelocityKind {
    /// Returns the stored selector code.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Constant => 0,
            Self::Radial => 1,
            Self::ZAxis => 2,
            Self::Random => 3,
            Self::MeshNormal => 4,
            Self::Unknown(value) => value,
        }
    }
    /// Selects a known meaning, preserving any unrecognized code.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Constant,
            1 => Self::Radial,
            2 => Self::ZAxis,
            3 => Self::Random,
            4 => Self::MeshNormal,
            value => Self::Unknown(value),
        }
    }
}

impl Default for VelocityKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// M3 ParticleFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct ParticleFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the sort distance control.
    pub sort_distance, set_sort_distance: 0;
    /// Returns or changes the collide terrain control.
    pub collide_terrain, set_collide_terrain: 1;
    /// Returns or changes the collide objects control.
    pub collide_objects, set_collide_objects: 2;
    /// Returns or changes the collide emit control.
    pub collide_emit, set_collide_emit: 3;
    /// Returns or changes the emit shape cutout control.
    pub emit_shape_cutout, set_emit_shape_cutout: 4;
    /// Returns or changes the inherit emit params control.
    pub inherit_emit_params, set_inherit_emit_params: 5;
    /// Returns or changes the inherit parent velocity control.
    pub inherit_parent_velocity, set_inherit_parent_velocity: 6;
    /// Returns or changes the sort height control.
    pub sort_height, set_sort_height: 7;
    /// Returns or changes the sort reverse control.
    pub sort_reverse, set_sort_reverse: 8;
    /// Returns or changes the old rotation smooth control.
    pub old_rotation_smooth, set_old_rotation_smooth: 9;
    /// Returns or changes the old rotation bezier control.
    pub old_rotation_bezier, set_old_rotation_bezier: 10;
    /// Returns or changes the old size smooth control.
    pub old_size_smooth, set_old_size_smooth: 11;
    /// Returns or changes the old size bezier control.
    pub old_size_bezier, set_old_size_bezier: 12;
    /// Returns or changes the old color smooth control.
    pub old_color_smooth, set_old_color_smooth: 13;
    /// Returns or changes the old color bezier control.
    pub old_color_bezier, set_old_color_bezier: 14;
    /// Returns or changes the lit parts control.
    pub lit_parts, set_lit_parts: 15;
    /// Returns or changes the random uv flipbook start control.
    pub random_uv_flipbook_start, set_random_uv_flipbook_start: 16;
    /// Returns or changes the multiply gravity control.
    pub multiply_gravity, set_multiply_gravity: 17;
    /// Returns or changes the tail clamp control.
    pub tail_clamp, set_tail_clamp: 18;
    /// Returns or changes the use trailing particle control.
    pub use_trailing_particle, set_use_trailing_particle: 19;
    /// Returns or changes the tail fix control.
    pub tail_fix, set_tail_fix: 20;
    /// Returns or changes the vertex alpha control.
    pub vertex_alpha, set_vertex_alpha: 21;
    /// Returns or changes the model particles control.
    pub model_particles, set_model_particles: 22;
    /// Returns or changes the swap yz on model particles control.
    pub swap_yz_on_model_particles, set_swap_yz_on_model_particles: 23;
    /// Returns or changes the scale time parent control.
    pub scale_time_parent, set_scale_time_parent: 24;
    /// Returns or changes the local time control.
    pub local_time, set_local_time: 25;
    /// Returns or changes the simulate init control.
    pub simulate_init, set_simulate_init: 26;
    /// Returns or changes the copy control.
    pub copy, set_copy: 27;
}
impl From<u32> for ParticleFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ParticleFlags> for u32 {
    fn from(value: ParticleFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// M3 ParticleAdditionalFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct ParticleAdditionalFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the emit speed randomize control.
    pub emit_speed_randomize, set_emit_speed_randomize: 0;
    /// Returns or changes the lifespan randomize control.
    pub lifespan_randomize, set_lifespan_randomize: 1;
    /// Returns or changes the mass randomize control.
    pub mass_randomize, set_mass_randomize: 2;
    /// Returns or changes the world space control.
    pub world_space, set_world_space: 3;
}
impl From<u32> for ParticleAdditionalFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ParticleAdditionalFlags> for u32 {
    fn from(value: ParticleAdditionalFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// M3 ParticleRotationFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct ParticleRotationFlags(u32);
    /// Returns the complete stored bit mask.
    pub bits, _: 31, 0;
    /// Returns or changes the relative control.
    pub relative, set_relative: 1;
    /// Returns or changes the always set control.
    pub always_set, set_always_set: 2;
}
impl From<u32> for ParticleRotationFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<ParticleRotationFlags> for u32 {
    fn from(value: ParticleRotationFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// M3 LocalForces bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct LocalForces(u16);
    /// Returns the complete stored bit mask.
    pub bits, _: 15, 0;
    /// Returns or changes the channel 0 control.
    pub channel_0, set_channel_0: 0;
    /// Returns or changes the channel 1 control.
    pub channel_1, set_channel_1: 1;
    /// Returns or changes the channel 2 control.
    pub channel_2, set_channel_2: 2;
    /// Returns or changes the channel 3 control.
    pub channel_3, set_channel_3: 3;
    /// Returns or changes the channel 4 control.
    pub channel_4, set_channel_4: 4;
    /// Returns or changes the channel 5 control.
    pub channel_5, set_channel_5: 5;
    /// Returns or changes the channel 6 control.
    pub channel_6, set_channel_6: 6;
    /// Returns or changes the channel 7 control.
    pub channel_7, set_channel_7: 7;
    /// Returns or changes the channel 8 control.
    pub channel_8, set_channel_8: 8;
    /// Returns or changes the channel 9 control.
    pub channel_9, set_channel_9: 9;
    /// Returns or changes the channel 10 control.
    pub channel_10, set_channel_10: 10;
    /// Returns or changes the channel 11 control.
    pub channel_11, set_channel_11: 11;
    /// Returns or changes the channel 12 control.
    pub channel_12, set_channel_12: 12;
    /// Returns or changes the channel 13 control.
    pub channel_13, set_channel_13: 13;
    /// Returns or changes the channel 14 control.
    pub channel_14, set_channel_14: 14;
    /// Returns or changes the channel 15 control.
    pub channel_15, set_channel_15: 15;
}
impl From<u16> for LocalForces {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<LocalForces> for u16 {
    fn from(value: LocalForces) -> Self {
        value.bits()
    }
}

bitfield! {
    /// M3 WorldForces bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash,)]
    pub struct WorldForces(u16);
    /// Returns the complete stored bit mask.
    pub bits, _: 15, 0;
    /// Returns or changes the channel 0 control.
    pub channel_0, set_channel_0: 0;
    /// Returns or changes the channel 1 control.
    pub channel_1, set_channel_1: 1;
    /// Returns or changes the channel 2 control.
    pub channel_2, set_channel_2: 2;
    /// Returns or changes the channel 3 control.
    pub channel_3, set_channel_3: 3;
    /// Returns or changes the channel 4 control.
    pub channel_4, set_channel_4: 4;
    /// Returns or changes the channel 5 control.
    pub channel_5, set_channel_5: 5;
    /// Returns or changes the channel 6 control.
    pub channel_6, set_channel_6: 6;
    /// Returns or changes the channel 7 control.
    pub channel_7, set_channel_7: 7;
    /// Returns or changes the channel 8 control.
    pub channel_8, set_channel_8: 8;
    /// Returns or changes the channel 9 control.
    pub channel_9, set_channel_9: 9;
    /// Returns or changes the channel 10 control.
    pub channel_10, set_channel_10: 10;
    /// Returns or changes the channel 11 control.
    pub channel_11, set_channel_11: 11;
    /// Returns or changes the channel 12 control.
    pub channel_12, set_channel_12: 12;
    /// Returns or changes the channel 13 control.
    pub channel_13, set_channel_13: 13;
    /// Returns or changes the channel 14 control.
    pub channel_14, set_channel_14: 14;
    /// Returns or changes the channel 15 control.
    pub channel_15, set_channel_15: 15;
}
impl From<u16> for WorldForces {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<WorldForces> for u16 {
    fn from(value: WorldForces) -> Self {
        value.bits()
    }
}
