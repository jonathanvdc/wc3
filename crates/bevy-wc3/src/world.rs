//! Owned static-world catalogs, conservative visibility, and authored liquid meshes.
use bevy::asset::RenderAssetUsages;
use bevy::math::Vec3;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use wc3::model::controls::{
    world::{AmbientId, DoodadSetId, FogId, GroupId, PortalId},
    InstanceControls,
};
use wc3::model::scene::world::{
    Ambient, Fog, GroupOptions, Liquid, LiquidVertex, PortalConnection, Volume,
};
use wc3::model::systems::{ModelMetadata, System};
use wc3::model::{FeatureId, InstanceId, MaterialId, Model, NodeId, SystemId, Texture};

use super::Playback;
use crate::ModelError;

/// Explicit portal policy; the default keeps every potentially visible group.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum WorldPortalPolicy {
    /// Draw all groups, relying on Bevy's geometry frustum culling.
    #[default]
    Conservative,
    /// Restrict a system to groups connected to an application-selected group.
    /// This is a reachability filter, not an occlusion or clipped-frustum solver.
    ReachableFrom(GroupId),
    /// Traverse convex portal windows from an explicit application-owned view.
    /// Exterior and unclassified groups remain conservative.
    Clipped(PortalView),
}

/// One application's chosen portal-culling view, in semantic model coordinates.
///
/// Supply the eye and frustum of the camera whose visibility should drive this
/// instance. Update the selection when that view moves. This policy also works
/// for baking; it does not choose a global Bevy camera or modify scene lighting.
#[derive(Clone, Debug, PartialEq)]
pub struct PortalView {
    group: GroupId,
    eye: Vec3,
    planes: Arc<[Vec4]>,
}

impl PortalView {
    /// Construct a view with inward-facing planes: `normal.dot(point) + w >= 0`.
    /// Empty planes admit every initial direction. Planes are normalized once;
    /// nonfinite eyes/planes and zero-length normals are rejected.
    pub fn new(
        group: GroupId,
        eye: Vec3,
        planes: impl IntoIterator<Item = Vec4>,
    ) -> Result<Self, ModelError> {
        if !eye.is_finite() {
            return Err(ModelError("portal view eye must be finite".into()));
        }
        let planes = planes
            .into_iter()
            .map(|plane| {
                let length = plane.truncate().length();
                if !plane.is_finite() || !length.is_finite() || length <= f32::EPSILON {
                    return Err(ModelError(
                        "portal view planes need finite nonzero normals".into(),
                    ));
                }
                let normalized = plane / length;
                if !normalized.is_finite() {
                    return Err(ModelError("normalized portal plane must be finite".into()));
                }
                Ok(normalized)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            group,
            eye,
            planes: planes.into(),
        })
    }
    /// Explicit group containing the driving eye; no containment guess is made.
    pub fn group(&self) -> GroupId {
        self.group
    }
    /// Driving eye in semantic model coordinates.
    pub fn eye(&self) -> Vec3 {
        self.eye
    }
    /// Normalized inward-facing halfspaces in semantic model coordinates.
    pub fn planes(&self) -> &[Vec4] {
        &self.planes
    }
}

/// Application selection for world groups and external placement sets.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct WorldSelection {
    /// Explicitly suppressed groups.
    pub hidden_groups: HashSet<GroupId>,
    /// Active placement sets; the default contains only the common set zero.
    /// `None` explicitly includes all sets, including mutually exclusive variants.
    /// Explicit selections always retain the common placement set.
    pub doodad_sets: Option<HashSet<DoodadSetId>>,
    /// Conservative default or explicit connectivity-based filtering.
    pub portals: WorldPortalPolicy,
}

impl Default for WorldSelection {
    fn default() -> Self {
        Self {
            hidden_groups: HashSet::new(),
            doodad_sets: Some(HashSet::new()),
            portals: WorldPortalPolicy::default(),
        }
    }
}

/// Named static-world visibility group in model coordinates.
#[derive(Clone, Debug)]
pub struct WorldGroup {
    /// Semantic group identity.
    pub id: GroupId,
    /// Authored display name.
    pub name: String,
    /// Associated geometry placements, across authored detail levels.
    pub instances: Arc<[InstanceId]>,
    /// Conservative bounds, before the model coordinate-frame transform.
    pub bounds: [Vec3; 2],
    /// Authored interior/exterior and baked-light controls.
    pub options: GroupOptions,
    /// Supplemental authored detail-level selection, when provided.
    pub detail_selection: Option<u32>,
}

