//! Native particle selectors and bit masks.
use crate::model::formats::m3;
use bitfield::bitfield;

/// Native EmitterShape selector.
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
pub enum EmitterShape {
    /// Selects Point.
    #[m3(value = 0)]
    Point,
    /// Selects Plane.
    #[m3(value = 1)]
    Plane,
    /// Selects Sphere.
    #[m3(value = 2)]
    Sphere,
    /// Selects Box.
    #[m3(value = 3)]
    Box,
    /// Selects Cylinder.
    #[m3(value = 4)]
    Cylinder,
    /// Selects Disc.
    #[m3(value = 5)]
    Disc,
    /// Selects Spline.
    #[m3(value = 6)]
    Spline,
    /// Selects Mesh.
    #[m3(value = 7)]
    Mesh,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for EmitterShape {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native ParticleInstance selector.
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
pub enum ParticleInstance {
    /// Selects Billboard.
    #[m3(value = 0)]
    Billboard,
    /// Selects Tail.
    #[m3(value = 1)]
    Tail,
    /// Selects FaceTravelDir.
    #[m3(value = 2)]
    FaceTravelDir,
    /// Selects FaceWorldDir.
    #[m3(value = 3)]
    FaceWorldDir,
    /// Selects SingleAxis.
    #[m3(value = 4)]
    SingleAxis,
    /// Selects TerrainOriented.
    #[m3(value = 5)]
    TerrainOriented,
    /// Selects TerrainDirOriented.
    #[m3(value = 6)]
    TerrainDirOriented,
    /// Selects EmitterOriented.
    #[m3(value = 7)]
    EmitterOriented,
    /// Selects PhysicsOriented.
    #[m3(value = 8)]
    PhysicsOriented,
    /// Selects Pinned.
    #[m3(value = 9)]
    Pinned,
    /// Selects Trail.
    #[m3(value = 10)]
    Trail,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for ParticleInstance {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native Smoothing selector.
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
pub enum Smoothing {
    /// Selects Linear.
    #[m3(value = 0)]
    Linear,
    /// Selects LinearSmooth.
    #[m3(value = 1)]
    LinearSmooth,
    /// Selects Bezier.
    #[m3(value = 2)]
    Bezier,
    /// Selects LinearWithHold.
    #[m3(value = 3)]
    LinearWithHold,
    /// Selects BezierWithHold.
    #[m3(value = 4)]
    BezierWithHold,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for Smoothing {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native Variation selector.
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
pub enum Variation {
    /// Selects None.
    #[m3(value = 0)]
    None,
    /// Selects Sine.
    #[m3(value = 1)]
    Sine,
    /// Selects Cosine.
    #[m3(value = 2)]
    Cosine,
    /// Selects Saw.
    #[m3(value = 3)]
    Saw,
    /// Selects Square.
    #[m3(value = 4)]
    Square,
    /// Selects RandomNoise.
    #[m3(value = 5)]
    RandomNoise,
    /// Selects ContinuousNoise.
    #[m3(value = 6)]
    ContinuousNoise,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for Variation {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

/// Native VelocityKind selector.
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
pub enum VelocityKind {
    /// Selects Constant.
    #[m3(value = 0)]
    Constant,
    /// Selects Radial.
    #[m3(value = 1)]
    Radial,
    /// Selects ZAxis.
    #[m3(value = 2)]
    ZAxis,
    /// Selects Random.
    #[m3(value = 3)]
    Random,
    /// Selects MeshNormal.
    #[m3(value = 4)]
    MeshNormal,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for VelocityKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// Native ParticleFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
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
    /// Native ParticleAdditionalFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
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
    /// Native ParticleRotationFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
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
    /// Native LocalForces bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
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
    /// Native WorldForces bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
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
