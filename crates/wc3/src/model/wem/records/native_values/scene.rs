//! Native scene selectors and flags preserve their encoded widths.
use crate::model::wem;
use bitfield::bitfield;

bitfield! {
    /// Native Sc2ParticleFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct Sc2ParticleFlags(u32);
    /// Returns the exact stored bits.
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
impl Sc2ParticleFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u32> for Sc2ParticleFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<Sc2ParticleFlags> for u32 {
    fn from(flags: Sc2ParticleFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native Sc2ParticleAdditionalFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct Sc2ParticleAdditionalFlags(u32);
    /// Returns the exact stored bits.
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
impl Sc2ParticleAdditionalFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u32> for Sc2ParticleAdditionalFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<Sc2ParticleAdditionalFlags> for u32 {
    fn from(flags: Sc2ParticleAdditionalFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native Sc2ParticleRotationFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct Sc2ParticleRotationFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the relative control.
    pub relative, set_relative: 1;
    /// Returns or changes the always set control.
    pub always_set, set_always_set: 2;
}
impl Sc2ParticleRotationFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u32> for Sc2ParticleRotationFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<Sc2ParticleRotationFlags> for u32 {
    fn from(flags: Sc2ParticleRotationFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native Sc2RibbonFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct Sc2RibbonFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the collide terrain control.
    pub collide_terrain, set_collide_terrain: 1;
    /// Returns or changes the collide objects control.
    pub collide_objects, set_collide_objects: 2;
    /// Returns or changes the edge falloff control.
    pub edge_falloff, set_edge_falloff: 3;
    /// Returns or changes the inherit parent velocity control.
    pub inherit_parent_velocity, set_inherit_parent_velocity: 4;
    /// Returns or changes the scale smooth control.
    pub scale_smooth, set_scale_smooth: 5;
    /// Returns or changes the scale bezier control.
    pub scale_bezier, set_scale_bezier: 6;
    /// Returns or changes the vertex alpha control.
    pub vertex_alpha, set_vertex_alpha: 7;
    /// Returns or changes the scale time parent control.
    pub scale_time_parent, set_scale_time_parent: 8;
    /// Returns or changes the force cpu sim control.
    pub force_cpu_sim, set_force_cpu_sim: 9;
    /// Returns or changes the local time control.
    pub local_time, set_local_time: 10;
    /// Returns or changes the simulate init control.
    pub simulate_init, set_simulate_init: 11;
    /// Returns or changes the length time control.
    pub length_time, set_length_time: 12;
    /// Returns or changes the accurate gpu tangents control.
    pub accurate_gpu_tangents, set_accurate_gpu_tangents: 13;
}
impl Sc2RibbonFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u32> for Sc2RibbonFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<Sc2RibbonFlags> for u32 {
    fn from(flags: Sc2RibbonFlags) -> Self {
        flags.bits()
    }
}

bitfield! {
    /// Native Sc2RibbonAdditionalFlags bit field; unnamed bits remain representable.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, wem::Read, wem::Write, wem::RecordLayout, wem::Validate, wem::VisitReferences)]
    pub struct Sc2RibbonAdditionalFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the speed randomize control.
    pub speed_randomize, set_speed_randomize: 0;
    /// Returns or changes the lifespan randomize control.
    pub lifespan_randomize, set_lifespan_randomize: 1;
    /// Returns or changes the mass randomize control.
    pub mass_randomize, set_mass_randomize: 2;
    /// Returns or changes the world space control.
    pub world_space, set_world_space: 3;
}
impl Sc2RibbonAdditionalFlags {
    /// Preserves known and unrecognized bits.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl From<u32> for Sc2RibbonAdditionalFlags {
    fn from(bits: u32) -> Self {
        Self(bits)
    }
}
impl From<Sc2RibbonAdditionalFlags> for u32 {
    fn from(flags: Sc2RibbonAdditionalFlags) -> Self {
        flags.bits()
    }
}

/// Wave function used by StarCraft II particle and ribbon variation controls.
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
pub enum Sc2VariationKind {
    /// No variation.
    #[default]
    #[wem(value = 0)]
    None,
    /// Sine wave.
    #[wem(value = 1)]
    Sine,
    /// Cosine wave.
    #[wem(value = 2)]
    Cosine,
    /// Saw wave.
    #[wem(value = 3)]
    Saw,
    /// Square wave.
    #[wem(value = 4)]
    Square,
    /// Independent random noise.
    #[wem(value = 5)]
    RandomNoise,
    /// Continuous noise.
    #[wem(value = 6)]
    ContinuousNoise,
    /// Unrecognized wave-function code.
    #[wem(unknown)]
    Unknown(u32),
}

/// Direction used to initialize StarCraft II particle velocity.
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
pub enum Sc2EmissionDirection {
    /// Constant emission direction.
    #[default]
    #[wem(value = 0)]
    Constant,
    /// Radial emission direction.
    #[wem(value = 1)]
    Radial,
    /// Direction along the Z axis.
    #[wem(value = 2)]
    ZAxis,
    /// Random emission direction.
    #[wem(value = 3)]
    Random,
    /// Direction along the emitting mesh normal.
    #[wem(value = 4)]
    MeshNormal,
    /// Unrecognized direction selector.
    #[wem(unknown)]
    Unknown(u32),
}