/// Portal data retained for application visibility solvers.
#[derive(Clone, Debug)]
pub struct WorldPortal {
    /// Semantic portal identity.
    pub id: PortalId,
    /// Boundary polygon in model coordinates.
    pub vertices: Arc<[Vec3]>,
    /// Authored plane equation.
    pub plane: Vec4,
    /// Unresolved supplemental adjustment retained without applying a guess.
    pub adjustment: Option<[u32; 3]>,
}

/// Named external-placement selection set.
#[derive(Clone, Debug)]
pub struct DoodadSet {
    /// Stable set identity.
    pub id: DoodadSetId,
    /// Authored name.
    pub name: String,
    /// External placement feature identities.
    pub members: Arc<[FeatureId]>,
}

/// Environment data selected by a region; applications retain scene ownership.
#[derive(Clone, Debug)]
pub struct WorldRegion {
    /// Geometry placement owning this region.
    pub instance: InstanceId,
    /// Placement hierarchy node.
    pub node: NodeId,
    /// Authored region description.
    pub description: String,
    /// Authored region visibility and lighting options.
    pub options: GroupOptions,
    /// Referenced environmental fogs.
    pub fogs: Arc<[FogId]>,
    /// Referenced ambient-light volumes.
    pub ambients: Arc<[AmbientId]>,
    /// Explicit region-level ambient override.
    pub ambient_override: Option<[u8; 4]>,
    /// Placement features visible within this region.
    pub doodads: Arc<[FeatureId]>,
    /// Region-local authored light references.
    pub lights: Arc<[FeatureId]>,
    /// Supplemental point-light references.
    pub point_lights: Arc<[FeatureId]>,
    /// Supplemental light-program references.
    pub point_programs: Arc<[FeatureId]>,
    /// Baked lightmap textures retained for an application interpreter.
    pub lightmap_images: Arc<[Texture]>,
    /// Authored packed lightmap rectangle controls.
    pub lightmap_rectangles: Arc<[[u8; 4]]>,
    /// Authored outgoing portal relationships, preserving side/adjustment controls.
    pub portals: Arc<[PortalConnection]>,
}

/// Generated static liquid surface, with resource and region identities intact.
pub(crate) struct WorldLiquidMesh {
    pub instance: InstanceId,
    pub node: NodeId,
    pub material: Option<MaterialId>,
    pub liquid_type: u32,
    pub ordinal: usize,
    pub mesh: Mesh,
}

/// Semantic identity of a generated liquid entity, independent of mesh draw IDs.
#[derive(Component, Clone, Copy, Debug)]
pub struct WorldLiquidSurface {
    /// Owning world geometry placement.
    pub instance: InstanceId,
    /// Liquid ordinal within that region.
    pub ordinal: usize,
    /// Authored external liquid-type selector.
    pub liquid_type: u32,
    /// Selected authored material; absent requires an application material provider.
    pub material: Option<MaterialId>,
}

/// Authored grid controls retained independently of generated render geometry.
#[derive(Clone, Debug)]
pub struct WorldLiquid {
    /// Owning geometry region.
    pub instance: InstanceId,
    /// Grid ordinal within that region.
    pub ordinal: usize,
    /// External liquid-type database selector.
    pub liquid_type: u32,
    /// Authored grid vertex dimensions.
    pub dimensions: [u32; 2],
    /// Authored surface material, when supplied by the asset.
    pub material: Option<MaterialId>,
    /// Original model-space origin; vertex heights are absolute elevations.
    pub position: Vec3,
    /// Native WMO horizontal grid spacing used by the generated mesh.
    pub grid_spacing: Vec2,
    /// Original per-vertex flow or magma coordinate declarations.
    pub vertices: Arc<[LiquidVertex]>,
    /// Authored tile visibility and category bytes.
    pub tiles: Arc<[u8]>,
}

/// Authored environmental catalog, available without modifying global scene state.
#[derive(Clone, Debug)]
pub struct WorldEnvironment {
    /// Owning static-world system.
    pub system: SystemId,
    /// Fog volumes in stable authored order.
    pub fogs: Arc<[Fog]>,
    /// Ambient-light regions in stable authored order.
    pub ambients: Arc<[Ambient]>,
    /// Authored visibility boundary polygons for application culling solvers.
    pub visibility_polygons: Arc<[Arc<[Vec3]>]>,
    /// Authored occluder plane equations.
    pub occluders: Arc<[Vec4]>,
}

/// Authored regional influences for an application's fog/lighting integration.
#[derive(Clone, Debug, Default)]
pub struct WorldEnvironmentSample {
    /// Fog identities and boundary influence in authored order.
    pub fogs: Vec<(FogId, f32)>,
    /// Ambient identities and boundary influence in authored order.
    pub ambients: Vec<(AmbientId, f32)>,
}

