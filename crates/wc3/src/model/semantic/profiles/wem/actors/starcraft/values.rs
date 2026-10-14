//! StarCraft emitter behavior selectors and editable state bits.
use bitfield::bitfield;
/// EmitterShape behavior selected for the effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmitterShape {
    #[default]
    /// Point behavior.
    Point,
    /// Plane behavior.
    Plane,
    /// Sphere behavior.
    Sphere,
    /// Box behavior.
    Box,
    /// Cylinder behavior.
    Cylinder,
    /// Disc behavior.
    Disc,
    /// Spline behavior.
    Spline,
    /// Mesh behavior.
    Mesh,
    /// Authored selector without an established interpretation.
    Unknown(u32),
}
impl EmitterShape {
    /// Selects a documented behavior or retains the authored selector.
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
    /// Authored behavior selector.
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
}
/// ParticleInstance behavior selected for the effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ParticleInstance {
    #[default]
    /// Billboard behavior.
    Billboard,
    /// Tail behavior.
    Tail,
    /// FaceTravelDir behavior.
    FaceTravelDir,
    /// FaceWorldDir behavior.
    FaceWorldDir,
    /// SingleAxis behavior.
    SingleAxis,
    /// TerrainOriented behavior.
    TerrainOriented,
    /// TerrainDirOriented behavior.
    TerrainDirOriented,
    /// EmitterOriented behavior.
    EmitterOriented,
    /// PhysicsOriented behavior.
    PhysicsOriented,
    /// Pinned behavior.
    Pinned,
    /// Trail behavior.
    Trail,
    /// Authored selector without an established interpretation.
    Unknown(u32),
}
impl ParticleInstance {
    /// Selects a documented behavior or retains the authored selector.
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
    /// Authored behavior selector.
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
}
/// Smoothing behavior selected for the effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Smoothing {
    #[default]
    /// Linear behavior.
    Linear,
    /// LinearSmooth behavior.
    LinearSmooth,
    /// Bezier behavior.
    Bezier,
    /// LinearWithHold behavior.
    LinearWithHold,
    /// BezierWithHold behavior.
    BezierWithHold,
    /// Authored selector without an established interpretation.
    Unknown(u32),
}
impl Smoothing {
    /// Selects a documented behavior or retains the authored selector.
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
    /// Authored behavior selector.
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
}
/// RibbonType behavior selected for the effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RibbonType {
    #[default]
    /// Billboard behavior.
    Billboard,
    /// Planar behavior.
    Planar,
    /// Cylinder behavior.
    Cylinder,
    /// Star behavior.
    Star,
    /// Authored selector without an established interpretation.
    Unknown(u32),
}
impl RibbonType {
    /// Selects a documented behavior or retains the authored selector.
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Billboard,
            1 => Self::Planar,
            2 => Self::Cylinder,
            3 => Self::Star,
            value => Self::Unknown(value),
        }
    }
    /// Authored behavior selector.
    pub const fn raw(self) -> u32 {
        match self {
            Self::Billboard => 0,
            Self::Planar => 1,
            Self::Cylinder => 2,
            Self::Star => 3,
            Self::Unknown(value) => value,
        }
    }
}
/// VariationKind behavior selected for the effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VariationKind {
    #[default]
    /// None behavior.
    None,
    /// Sine behavior.
    Sine,
    /// Cosine behavior.
    Cosine,
    /// Saw behavior.
    Saw,
    /// Square behavior.
    Square,
    /// RandomNoise behavior.
    RandomNoise,
    /// ContinuousNoise behavior.
    ContinuousNoise,
    /// Authored selector without an established interpretation.
    Unknown(u32),
}
impl VariationKind {
    /// Selects a documented behavior or retains the authored selector.
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
    /// Authored behavior selector.
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
}
/// EmissionDirection behavior selected for the effect.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmissionDirection {
    #[default]
    /// Constant behavior.
    Constant,
    /// Radial behavior.
    Radial,
    /// ZAxis behavior.
    ZAxis,
    /// Random behavior.
    Random,
    /// MeshNormal behavior.
    MeshNormal,
    /// Authored selector without an established interpretation.
    Unknown(u32),
}
impl EmissionDirection {
    /// Selects a documented behavior or retains the authored selector.
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
    /// Authored behavior selector.
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
}
bitfield! {
    /// ParticleFlags controls, including independently retained unnamed bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct ParticleFlags(u32);
    /// Reads or changes the bits control.
    pub bits,_:31, 0;
    /// Reads or changes the sort distance control.
    pub sort_distance,set_sort_distance:0;
    /// Reads or changes the collide terrain control.
    pub collide_terrain,set_collide_terrain:1;
    /// Reads or changes the collide objects control.
    pub collide_objects,set_collide_objects:2;
    /// Reads or changes the collide emit control.
    pub collide_emit,set_collide_emit:3;
    /// Reads or changes the emit shape cutout control.
    pub emit_shape_cutout,set_emit_shape_cutout:4;
    /// Reads or changes the inherit emit params control.
    pub inherit_emit_params,set_inherit_emit_params:5;
    /// Reads or changes the inherit parent velocity control.
    pub inherit_parent_velocity,set_inherit_parent_velocity:6;
    /// Reads or changes the sort height control.
    pub sort_height,set_sort_height:7;
    /// Reads or changes the sort reverse control.
    pub sort_reverse,set_sort_reverse:8;
    /// Reads or changes the old rotation smooth control.
    pub old_rotation_smooth,set_old_rotation_smooth:9;
    /// Reads or changes the old rotation bezier control.
    pub old_rotation_bezier,set_old_rotation_bezier:10;
    /// Reads or changes the old size smooth control.
    pub old_size_smooth,set_old_size_smooth:11;
    /// Reads or changes the old size bezier control.
    pub old_size_bezier,set_old_size_bezier:12;
    /// Reads or changes the old color smooth control.
    pub old_color_smooth,set_old_color_smooth:13;
    /// Reads or changes the old color bezier control.
    pub old_color_bezier,set_old_color_bezier:14;
    /// Reads or changes the lit parts control.
    pub lit_parts,set_lit_parts:15;
    /// Reads or changes the random uv flipbook start control.
    pub random_uv_flipbook_start,set_random_uv_flipbook_start:16;
    /// Reads or changes the multiply gravity control.
    pub multiply_gravity,set_multiply_gravity:17;
    /// Reads or changes the tail clamp control.
    pub tail_clamp,set_tail_clamp:18;
    /// Reads or changes the use trailing particle control.
    pub use_trailing_particle,set_use_trailing_particle:19;
    /// Reads or changes the tail fix control.
    pub tail_fix,set_tail_fix:20;
    /// Reads or changes the vertex alpha control.
    pub vertex_alpha,set_vertex_alpha:21;
    /// Reads or changes the model particles control.
    pub model_particles,set_model_particles:22;
    /// Reads or changes the swap yz on model particles control.
    pub swap_yz_on_model_particles,set_swap_yz_on_model_particles:23;
    /// Reads or changes the scale time parent control.
    pub scale_time_parent,set_scale_time_parent:24;
    /// Reads or changes the local time control.
    pub local_time,set_local_time:25;
    /// Reads or changes the simulate init control.
    pub simulate_init,set_simulate_init:26;
    /// Reads or changes the copy control.
    pub copy,set_copy:27;
}
impl ParticleFlags {
    /// Retains the entire authored bit pattern.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// ParticleAdditionalFlags controls, including independently retained unnamed bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct ParticleAdditionalFlags(u32);
    /// Reads or changes the bits control.
    pub bits,_:31, 0;
    /// Reads or changes the emit speed randomize control.
    pub emit_speed_randomize,set_emit_speed_randomize:0;
    /// Reads or changes the lifespan randomize control.
    pub lifespan_randomize,set_lifespan_randomize:1;
    /// Reads or changes the mass randomize control.
    pub mass_randomize,set_mass_randomize:2;
    /// Reads or changes the world space control.
    pub world_space,set_world_space:3;
}
impl ParticleAdditionalFlags {
    /// Retains the entire authored bit pattern.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// ParticleRotationFlags controls, including independently retained unnamed bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct ParticleRotationFlags(u32);
    /// Reads or changes the bits control.
    pub bits,_:31, 0;
    /// Reads or changes the relative control.
    pub relative,set_relative:1;
    /// Reads or changes the always set control.
    pub always_set,set_always_set:2;
}
impl ParticleRotationFlags {
    /// Retains the entire authored bit pattern.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// RibbonFlags controls, including independently retained unnamed bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct RibbonFlags(u32);
    /// Reads or changes the bits control.
    pub bits,_:31, 0;
    /// Reads or changes the collide terrain control.
    pub collide_terrain,set_collide_terrain:1;
    /// Reads or changes the collide objects control.
    pub collide_objects,set_collide_objects:2;
    /// Reads or changes the edge falloff control.
    pub edge_falloff,set_edge_falloff:3;
    /// Reads or changes the inherit parent velocity control.
    pub inherit_parent_velocity,set_inherit_parent_velocity:4;
    /// Reads or changes the scale smooth control.
    pub scale_smooth,set_scale_smooth:5;
    /// Reads or changes the scale bezier control.
    pub scale_bezier,set_scale_bezier:6;
    /// Reads or changes the vertex alpha control.
    pub vertex_alpha,set_vertex_alpha:7;
    /// Reads or changes the scale time parent control.
    pub scale_time_parent,set_scale_time_parent:8;
    /// Reads or changes the force cpu sim control.
    pub force_cpu_sim,set_force_cpu_sim:9;
    /// Reads or changes the local time control.
    pub local_time,set_local_time:10;
    /// Reads or changes the simulate init control.
    pub simulate_init,set_simulate_init:11;
    /// Reads or changes the length time control.
    pub length_time,set_length_time:12;
    /// Reads or changes the accurate gpu tangents control.
    pub accurate_gpu_tangents,set_accurate_gpu_tangents:13;
}
impl RibbonFlags {
    /// Retains the entire authored bit pattern.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// RibbonAdditionalFlags controls, including independently retained unnamed bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct RibbonAdditionalFlags(u32);
    /// Reads or changes the bits control.
    pub bits,_:31, 0;
    /// Reads or changes the speed randomize control.
    pub speed_randomize,set_speed_randomize:0;
    /// Reads or changes the lifespan randomize control.
    pub lifespan_randomize,set_lifespan_randomize:1;
    /// Reads or changes the mass randomize control.
    pub mass_randomize,set_mass_randomize:2;
    /// Reads or changes the world space control.
    pub world_space,set_world_space:3;
}
impl RibbonAdditionalFlags {
    /// Retains the entire authored bit pattern.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// LocalForceChannels controls, including independently retained unnamed bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct LocalForceChannels(u16);
    /// Reads or changes the bits control.
    pub bits,_:15, 0;
    /// Reads or changes the channel 0 control.
    pub channel_0,set_channel_0:0;
    /// Reads or changes the channel 1 control.
    pub channel_1,set_channel_1:1;
    /// Reads or changes the channel 2 control.
    pub channel_2,set_channel_2:2;
    /// Reads or changes the channel 3 control.
    pub channel_3,set_channel_3:3;
    /// Reads or changes the channel 4 control.
    pub channel_4,set_channel_4:4;
    /// Reads or changes the channel 5 control.
    pub channel_5,set_channel_5:5;
    /// Reads or changes the channel 6 control.
    pub channel_6,set_channel_6:6;
    /// Reads or changes the channel 7 control.
    pub channel_7,set_channel_7:7;
    /// Reads or changes the channel 8 control.
    pub channel_8,set_channel_8:8;
    /// Reads or changes the channel 9 control.
    pub channel_9,set_channel_9:9;
    /// Reads or changes the channel 10 control.
    pub channel_10,set_channel_10:10;
    /// Reads or changes the channel 11 control.
    pub channel_11,set_channel_11:11;
    /// Reads or changes the channel 12 control.
    pub channel_12,set_channel_12:12;
    /// Reads or changes the channel 13 control.
    pub channel_13,set_channel_13:13;
    /// Reads or changes the channel 14 control.
    pub channel_14,set_channel_14:14;
    /// Reads or changes the channel 15 control.
    pub channel_15,set_channel_15:15;
}
impl LocalForceChannels {
    /// Retains the entire authored bit pattern.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
bitfield! {
    /// WorldForceChannels controls, including independently retained unnamed bits.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct WorldForceChannels(u16);
    /// Reads or changes the bits control.
    pub bits,_:15, 0;
    /// Reads or changes the channel 0 control.
    pub channel_0,set_channel_0:0;
    /// Reads or changes the channel 1 control.
    pub channel_1,set_channel_1:1;
    /// Reads or changes the channel 2 control.
    pub channel_2,set_channel_2:2;
    /// Reads or changes the channel 3 control.
    pub channel_3,set_channel_3:3;
    /// Reads or changes the channel 4 control.
    pub channel_4,set_channel_4:4;
    /// Reads or changes the channel 5 control.
    pub channel_5,set_channel_5:5;
    /// Reads or changes the channel 6 control.
    pub channel_6,set_channel_6:6;
    /// Reads or changes the channel 7 control.
    pub channel_7,set_channel_7:7;
    /// Reads or changes the channel 8 control.
    pub channel_8,set_channel_8:8;
    /// Reads or changes the channel 9 control.
    pub channel_9,set_channel_9:9;
    /// Reads or changes the channel 10 control.
    pub channel_10,set_channel_10:10;
    /// Reads or changes the channel 11 control.
    pub channel_11,set_channel_11:11;
    /// Reads or changes the channel 12 control.
    pub channel_12,set_channel_12:12;
    /// Reads or changes the channel 13 control.
    pub channel_13,set_channel_13:13;
    /// Reads or changes the channel 14 control.
    pub channel_14,set_channel_14:14;
    /// Reads or changes the channel 15 control.
    pub channel_15,set_channel_15:15;
}
impl WorldForceChannels {
    /// Retains the entire authored bit pattern.
    pub const fn from_bits_retain(bits: u16) -> Self {
        Self(bits)
    }
}
