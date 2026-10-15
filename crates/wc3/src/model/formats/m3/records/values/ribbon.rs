//! Native ribbon selectors and bit masks.
use crate::model::formats::m3;
use bitfield::bitfield;

/// Native RibbonKind selector.
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
pub enum RibbonKind {
    /// Selects Billboard.
    #[m3(value = 0)]
    Billboard,
    /// Selects Planar.
    #[m3(value = 1)]
    Planar,
    /// Selects Cylinder.
    #[m3(value = 2)]
    Cylinder,
    /// Selects Star.
    #[m3(value = 3)]
    Star,
    /// An unrecognized native code, retained without reinterpretation.
    #[m3(unknown)]
    Unknown(u32),
}
impl Default for RibbonKind {
    fn default() -> Self {
        Self::from_raw(0)
    }
}

bitfield! {
    /// Native RibbonFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct RibbonFlags(u32);
    /// Returns the complete stored bit mask.
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
impl From<u32> for RibbonFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<RibbonFlags> for u32 {
    fn from(value: RibbonFlags) -> Self {
        value.bits()
    }
}

bitfield! {
    /// Native RibbonAdditionalFlags bit mask.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, m3::Read, m3::Write, m3::RecordLayout, m3::Validate, m3::VisitReferences)]
    pub struct RibbonAdditionalFlags(u32);
    /// Returns the complete stored bit mask.
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
impl From<u32> for RibbonAdditionalFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<RibbonAdditionalFlags> for u32 {
    fn from(value: RibbonAdditionalFlags) -> Self {
        value.bits()
    }
}