/// Detached WMO catalogs and precomputed selection relationships.
#[derive(Clone, Default)]
pub(crate) struct WorldRuntime {
    pub ambient: Option<[u8; 4]>,
    pub groups: Arc<[WorldGroup]>,
    pub portals: Arc<[WorldPortal]>,
    pub doodad_sets: Arc<[DoodadSet]>,
    pub regions: Arc<[WorldRegion]>,
    pub environments: Arc<[WorldEnvironment]>,
    pub liquid_definitions: Arc<[WorldLiquid]>,
    memberships: HashMap<InstanceId, Vec<GroupId>>,
    feature_sets: HashMap<FeatureId, Vec<DoodadSetId>>,
    feature_groups: HashMap<FeatureId, Vec<GroupId>>,
    neighbors: HashMap<GroupId, Vec<GroupId>>,
    portal_links: HashMap<GroupId, Vec<PortalConnection>>,
    portal_indices: HashMap<PortalId, usize>,
    portal_groups: HashMap<SystemId, PortalGroups>,
    placements: Vec<(InstanceId, NodeId, Option<u32>)>,
    feature_nodes: Vec<(FeatureId, NodeId)>,
}

/// Immutable group lookup and uncullable baseline, shared by all camera updates.
#[derive(Clone, Default)]
struct PortalGroups {
    options: HashMap<GroupId, GroupOptions>,
    conservative: HashSet<GroupId>,
}

pub(crate) struct CompiledWorld {
    pub runtime: WorldRuntime,
    pub liquids: Vec<WorldLiquidMesh>,
    pub diagnostics: Vec<String>,
}

