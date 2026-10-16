//! Native WMO behavior words with accessors for established bits.
use super::BspAxis;
use crate::model::{common::Ownership, formats::wmo};
use bitfield::bitfield;

bitfield! {
    /// Native root behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct RootFlags(u16);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 15, 0;
    /// Returns or changes the no vertex attenuation bit.
    pub no_vertex_attenuation, set_no_vertex_attenuation: 0;
    /// Returns or changes the unified render path bit.
    pub unified_render_path, set_unified_render_path: 1;
    /// Returns or changes the liquid type database id bit.
    pub liquid_type_database_id, set_liquid_type_database_id: 2;
    /// Preserve vertex RGB while correcting non-transition lighting alpha.
    pub vertex_color_alpha_only, set_vertex_color_alpha_only: 3;
    /// Returns or changes authored group LOD availability.
    pub has_group_lods, set_has_group_lods: 4;
    /// Returns or changes default maximum LOD selection.
    pub default_max_lod, set_default_max_lod: 5;
}
impl From<u16> for RootFlags {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<RootFlags> for u16 {
    fn from(value: RootFlags) -> Self {
        value.bits()
    }
}
impl From<RootFlags> for i64 {
    fn from(value: RootFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native material behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct MaterialFlags(u32);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 31, 0;
    /// Returns or changes the unlit bit.
    pub unlit, set_unlit: 0;
    /// Returns or changes the unfogged bit.
    pub unfogged, set_unfogged: 1;
    /// Returns or changes the two sided bit.
    pub two_sided, set_two_sided: 2;
    /// Returns or changes the exterior lit bit.
    pub exterior_lit, set_exterior_lit: 3;
    /// Returns or changes the clamp s bit.
    pub clamp_s, set_clamp_s: 6;
    /// Returns or changes the clamp t bit.
    pub clamp_t, set_clamp_t: 7;
}
impl From<u32> for MaterialFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<MaterialFlags> for u32 {
    fn from(value: MaterialFlags) -> Self {
        value.bits()
    }
}
impl From<MaterialFlags> for i64 {
    fn from(value: MaterialFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native materialf behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct MaterialFlags2(u32);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 31, 0;
}
impl From<u32> for MaterialFlags2 {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<MaterialFlags2> for u32 {
    fn from(value: MaterialFlags2) -> Self {
        value.bits()
    }
}
impl From<MaterialFlags2> for i64 {
    fn from(value: MaterialFlags2) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native group behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct GroupFlags(u32);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 31, 0;
    /// Returns or changes the has bsp bit.
    pub has_bsp, set_has_bsp: 0;
    /// Returns or changes the has lightmap bit.
    pub has_lightmap, set_has_lightmap: 1;
    /// Returns or changes the has vertex colors bit.
    pub has_vertex_colors, set_has_vertex_colors: 2;
    /// Returns or changes the exterior bit.
    pub exterior, set_exterior: 3;
    /// Returns or changes the exterior lit bit.
    pub exterior_lit, set_exterior_lit: 6;
    /// Returns or changes the unreachable bit.
    pub unreachable, set_unreachable: 7;
    /// Returns or changes the show exterior sky bit.
    pub show_exterior_sky, set_show_exterior_sky: 8;
    /// Returns or changes the has lights bit.
    pub has_lights, set_has_lights: 9;
    /// Returns or changes the lod bit.
    pub lod, set_lod: 10;
    /// Returns or changes the has doodads bit.
    pub has_doodads, set_has_doodads: 11;
    /// Returns or changes the has liquid bit.
    pub has_liquid, set_has_liquid: 12;
    /// Returns or changes the interior bit.
    pub interior, set_interior: 13;
}
impl From<u32> for GroupFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<GroupFlags> for u32 {
    fn from(value: GroupFlags) -> Self {
        value.bits()
    }
}
impl From<GroupFlags> for i64 {
    fn from(value: GroupFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native groupf behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct GroupFlags2(u32);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 31, 0;
}
impl From<u32> for GroupFlags2 {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<GroupFlags2> for u32 {
    fn from(value: GroupFlags2) -> Self {
        value.bits()
    }
}
impl From<GroupFlags2> for i64 {
    fn from(value: GroupFlags2) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native portal behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct PortalFlags(u16);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 15, 0;
    /// Returns or changes the skip viewer queries bit.
    pub skip_viewer_queries, set_skip_viewer_queries: 0;
    /// Returns or changes the plane adjustment bit.
    pub plane_adjustment, set_plane_adjustment: 5;
}
impl From<u16> for PortalFlags {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<PortalFlags> for u16 {
    fn from(value: PortalFlags) -> Self {
        value.bits()
    }
}
impl From<PortalFlags> for i64 {
    fn from(value: PortalFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native triangle behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct TriangleFlags(u8);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 7, 0;

    /// Returns or changes the transition bit.
    pub transition, set_transition: 0;
    /// Returns or changes the no camera collision bit.
    pub no_camera_collision, set_no_camera_collision: 1;
    /// Returns or changes the detail bit.
    pub detail, set_detail: 2;
    /// Returns or changes the collision bit.
    pub collision, set_collision: 3;
    /// Returns or changes the hint bit.
    pub hint, set_hint: 4;
    /// Returns or changes the render bit.
    pub render, set_render: 5;
    /// Returns or changes the cull objects bit.
    pub cull_objects, set_cull_objects: 6;
    /// Returns or changes the collide hit bit.
    pub collide_hit, set_collide_hit: 7;
}
impl From<u8> for TriangleFlags {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<TriangleFlags> for u8 {
    fn from(value: TriangleFlags) -> Self {
        value.bits()
    }
}
impl From<TriangleFlags> for i64 {
    fn from(value: TriangleFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native trianglef behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct TriangleFlags2(u16);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 15, 0;

    /// Returns or changes the transition bit.
    pub transition, set_transition: 0;
    /// Returns or changes the no camera collision bit.
    pub no_camera_collision, set_no_camera_collision: 1;
    /// Returns or changes the detail bit.
    pub detail, set_detail: 2;
    /// Returns or changes the collision bit.
    pub collision, set_collision: 3;
    /// Returns or changes the hint bit.
    pub hint, set_hint: 4;
    /// Returns or changes the render bit.
    pub render, set_render: 5;
    /// Returns or changes the cull objects bit.
    pub cull_objects, set_cull_objects: 6;
    /// Returns or changes the collide hit bit.
    pub collide_hit, set_collide_hit: 7;
    /// Returns or changes the ground type entry bit.
    pub ground_type_entry, set_ground_type_entry: 8;
}
impl From<u16> for TriangleFlags2 {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<TriangleFlags2> for u16 {
    fn from(value: TriangleFlags2) -> Self {
        value.bits()
    }
}
impl From<TriangleFlags2> for i64 {
    fn from(value: TriangleFlags2) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native batch behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct BatchFlags(u8);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 7, 0;
    /// Returns or changes the large material bit.
    pub large_material, set_large_material: 1;
    /// Returns or changes the extended vertices bit.
    pub extended_vertices, set_extended_vertices: 2;
}
impl From<u8> for BatchFlags {
    fn from(value: u8) -> Self {
        Self(value)
    }
}
impl From<BatchFlags> for u8 {
    fn from(value: BatchFlags) -> Self {
        value.bits()
    }
}
impl From<BatchFlags> for i64 {
    fn from(value: BatchFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native bsp behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct BspFlags(u16);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 15, 0;
    /// Returns or changes the leaf bit.
    pub leaf, set_leaf: 2;
    /// Returns or changes the split-axis code.
    axis_bits, set_axis_bits: 1, 0;
}
impl From<u16> for BspFlags {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
impl From<BspFlags> for u16 {
    fn from(value: BspFlags) -> Self {
        value.bits()
    }
}
impl From<BspFlags> for i64 {
    fn from(value: BspFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native fog behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct FogFlags(u32);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 31, 0;
    /// Returns or changes the infinite radius bit.
    pub infinite_radius, set_infinite_radius: 0;
}
impl From<u32> for FogFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<FogFlags> for u32 {
    fn from(value: FogFlags) -> Self {
        value.bits()
    }
}
impl From<FogFlags> for i64 {
    fn from(value: FogFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native fogbox behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct FogBoxFlags(u32);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 31, 0;
}
impl From<u32> for FogBoxFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<FogBoxFlags> for u32 {
    fn from(value: FogBoxFlags) -> Self {
        value.bits()
    }
}
impl From<FogBoxFlags> for i64 {
    fn from(value: FogBoxFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native ambient behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct AmbientFlags(u32);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 31, 0;
    /// Returns or changes the distinct colors bit.
    pub distinct_colors, set_distinct_colors: 0;
}
impl From<u32> for AmbientFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<AmbientFlags> for u32 {
    fn from(value: AmbientFlags) -> Self {
        value.bits()
    }
}
impl From<AmbientFlags> for i64 {
    fn from(value: AmbientFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native ambientbox behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct AmbientBoxFlags(u32);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 31, 0;
}
impl From<u32> for AmbientBoxFlags {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<AmbientBoxFlags> for u32 {
    fn from(value: AmbientBoxFlags) -> Self {
        value.bits()
    }
}
impl From<AmbientBoxFlags> for i64 {
    fn from(value: AmbientBoxFlags) -> Self {
        i64::from(value.bits())
    }
}

bitfield! {
    /// Native light behavior flags; unassigned bits remain intact.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct LightFlags(i32);
    /// Returns every stored bit, including unassigned bits.
    pub bits, _: 31, 0;
    /// Returns or changes the blend colors bit.
    pub blend_colors, set_blend_colors: 0;
    /// Returns or changes the cast shadows bit.
    pub cast_shadows, set_cast_shadows: 1;
}
impl From<i32> for LightFlags {
    fn from(value: i32) -> Self {
        Self(value)
    }
}
impl From<LightFlags> for i32 {
    fn from(value: LightFlags) -> Self {
        value.bits()
    }
}
impl From<LightFlags> for i64 {
    fn from(value: LightFlags) -> Self {
        i64::from(value.bits())
    }
}

impl From<RootFlags> for u32 {
    fn from(value: RootFlags) -> Self {
        u32::from(value.bits())
    }
}

impl From<TriangleFlags> for u32 {
    fn from(value: TriangleFlags) -> Self {
        u32::from(value.bits())
    }
}

impl From<BatchFlags> for u32 {
    fn from(value: BatchFlags) -> Self {
        u32::from(value.bits())
    }
}

impl From<PortalFlags> for u32 {
    fn from(value: PortalFlags) -> Self {
        u32::from(value.bits())
    }
}

impl From<BspFlags> for u32 {
    fn from(value: BspFlags) -> Self {
        u32::from(value.bits())
    }
}

impl From<TriangleFlags2> for u32 {
    fn from(value: TriangleFlags2) -> Self {
        u32::from(value.bits())
    }
}

impl From<TriangleFlags> for u16 {
    fn from(value: TriangleFlags) -> Self {
        u16::from(value.bits())
    }
}

bitfield! {
    /// Behavior flags stored in the high byte of a doodad reference.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct DoodadFlags(u8);
    /// Exact native bits, including unassigned bits.
    pub bits, _: 7, 0;
    /// Whether the tint alpha byte selects a root light.
    pub has_light_reference, set_has_light_reference: 2;
}
bitfield! {
    /// Packed doodad model identity and behavior flags.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ownership, wmo::Read, wmo::Write, wmo::RecordLayout, wmo::Validate)]
    pub struct DoodadReference(u32);
    /// Exact native packed word.
    pub bits, _: 31, 0;
    /// Returns or changes the 24-bit model-name offset or model-ID index.
    pub name_index, set_name_index: 23, 0;
    u8;
    flag_bits, set_flag_bits: 31, 24;
}
impl DoodadReference {
    /// Returns the behavior byte as typed flags.
    pub fn flags(&self) -> DoodadFlags {
        DoodadFlags(self.flag_bits())
    }
    /// Replaces behavior flags while retaining the model identity.
    pub fn set_flags(&mut self, value: DoodadFlags) {
        self.set_flag_bits(value.bits());
    }
}
impl From<u32> for DoodadReference {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl From<DoodadReference> for u32 {
    fn from(value: DoodadReference) -> Self {
        value.bits()
    }
}
impl From<DoodadFlags> for i64 {
    fn from(value: DoodadFlags) -> Self {
        i64::from(value.bits())
    }
}

impl BspFlags {
    /// Returns the packed split-axis choice without changing the other flags.
    pub fn split_axis(&self) -> BspAxis {
        BspAxis::from_raw(self.axis_bits())
    }
    /// Changes the split axis while retaining leaf and unassigned flags.
    pub fn set_split_axis(&mut self, axis: BspAxis) -> Result<(), wmo::WriteError> {
        if axis.raw() > 3 {
            return Err(wmo::WriteError::invalid(
                "BSP axis exceeds its two-bit field",
            ));
        }
        self.set_axis_bits(axis.raw());
        Ok(())
    }
}
