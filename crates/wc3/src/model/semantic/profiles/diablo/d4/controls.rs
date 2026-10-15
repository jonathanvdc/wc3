//! Owner-local Diablo IV declarations are independent of native record ordering.
use crate::model::semantic::{
    Bounds, Id, Link, MaterialSlotId, MeshId, NodeId, Scope, Transform, ValidateLinks,
};

/// Appearance-wide authoring declarations.
#[derive(Clone, Debug)]
pub struct ModelMetadata {
    scope: Scope,
    chunks: Vec<(ChunkId, Chunk)>,
    /// External appearance identity.
    pub sno_id: i32,
    /// Root authoring flags.
    pub record_flags: u32,
    /// Appearance options.
    pub flags: u32,
    /// Structure-wide options.
    pub structure_flags: u32,
    /// Radius of authored vertex animation.
    pub vertex_animation_radius: f32,
    /// Selected low-quality detail level.
    pub low_quality_lod: i32,
    /// Multiplier applied to authored LOD distances.
    pub lod_distance_multiplier: f32,
    /// Authored deformation scale.
    pub deformation_scale: f32,
    /// External fused-prefab resource.
    pub fused_prefab: i32,
    /// Authored look-variant selection.
    pub look_variant: u32,
    /// Independently authored structure bounding-sphere center.
    pub structure_sphere_center: [f32; 3],
    /// Independently authored structure bounding-sphere radius.
    pub structure_sphere_radius: f32,
}
impl Default for ModelMetadata {
    fn default() -> Self {
        Self {
            scope: Scope::default(),
            chunks: Vec::new(),
            sno_id: -1,
            record_flags: 0,
            flags: 0,
            structure_flags: 0,
            vertex_animation_radius: 0.,
            low_quality_lod: 0,
            lod_distance_multiplier: 1.,
            deformation_scale: 1.,
            fused_prefab: -1,
            look_variant: 0,
            structure_sphere_center: [0.; 3],
            structure_sphere_radius: 0.,
        }
    }
}
impl ValidateLinks for ModelMetadata {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        for (_, chunk) in &self.chunks {
            if chunk.bounds.validate().is_err()
                || chunk.lod_distances.iter().any(|v| !v.is_finite())
            {
                visitor(Link::InvalidValue);
            }
        }
        if self.structure_sphere_center.iter().any(|v| !v.is_finite())
            || !self.structure_sphere_radius.is_finite()
            || self.structure_sphere_radius < 0.
            || [
                self.vertex_animation_radius,
                self.lod_distance_multiplier,
                self.deformation_scale,
            ]
            .iter()
            .any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
/// Independent authored bone bind poses and identity declarations.
#[derive(Clone, Debug)]
pub struct BoneControls {
    /// Authored bone name hash.
    pub hash: u32,
    /// Bone behavior flags.
    pub flags: u32,
    /// Bone detail level.
    pub lod: u16,
    /// Authored true-bone detail level.
    pub true_bone_lod: u16,
    /// Independent unresolved bone control.
    pub unclassified_control: u16,
    /// External particle-system resource.
    pub particle_system: i32,
    /// Authored bounding sphere center.
    pub sphere_center: [f32; 3],
    /// Authored bounding sphere radius.
    pub sphere_radius: f32,
    /// Model-space bind pose, independently authored from the local node transform.
    pub bind_pose: Transform,
    /// Independently authored inverse bind transform.
    pub inverse_bind_pose: Transform,
    /// Independently authored inverse skinning transform.
    pub inverse_skinning_pose: Transform,
}
impl Default for BoneControls {
    fn default() -> Self {
        Self {
            hash: 0,
            flags: 0,
            lod: 0,
            true_bone_lod: 0,
            unclassified_control: 0,
            particle_system: -1,
            sphere_center: [0.; 3],
            sphere_radius: 0.,
            bind_pose: Transform::default(),
            inverse_bind_pose: Transform::default(),
            inverse_skinning_pose: Transform::default(),
        }
    }
}
impl ValidateLinks for BoneControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if [
            self.bind_pose,
            self.inverse_bind_pose,
            self.inverse_skinning_pose,
        ]
        .iter()
        .any(|t| t.validate().is_err() || t.pivot != [0.; 3])
            || self.sphere_center.iter().any(|v| !v.is_finite())
            || !self.sphere_radius.is_finite()
            || self.sphere_radius < 0.
        {
            visitor(Link::InvalidValue);
        }
    }
}
/// Hierarchy role distinguishing the implicit frame, bones, and named hardpoints.
#[derive(Clone, Debug, Default)]
pub enum NodeRole {
    /// Fixed model-space placement frame.
    #[default]
    ModelSpace,
    /// Authored skeleton bone.
    Bone(Box<BoneControls>),
    /// Authored attachment hash; its parent and transform belong to the node.
    Hardpoint {
        /// Hashed attachment name.
        hash: u32,
        /// Authored hardpoint hash options.
        hash_flags: u32,
        /// Native choice governing inherited orientation.
        ignore_parent_orientation: i32,
    },
}
impl ValidateLinks for NodeRole {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if let Self::Bone(value) = self {
            value.visit_links(visitor);
        }
    }
}
/// Semantic subobject identity and appearance-selection declarations.
#[derive(Clone, Debug, Default)]
pub struct NameInfo {
    /// Object classification, preserving unresolved selectors.
    pub kind: i32,
    /// Selection flags.
    pub flags: u32,
    /// Persona selector.
    pub persona: u8,
    /// State selector.
    pub state: u8,
    /// Style selector.
    pub style: u32,
    /// Equipment slot hash.
    pub slot_hash: u32,
    /// Non-player-character component hash.
    pub npc_component_hash: u32,
    /// Detail hash.
    pub detail_hash: u32,
    /// Three independently authored unresolved selectors.
    pub unclassified_controls: [u32; 3],
}
/// Behavior belonging to one logical subobject's geometry.
#[derive(Clone, Debug)]
pub struct MeshControls {
    /// Logical geometry chunk membership.
    pub chunk: Option<ChunkId>,
    /// Source subobject identity hash.
    pub subobject_hash: u32,
    /// Hash of the assigned appearance material slot.
    pub material_slot: Option<MaterialSlotId>,
    /// Subobject behavior flags.
    pub flags: u32,
    /// Appearance and equipment selectors.
    pub name_info: NameInfo,
    /// Actual base-detail geometry, when the authored declaration selects one.
    pub base_lod_subobject: Option<MeshId>,
    /// Highest authored detail level.
    pub max_lod: i32,
    /// External shader-map override selection.
    pub shader_map_override: u32,
    /// External campaign-visibility condition.
    pub campaign_visibility_condition: i32,
    /// Logical position offset used by fixed-point native encodings.
    pub fixed_point_position_offset: [f32; 3],
    /// Logical position scale used by fixed-point native encodings.
    pub fixed_point_position_scale: [f32; 3],
    /// Independent authored unresolved integer controls.
    pub unclassified_controls: [u32; 7],
    /// Authored subobject segmentation and local bone palettes.
    pub segments: Vec<Segment>,
}
impl Default for MeshControls {
    fn default() -> Self {
        Self {
            chunk: None,
            subobject_hash: 0,
            material_slot: None,
            flags: 0,
            name_info: NameInfo::default(),
            base_lod_subobject: None,
            max_lod: 0,
            shader_map_override: 0,
            campaign_visibility_condition: -1,
            fixed_point_position_offset: [0.; 3],
            fixed_point_position_scale: [1.; 3],
            unclassified_controls: [0; 7],
            segments: Vec::new(),
        }
    }
}
impl ValidateLinks for MeshControls {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        self.base_lod_subobject.visit_links(visitor);
        self.material_slot.visit_links(visitor);
        for segment in &self.segments {
            for id in &segment.bones {
                id.visit_links(visitor);
            }
        }
        if self
            .fixed_point_position_offset
            .iter()
            .chain(self.fixed_point_position_scale.iter())
            .any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}