impl WorldRuntime {
    pub(crate) fn lod_levels(&self) -> impl Iterator<Item = u32> + '_ {
        self.placements.iter().filter_map(|(_, _, lod)| *lod)
    }
    pub(crate) fn minimum_lod(&self) -> Option<u32> {
        self.lod_levels().min()
    }
    pub(crate) fn environment_at(
        &self,
        position: Vec3,
        selection: &WorldSelection,
        region: Option<InstanceId>,
    ) -> WorldEnvironmentSample {
        let set_visible = |set: DoodadSetId| {
            set.index == 0
                || selection
                    .doodad_sets
                    .as_ref()
                    .is_none_or(|sets| sets.contains(&set))
        };
        let region = region.and_then(|id| self.regions.iter().find(|region| region.instance == id));
        let mut sample = WorldEnvironmentSample::default();
        for environment in self.environments.iter() {
            for (index, fog) in environment.fogs.iter().enumerate() {
                let id = FogId {
                    system: environment.system,
                    index: index as u32,
                };
                if region.is_some_and(|region| !region.fogs.contains(&id))
                    || fog.doodad_set.is_some_and(|set| !set_visible(set))
                {
                    continue;
                }
                let weight = if fog.infinite {
                    1.
                } else {
                    world_volume_weight(&fog.volume, position)
                };
                if weight > 0. {
                    sample.fogs.push((id, weight));
                }
            }
            for (index, ambient) in environment.ambients.iter().enumerate() {
                let id = AmbientId {
                    system: environment.system,
                    index: index as u32,
                };
                if region.is_some_and(|region| !region.ambients.contains(&id))
                    || ambient.doodad_set.is_some_and(|set| !set_visible(set))
                {
                    continue;
                }
                let weight = if ambient.global {
                    1.
                } else {
                    world_volume_weight(&ambient.volume, position)
                };
                if weight > 0. {
                    sample.ambients.push((id, weight));
                }
            }
        }
        sample
    }
    pub(crate) fn compile(model: &Model<'_>) -> Result<CompiledWorld, ModelError> {
        let mut runtime = Self {
            ambient: match model.metadata() {
                ModelMetadata::Wmo(metadata) => Some(metadata.ambient),
                _ => None,
            },
            ..Default::default()
        };
        let mut groups = Vec::new();
        let mut portals = Vec::new();
        let mut sets = Vec::new();
        let mut environments = Vec::new();
        let mut diagnostics = Vec::new();
        for (system, payload) in model.systems() {
            let System::Wmo(world) = payload else {
                continue;
            };
            for (index, group) in world.groups.iter().enumerate() {
                let id = GroupId {
                    system,
                    index: index as u32,
                };
                for &instance in &group.instances {
                    runtime.memberships.entry(instance).or_default().push(id);
                }
                runtime.neighbors.entry(id).or_default();
                groups.push(WorldGroup {
                    id,
                    name: group.name.to_string(),
                    instances: group.instances.clone().into(),
                    bounds: group.bounds,
                    options: group.options,
                    detail_selection: group.detail_selection,
                });
            }
            for (index, portal) in world.portals.iter().enumerate() {
                portals.push(WorldPortal {
                    id: PortalId {
                        system,
                        index: index as u32,
                    },
                    vertices: portal.vertices.iter().collect::<Vec<_>>().into(),
                    plane: portal.plane,
                    adjustment: portal.adjustment,
                });
            }
            for (index, set) in world.doodad_sets.iter().enumerate() {
                let id = DoodadSetId {
                    system,
                    index: index as u32,
                };
                for &feature in &set.members {
                    runtime.feature_sets.entry(feature).or_default().push(id);
                }
                sets.push(DoodadSet {
                    id,
                    name: set.name.to_string(),
                    members: set.members.clone().into(),
                });
            }
            environments.push(WorldEnvironment {
                system,
                fogs: world.fogs.clone().into(),
                ambients: world.ambient_volumes.clone().into(),
                visibility_polygons: world
                    .visibility
                    .iter()
                    .map(|polygon| polygon.iter().collect::<Vec<_>>().into())
                    .collect::<Vec<_>>()
                    .into(),
                occluders: world.occluders.iter().collect::<Vec<_>>().into(),
            });
            if !world.portals.is_empty() {
                diagnostics.push(format!("world system {system:?}: portal polygons and connectivity retained; default visibility is conservative, explicit PortalView enables clipped-window traversal; native occluder planes are not evaluated"));
            }
            if !world.fogs.is_empty() || !world.ambient_volumes.is_empty() {
                diagnostics.push(format!("world system {system:?}: fog and ambient regions are exposed for application integration; global scene fog/ambient are not overwritten"));
            }
        }
        let mut regions = Vec::new();
        let mut liquid_definitions = Vec::new();
        let mut liquids = Vec::new();
        for (instance, placement) in model.instances() {
            if runtime.memberships.contains_key(&instance)
                || matches!(&placement.controls, Some(InstanceControls::Wmo(_)))
            {
                runtime
                    .placements
                    .push((instance, placement.node, placement.lod));
            }
            let Some(InstanceControls::Wmo(region)) = &placement.controls else {
                continue;
            };
            for &group in runtime.memberships.get(&instance).into_iter().flatten() {
                for &feature in region
                    .doodads
                    .iter()
                    .chain(&region.lights)
                    .chain(&region.point_lights)
                    .chain(&region.point_programs)
                {
                    runtime
                        .feature_groups
                        .entry(feature)
                        .or_default()
                        .push(group);
                }
                runtime
                    .portal_links
                    .entry(group)
                    .or_default()
                    .extend(region.portals.iter().cloned());
                for portal in &region.portals {
                    // Bidirectional reachability deliberately overestimates visibility.
                    // Side tests and unknown supplemental adjustments cannot safely
                    // hide a connected group without a camera/frustum solver.
                    runtime
                        .neighbors
                        .entry(group)
                        .or_default()
                        .push(portal.neighbor);
                    runtime
                        .neighbors
                        .entry(portal.neighbor)
                        .or_default()
                        .push(group);
                }
            }
            regions.push(WorldRegion {
                instance,
                node: placement.node,
                description: region.description.to_string(),
                options: region.options,
                fogs: region.fogs.clone().into(),
                ambients: region.ambients.clone().into(),
                ambient_override: region.ambient_override,
                doodads: region.doodads.clone().into(),
                lights: region.lights.clone().into(),
                point_lights: region.point_lights.clone().into(),
                point_programs: region.point_programs.clone().into(),
                lightmap_images: region.lightmap_images.clone().into(),
                lightmap_rectangles: region.lightmap_rectangles.iter().collect::<Vec<_>>().into(),
                portals: region.portals.clone().into(),
            });
            for (ordinal, liquid) in region.liquids.iter().enumerate() {
                liquid_definitions.push(WorldLiquid {
                    instance,
                    ordinal,
                    liquid_type: liquid.liquid_type,
                    dimensions: liquid.dimensions,
                    material: liquid.material,
                    position: liquid.position,
                    grid_spacing: Vec2::splat(25.0 / 6.0),
                    vertices: liquid.vertices.iter().collect::<Vec<_>>().into(),
                    tiles: liquid.tiles.iter().collect::<Vec<_>>().into(),
                });
                if let Some(mesh) = liquid_mesh(liquid, Vec2::splat(25.0 / 6.0))? {
                    liquids.push(WorldLiquidMesh {
                        instance,
                        node: placement.node,
                        material: liquid.material,
                        liquid_type: liquid.liquid_type,
                        ordinal,
                        mesh,
                    });
                }
                diagnostics.push(format!("world region {instance:?} liquid {ordinal}: authored heights/tile holes generate geometry at native WMO grid spacing; flow, database shading and animated liquid response need an application interpreter"));
                if liquid.material.is_none() {
                    diagnostics.push(format!("world region {instance:?} liquid {ordinal}: liquid type {} requires a material provider",liquid.liquid_type));
                }
            }
            if !region.lightmap_images.is_empty() || !region.lightmap_rectangles.is_empty() {
                diagnostics.push(format!("world region {instance:?}: baked lightmap resources/rectangles are exposed; lightmap shading requires an application interpreter"));
            }
            if region.implicit_liquid.is_some() {
                diagnostics.push(format!(
                    "world region {instance:?}: implicit liquid has no authored surface grid"
                ));
            }
        }
        for neighbors in runtime.neighbors.values_mut() {
            neighbors.sort();
            neighbors.dedup();
        }
        runtime.feature_nodes = model
            .features()
            .iter()
            .filter_map(|(id, feature)| {
                (runtime.feature_sets.contains_key(&id) || runtime.feature_groups.contains_key(&id))
                    .then_some((id, feature.node))
            })
            .collect();
        runtime.groups = groups.into();
        runtime.index_portal_groups();
        runtime.portal_indices = portals
            .iter()
            .enumerate()
            .map(|(index, portal)| (portal.id, index))
            .collect();
        runtime.portals = portals.into();
        runtime.doodad_sets = sets.into();
        runtime.regions = regions.into();
        runtime.environments = environments.into();
        runtime.liquid_definitions = liquid_definitions.into();
        Ok(CompiledWorld {
            runtime,
            liquids,
            diagnostics,
        })
    }

    /// Resolve an explicit policy once per instance/update, not once per draw.
    pub(crate) fn visibility(&self, selection: &WorldSelection) -> WorldVisibility {
        let reachable = if let WorldPortalPolicy::ReachableFrom(start) = &selection.portals {
            let start = *start;
            if self.neighbors.contains_key(&start) {
                let mut visited = HashSet::from([start]);
                let mut pending = VecDeque::from([start]);
                while let Some(group) = pending.pop_front() {
                    for &neighbor in &self.neighbors[&group] {
                        if visited.insert(neighbor) {
                            pending.push_back(neighbor);
                        }
                    }
                }
                Some((start.system, visited))
            } else {
                None
            } // Unknown application selections cannot hide the whole model.
        } else if let WorldPortalPolicy::Clipped(view) = &selection.portals {
            self.clipped_visibility(view)
                .map(|groups| (view.group.system, groups))
        } else {
            None
        };
        WorldVisibility {
            hidden: selection.hidden_groups.clone(),
            sets: selection.doodad_sets.clone(),
            reachable,
        }
    }
    fn index_portal_groups(&mut self) {
        self.portal_groups.clear();
        for group in self.groups.iter() {
            let system = self.portal_groups.entry(group.id.system).or_default();
            system.options.insert(group.id, group.options);
            if !group.options.interior || group.options.exterior {
                system.conservative.insert(group.id);
            }
        }
    }

    /// A bounded, per-path traversal: a group can be reached through several
    /// windows, each of which may expose different outgoing portals.
    fn clipped_visibility(&self, view: &PortalView) -> Option<HashSet<GroupId>> {
        const MAX_WINDOWS: usize = 4096;
        const MAX_PLANES: usize = 128;
        const MAX_DEPTH: usize = 128;
        const MAX_CONNECTIONS: usize = 32768;
        let groups = self.portal_groups.get(&view.group.system)?;
        let start = groups.options.get(&view.group)?;
        if !start.interior || start.exterior {
            return None;
        }
        let mut visible = groups.conservative.clone();
        let mut pending = vec![(view.group, view.planes.to_vec(), vec![view.group])];
        let mut windows = 0;
        let mut connections = 0;
        while let Some((group, planes, path)) = pending.pop() {
            windows += 1;
            if windows > MAX_WINDOWS || planes.len() > MAX_PLANES || path.len() > MAX_DEPTH {
                return None; // Work limits must never turn into missing geometry.
            }
            visible.insert(group);
            let links = self
                .portal_links
                .get(&group)
                .map(Vec::as_slice)
                .unwrap_or_default();
            for connection in links {
                connections += 1;
                if connections > MAX_CONNECTIONS || pending.len() >= MAX_WINDOWS {
                    return None;
                }
                if path.contains(&connection.neighbor) {
                    continue;
                }
                if !groups.options.contains_key(&connection.neighbor) {
                    return None;
                }
                // Unresolved plane adjustments and query flags remain open;
                // authored side signs are not guessed from client conventions.
                let clipped = if connection.adjusted_plane
                    || connection.skip_viewer
                    || connection.unclassified.bits() != 0
                {
                    Some(planes.clone())
                } else {
                    self.portal_indices
                        .get(&connection.portal)
                        .and_then(|&index| self.portals.get(index))
                        .map_or_else(
                            || Some(planes.clone()),
                            |portal| {
                                portal_window(&portal.vertices, view.eye, &planes).map_or_else(
                                    || Some(planes.clone()),
                                    |polygon| {
                                        if polygon.is_empty() {
                                            None
                                        } else {
                                            Some(window_planes(&polygon, view.eye, &planes))
                                        }
                                    },
                                )
                            },
                        )
                };
                if let Some(clipped) = clipped {
                    let mut next_path = path.clone();
                    next_path.push(connection.neighbor);
                    pending.push((connection.neighbor, clipped, next_path));
                }
            }
            // Incomplete native links retain the conservative connectivity path.
            for &neighbor in self.neighbors.get(&group).into_iter().flatten() {
                connections += 1;
                if connections > MAX_CONNECTIONS || pending.len() >= MAX_WINDOWS {
                    return None;
                }
                if links.iter().any(|link| link.neighbor == neighbor) || path.contains(&neighbor) {
                    continue;
                }
                if !groups.options.contains_key(&neighbor) {
                    return None;
                }
                let mut next_path = path.clone();
                next_path.push(neighbor);
                pending.push((neighbor, planes.clone(), next_path));
            }
        }
        Some(visible)
    }
    pub(crate) fn instance_lod(&self, instance: InstanceId) -> Option<u32> {
        self.placements
            .iter()
            .find(|(candidate, _, _)| *candidate == instance)
            .and_then(|(_, _, lod)| *lod)
    }
    pub(crate) fn instance_visible(
        &self,
        instance: InstanceId,
        selection: &WorldVisibility,
    ) -> bool {
        self.memberships
            .get(&instance)
            .is_none_or(|groups| groups.iter().any(|group| selection.group_visible(*group)))
    }
    pub(crate) fn feature_visible(&self, feature: FeatureId, selection: &WorldVisibility) -> bool {
        self.feature_groups
            .get(&feature)
            .is_none_or(|groups| groups.iter().any(|group| selection.group_visible(*group)))
            && self
                .feature_sets
                .get(&feature)
                .is_none_or(|sets| sets.iter().any(|set| selection.set_visible(*set)))
    }
}

