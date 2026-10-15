//! World-model scene semantics independent of WMO chunk revisions.
pub use crate::model::semantic::{SurfaceTriangleControls, TriangleFlags};
mod flags;
pub use flags::*;
mod draw;
mod region_lights;
pub use region_lights::*;
mod links;
pub use draw::*;
mod materials;
mod validation;
mod world;
use crate::model::semantic::{FeatureId, InstanceId, Resource, ZeroVec};
pub use materials::*;
use std::borrow::Cow;
pub use world::*;

/// Identity of a world group in its owning world system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GroupId {
    /// Owning world system.
    pub system: crate::model::semantic::SystemId,
    /// Local group identity.
    pub index: u32,
}
/// Identity of an authored portal in its owning world system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PortalId {
    /// Owning world system.
    pub system: crate::model::semantic::SystemId,
    /// Local portal identity.
    pub index: u32,
}
/// Identity of a doodad set in its owning world system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DoodadSetId {
    /// Owning world system.
    pub system: crate::model::semantic::SystemId,
    /// Local set identity.
    pub index: u32,
}
/// Identity of an environmental fog volume in its owning world system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FogId {
    /// Owning world system.
    pub system: crate::model::semantic::SystemId,
    /// Local fog identity.
    pub index: u32,
}
/// Identity of an ambient illumination volume in its owning world system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AmbientId {
    /// Owning world system.
    pub system: crate::model::semantic::SystemId,
    /// Local ambient-volume identity.
    pub index: u32,
}
/// Named group in the world catalog and its placed geometry at each detail level.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group<'a> {
    /// Catalog display name.
    pub name: Cow<'a, str>,
    /// Geometry placements; each placement owns its authored detail level.
    pub instances: Vec<InstanceId>,
    /// Catalog visibility and illumination controls.
    pub options: GroupOptions,
    /// Catalog minimum and maximum bounds.
    pub bounds: [[f32; 3]; 2],
    /// Supplemental catalog flags when authored.
    pub supplemental: Option<GroupFlags2>,
    /// Catalog detail-level selection when authored.
    pub detail_selection: Option<u32>,
}
/// Region behavior and authored relationships of one placed world geometry.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RegionControls<'a> {
    /// Authoring description for this region.
    pub description: Cow<'a, str>,
    /// Database identity of this region's separately addressable geometry, when assigned.
    pub file_id: Option<u32>,
    /// External area-table group identifier.
    pub area_id: u32,
    /// Region visibility and illumination controls.
    pub options: GroupOptions,
    /// Split parent or first child in the world catalog.
    pub parent_or_first_child: Option<GroupId>,
    /// Next sibling in a split-child chain.
    pub next_child: Option<GroupId>,
    /// Doodad features visible within this region.
    pub doodads: Vec<FeatureId>,
    /// Light features used within this region.
    pub lights: Vec<FeatureId>,
    /// Local point-light features placed in this region.
    pub point_lights: Vec<FeatureId>,
    /// Local flicker/cookie programs placed in this region.
    pub point_programs: Vec<FeatureId>,
    /// Authored activation sets of local point lights.
    pub point_light_sets: Vec<Vec<FeatureId>>,
    /// Authored activation sets of local point-light programs.
    pub point_program_sets: Vec<Vec<FeatureId>>,
    /// Visibility connections through world portals.
    pub portals: Vec<PortalConnection>,
    /// Environmental fog volumes used within this region.
    pub fogs: Vec<FogId>,
    /// Independently authored embedded baked-light images.
    pub lightmap_images: Vec<crate::model::semantic::TextureId>,
    /// Baked-light texel rectangles; selector relationships are not inferred.
    pub lightmap_rectangles: ZeroVec<'a, [u8; 4]>,
    /// Historical interior vertex/draw windows, retained as individual numeric selections.
    pub interior_windows: Option<[[u16; 4]; 4]>,
    /// Historical exterior vertex/draw windows, retained as individual numeric selections.
    pub exterior_windows: Option<[[u16; 4]; 4]>,
    /// Historical independently authored geometry queries and their unresolved windows.
    pub auxiliary_geometry: Option<AuxiliaryGeometry<'a>>,
    /// Ambient illumination volumes selected for this region.
    pub ambients: Vec<AmbientId>,
    /// Liquid category of an implicitly filled region without an authored surface grid.
    pub implicit_liquid: Option<u32>,
    /// Authored liquid grids within this region.
    pub liquids: Vec<Liquid<'a>>,
    /// Supplemental batch classification control whose meaning is unresolved.
    pub batch_control: u16,
    /// Additional region control words whose meanings are unresolved.
    pub supplemental_controls: Vec<u32>,
    /// Authored terrain-cutting planes in world coordinates.
    pub terrain_cut_planes: ZeroVec<'a, [f32; 4]>,
    /// Optional region ambient color override in RGBA order.
    pub ambient_override: Option<[u8; 4]>,
    /// Optional detail-doodad layer selection.
    pub detail_doodad_key: Option<u8>,
}
/// Historical auxiliary geometry streams with no inferred runtime interpretation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AuxiliaryGeometry<'a> {
    /// Index selections into the auxiliary geometry stream.
    pub indices: ZeroVec<'a, u16>,
    /// Three-component authored auxiliary values.
    pub values: ZeroVec<'a, [f32; 3]>,
    /// Start/count selections associated with vertices.
    pub vertex_windows: ZeroVec<'a, [u16; 2]>,
    /// Start/count selections associated with planes.
    pub plane_windows: ZeroVec<'a, [u16; 2]>,
}

/// Complete static world authoring data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldSystem<'a> {
    /// Authored world groups in stable local identity order.
    pub groups: Vec<Group<'a>>,
    /// Portal boundary polygons and culling planes.
    pub portals: Vec<Portal<'a>>,
    /// Visibility vertex runs retained as logical polygons.
    pub visibility: Vec<ZeroVec<'a, [f32; 3]>>,
    /// Occluder plane equations.
    pub occluders: ZeroVec<'a, [f32; 4]>,
    /// Doodad sets and their explicit membership.
    pub doodad_sets: Vec<DoodadSet<'a>>,
    /// Spherical and plane-bounded fog volumes.
    pub fogs: Vec<Fog>,
    /// Spherical and plane-bounded ambient-light volumes.
    pub ambient_volumes: Vec<Ambient>,
}

/// Model-wide world rendering and environment controls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelMetadata<'a> {
    /// Optional background sky model resource.
    pub skybox: Option<Resource<'a>>,
    /// External world-model identifier.
    pub id: u32,
    /// World ambient color in RGBA order.
    pub ambient: [u8; 4],
    /// Root rendering and illumination behavior.
    pub options: WorldOptions,
}
impl crate::model::semantic::ValidateLinks for ModelMetadata<'_> {
    fn visit_links(&self, _visitor: &mut dyn FnMut(crate::model::semantic::Link)) {}
}

mod vocabulary;
pub use vocabulary::Wmo;

mod geometry;
pub use geometry::*;