/// One subobject segment's logical geometry window and actual bone palette.
#[derive(Clone, Debug, Default)]
pub struct Segment {
    /// First vertex in the subobject geometry.
    pub first_vertex: u32,
    /// Number of vertices addressed by this segment.
    pub vertex_count: u32,
    /// First triangle in the subobject connectivity.
    pub first_triangle: u32,
    /// Number of triangles addressed by this segment.
    pub triangle_count: u32,
    /// Ordered actual skeleton bones selected by the segment's local palette.
    pub bones: Vec<NodeId>,
}

/// Identity marker for a geometry chunk owned by one model's controls.
pub enum ChunkObject {}
/// Stable model-scoped geometry chunk identity, independent of native table ordering.
pub type ChunkId = Id<ChunkObject>;
/// Shared authoring declarations belonging to one actual geometry chunk.
#[derive(Clone, Debug, Default)]
pub struct Chunk {
    /// Independently authored chunk bounds.
    pub bounds: Bounds,
    /// Authored transition distance per logical detail level, in increasing level order.
    pub lod_distances: Vec<f32>,
}
impl ModelMetadata {
    /// Adds a geometry chunk without changing existing scoped identities.
    pub fn add_chunk(&mut self, chunk: Chunk) -> ChunkId {
        let id = self.scope.allocate();
        self.chunks.push((id, chunk));
        id
    }
    /// Iterates actual geometry chunks in authored order.
    pub fn chunks(&self) -> impl ExactSizeIterator<Item = (ChunkId, &Chunk)> {
        self.chunks.iter().map(|(id, value)| (*id, value))
    }
    /// Resolves a chunk only when this model owns its identity.
    pub fn chunk(&self, id: ChunkId) -> Option<&Chunk> {
        self.chunks
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
}

impl ModelMetadata {
    /// Edits an actual chunk without changing its scoped identity.
    pub fn chunk_mut(&mut self, id: ChunkId) -> Option<&mut Chunk> {
        self.chunks
            .iter_mut()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value)
    }
    /// Removes a chunk; checked publication rejects geometry that still selects it.
    pub fn remove_chunk(&mut self, id: ChunkId) -> Option<Chunk> {
        self.chunks
            .iter()
            .position(|(key, _)| *key == id)
            .map(|index| self.chunks.remove(index).1)
    }
}