// The tolerance expands accepted boundaries, preventing floating-point cracks.
const PORTAL_EPSILON: f32 = 1.0e-5;

/// None means malformed/ambiguous geometry; Some(empty) means safely clipped out.
fn portal_window(vertices: &[Vec3], eye: Vec3, planes: &[Vec4]) -> Option<Vec<Vec3>> {
    if vertices.len() < 3 || vertices.len() > 64 {
        return None;
    }
    let mut polygon: Vec<_> = vertices.to_vec();
    if polygon.iter().any(|point| !point.is_finite()) {
        return None;
    }
    let origin = polygon[0];
    let normal = (1..polygon.len() - 1).find_map(|index| {
        (polygon[index] - origin)
            .cross(polygon[index + 1] - origin)
            .try_normalize()
    })?;
    let eye_distance = normal.dot(eye - origin);
    if polygon.iter().any(|point| {
        let distance = normal.dot(*point - origin);
        !distance.is_finite() || distance.abs() > PORTAL_EPSILON
    }) || !eye_distance.is_finite()
        || eye_distance.abs() <= PORTAL_EPSILON
    {
        return None;
    }
    // Every edge must contain the remaining polygon in one consistent halfspace.
    // This rejects concave/self-crossing windows instead of underestimating them.
    let mut winding = 0.0_f32;
    for index in 0..polygon.len() {
        let a = polygon[index];
        let edge = polygon[(index + 1) % polygon.len()] - a;
        if !edge.is_finite()
            || !edge.length_squared().is_finite()
            || edge.length_squared() <= PORTAL_EPSILON * PORTAL_EPSILON
        {
            return None;
        }
        for point in &polygon {
            let side = normal.dot(edge.cross(*point - a));
            if !side.is_finite() {
                return None;
            }
            if side.abs() <= PORTAL_EPSILON {
                continue;
            }
            if winding == 0.0 {
                winding = side.signum();
            } else if side * winding < -PORTAL_EPSILON {
                return None;
            }
        }
    }
    for plane in planes {
        if polygon.is_empty() {
            break;
        }
        let input = polygon;
        polygon = Vec::with_capacity(input.len() + 1);
        let mut previous = *input.last()?;
        let mut previous_distance = plane.truncate().dot(previous) + plane.w + PORTAL_EPSILON;
        if !previous_distance.is_finite() {
            return None;
        }
        for current in input {
            let distance = plane.truncate().dot(current) + plane.w + PORTAL_EPSILON;
            if !distance.is_finite() {
                return None;
            }
            if (distance >= 0.0) != (previous_distance >= 0.0) {
                let denominator = previous_distance - distance;
                if !denominator.is_finite() {
                    return None;
                }
                let fraction = previous_distance / denominator;
                let intersection = previous.lerp(current, fraction);
                if !intersection.is_finite() {
                    return None;
                }
                polygon.push(intersection);
            }
            if distance >= 0.0 {
                polygon.push(current);
            }
            previous = current;
            previous_distance = distance;
        }
    }
    Some(polygon)
}

fn window_planes(polygon: &[Vec3], eye: Vec3, parent: &[Vec4]) -> Vec<Vec4> {
    let mut planes = parent.to_vec();
    if polygon.len() < 3 {
        return planes;
    }
    let center = polygon.iter().copied().sum::<Vec3>() / polygon.len() as f32;
    if !center.is_finite() {
        return planes;
    }
    let has_area = (1..polygon.len() - 1).any(|index| {
        let area = (polygon[index] - polygon[0]).cross(polygon[index + 1] - polygon[0]);
        area.is_finite() && area.length_squared() > PORTAL_EPSILON * PORTAL_EPSILON
    });
    if !has_area {
        return planes;
    }
    for index in 0..polygon.len() {
        let a = polygon[index] - eye;
        let b = polygon[(index + 1) % polygon.len()] - eye;
        let Some(mut normal) = a.cross(b).try_normalize() else {
            continue;
        };
        if normal.dot(center - eye) < 0.0 {
            normal = -normal;
        }
        let plane = normal.extend(-normal.dot(eye));
        if plane.is_finite() {
            planes.push(plane);
        }
    }
    planes
}

pub(crate) struct WorldVisibility {
    hidden: HashSet<GroupId>,
    sets: Option<HashSet<DoodadSetId>>,
    reachable: Option<(SystemId, HashSet<GroupId>)>,
}
impl WorldVisibility {
    pub(crate) fn group_visible(&self, group: GroupId) -> bool {
        !self.hidden.contains(&group)
            && self
                .reachable
                .as_ref()
                .is_none_or(|(system, groups)| group.system != *system || groups.contains(&group))
    }
    pub(crate) fn set_visible(&self, set: DoodadSetId) -> bool {
        set.index == 0 || self.sets.as_ref().is_none_or(|sets| sets.contains(&set))
    }
}

/// Region response at a point, expressed as an influence rather than scene mutation.
/// Plane volumes use nonpositive interior half-spaces; boundary fades extend inward.
pub fn world_volume_weight(volume: &Volume, position: Vec3) -> f32 {
    match volume {
        Volume::Sphere { center, radii } => {
            let distance = position.distance(*center);
            if distance <= radii[0] {
                1.
            } else if radii[1] <= radii[0] {
                0.
            } else {
                ((radii[1] - distance) / (radii[1] - radii[0])).clamp(0., 1.)
            }
        }
        Volume::Planes { planes, fade } => {
            let margin = planes
                .iter()
                .map(|plane| {
                    let normal = Vec3::new(plane[0], plane[1], plane[2]);
                    let length = normal.length();
                    if length > 0. {
                        -(normal.dot(position) + plane[3]) / length
                    } else {
                        f32::INFINITY
                    }
                })
                .fold(f32::INFINITY, f32::min);
            if *fade > 0. {
                (margin / fade).clamp(0., 1.)
            } else {
                u8::from(margin >= 0.) as f32
            }
        }
    }
}

fn liquid_mesh(liquid: &Liquid<'_>, spacing: Vec2) -> Result<Option<Mesh>, ModelError> {
    if spacing
        .to_array()
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.)
        || liquid
            .position
            .to_array()
            .iter()
            .any(|value| !value.is_finite())
        || liquid
            .vertices
            .iter()
            .any(|value| !value.height().is_finite())
    {
        return Err(ModelError(
            "world liquid coordinates must be finite with positive grid spacing".into(),
        ));
    }
    let [width, height] = liquid.dimensions.map(|v| v as usize);
    if width == 0 || height == 0 {
        return Ok(None);
    }
    if width.checked_mul(height) != Some(liquid.vertices.len())
        || width
            .saturating_sub(1)
            .checked_mul(height.saturating_sub(1))
            != Some(liquid.tiles.len())
    {
        return Err(ModelError("world liquid grid cardinality mismatch".into()));
    }
    let mut positions = Vec::new();
    let mut uv = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let vertex = liquid
                .vertices
                .get(y * width + x)
                .ok_or_else(|| ModelError("missing world liquid vertex".into()))?;
            positions.push([
                liquid.position[0] + x as f32 * spacing[0],
                liquid.position[1] + y as f32 * spacing[1],
                vertex.height(),
            ]);
            // Use tile coordinates until the external liquid shader supplies a
            // proven interpretation of flow/magma coordinate units.
            uv.push([x as f32, y as f32]);
        }
    }
    let mut indices = Vec::new();
    for y in 0..height - 1 {
        for x in 0..width - 1 {
            let tile = liquid
                .tiles
                .get(y * (width - 1) + x)
                .ok_or_else(|| ModelError("missing world liquid tile".into()))?;
            if tile & 0x8 != 0 {
                continue;
            }
            let a = (y * width + x) as u32;
            let b = a + 1;
            let d = a + width as u32;
            let c = d + 1;
            indices.extend([a, b, c, a, c, d]);
        }
    }
    if indices.is_empty() {
        return Ok(None);
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_normals();
    let _ = mesh.generate_tangents();
    Ok(Some(mesh))
}

/// Private visibility parents intersect world selection with authored animation.
#[derive(Component)]
pub(crate) struct WorldBindings {
    runtime: Arc<WorldRuntime>,
    gates: Vec<(Entity, WorldTarget)>,
    selection: Option<WorldSelection>,
    lod: Option<u32>,
}
#[derive(Clone, Copy)]
enum WorldTarget {
    Placement(InstanceId, Option<u32>),
    Feature(FeatureId),
}

pub(crate) struct WorldParents {
    pub placements: HashMap<InstanceId, Entity>,
    pub features: HashMap<FeatureId, Entity>,
}
impl WorldRuntime {
    pub(crate) fn spawn_gates(
        self: &Arc<Self>,
        commands: &mut Commands,
        root: Entity,
        nodes: &HashMap<NodeId, Entity>,
    ) -> WorldParents {
        let mut parents = WorldParents {
            placements: HashMap::new(),
            features: HashMap::new(),
        };
        let mut gates = Vec::new();
        for &(id, node, lod) in &self.placements {
            let gate = commands
                .spawn((
                    Name::new("World region visibility"),
                    Transform::default(),
                    Visibility::Inherited,
                    ChildOf(nodes[&node]),
                ))
                .id();
            parents.placements.insert(id, gate);
            gates.push((gate, WorldTarget::Placement(id, lod)));
        }
        for &(id, node) in &self.feature_nodes {
            let gate = commands
                .spawn((
                    Name::new("World placement visibility"),
                    Transform::default(),
                    Visibility::Inherited,
                    ChildOf(nodes[&node]),
                ))
                .id();
            parents.features.insert(id, gate);
            gates.push((gate, WorldTarget::Feature(id)));
        }
        commands.entity(root).remove::<WorldBindings>();
        if !gates.is_empty() {
            commands
                .entity(root)
                .insert(WorldBindings {
                    runtime: self.clone(),
                    gates,
                    selection: None,
                    lod: None,
                })
                .insert_if_new(WorldSelection::default());
        }
        parents
    }
}

/// Apply selection only when it or the authored detail level changes.
pub(crate) fn apply_world_selection(
    mut instances: Query<(&mut WorldBindings, &WorldSelection, &Playback)>,
    mut visibility: Query<&mut Visibility>,
) {
    for (mut bindings, selection, playback) in &mut instances {
        if bindings.selection.as_ref() == Some(selection) && bindings.lod == Some(playback.lod) {
            continue;
        }
        let resolved = bindings.runtime.visibility(selection);
        for &(entity, target) in &bindings.gates {
            let enabled = match target {
                WorldTarget::Placement(id, lod) => {
                    lod.is_none_or(|level| level == playback.lod)
                        && bindings.runtime.instance_visible(id, &resolved)
                }
                WorldTarget::Feature(id) => bindings.runtime.feature_visible(id, &resolved),
            };
            if let Ok(mut current) = visibility.get_mut(entity) {
                let value = if enabled {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if *current != value {
                    *current = value;
                }
            }
        }
        bindings.selection = Some(selection.clone());
        bindings.lod = Some(playback.lod);
    }
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;
