//! Collision shapes, body attachments, joints, and cloth tables use separate sections.
//!
//! Shape records distinguish source geometry from cooked hull or mesh data.
//! Reference counts and explicit geometry counts remain separate fields. Solver
//! parameters are exposed without imposing units or evaluating simulation.
use super::dialect::*;
use super::{targets, AnimationValue};
use crate::model::m3::{self, Record, Ref};
use core::fmt::Debug;
/// Fields of the native `DMSE` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ConvexHullHalfEdge {
    /// Signed displacement to the paired half-edge: adjacent pairs use +1 and -1.
    pub twin_offset: i8,
    /// Hull vertex index at the start of this directed edge.
    pub origin_vertex: u8,
    /// Hull face index on this directed edge.
    pub face: u8,
    /// Half-edge index continuing the boundary of the same hull face.
    pub next_in_face: u8,
}

impl Record for ConvexHullHalfEdge {
    const TAG: [u8; 4] = *b"DMSE";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `PhysicsMeshBvhNode` section revisions.
pub trait PhysicsMeshBvhNodeDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `normal`; `()` omits the field.
    type Normal: NativeField;
    /// Field representation for `octahedral_oct_x`; `()` omits the field.
    type OctahedralOctX: NativeField;
    /// Field representation for `octahedral_oct_y`; `()` omits the field.
    type OctahedralOctY: NativeField;
    /// Field representation for `octahedral_slab_min`; `()` omits the field.
    type OctahedralSlabMin: NativeField;
    /// Field representation for `octahedral_slab_max`; `()` omits the field.
    type OctahedralSlabMax: NativeField;
}

impl PhysicsMeshBvhNodeDialect for V0 {
    const VERSION: u32 = 0;
    type Normal = [f32; 3];
    type OctahedralOctX = ();
    type OctahedralOctY = ();
    type OctahedralSlabMin = ();
    type OctahedralSlabMax = ();
}

impl PhysicsMeshBvhNodeDialect for V1 {
    const VERSION: u32 = 1;
    type Normal = ();
    type OctahedralOctX = i16;
    type OctahedralOctY = i16;
    type OctahedralSlabMin = u16;
    type OctahedralSlabMax = u16;
}

/// Fields of the native `DMMN` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct PhysicsMeshBvhNode<D: PhysicsMeshBvhNodeDialect> {
    /// Three floating-point components of the slab normal in the older BVH layout.
    pub normal: D::Normal,
    /// First signed-normalized 16-bit coordinate of the octahedral normal encoding.
    pub octahedral_oct_x: D::OctahedralOctX,
    /// Second signed-normalized 16-bit coordinate of the octahedral normal encoding.
    pub octahedral_oct_y: D::OctahedralOctY,
    /// Lower distance bound of the slab in its quantized representation.
    pub octahedral_slab_min: D::OctahedralSlabMin,
    /// Upper quantized slab bound; zero identifies a leaf node.
    pub octahedral_slab_max: D::OctahedralSlabMax,
}

impl<D: PhysicsMeshBvhNodeDialect> Record for PhysicsMeshBvhNode<D> {
    const TAG: [u8; 4] = *b"DMMN";
    const VERSION: u32 = D::VERSION;
}

/// Fields of the native `DMMT` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct PhysicsMeshTriangle {
    /// First vertex selected from the owning shape position array.
    pub vertex_index0: u32,
    /// Second vertex selected from the owning shape position array.
    pub vertex_index1: u32,
    /// Third vertex selected from the owning shape position array.
    pub vertex_index2: u32,
    /// First edge selected from the owning shape edge array.
    pub edge_index0: u32,
    /// Second edge selected from the owning shape edge array.
    pub edge_index1: u32,
    /// Third edge selected from the owning shape edge array.
    pub edge_index2: u32,
    /// Uninterpreted word immediately before the triangle flag bits.
    pub reserved: u16,
    /// Triangle property bits.
    pub flags: u16,
}

impl Record for PhysicsMeshTriangle {
    const TAG: [u8; 4] = *b"DMMT";
    const VERSION: u32 = 0;
}

/// Fields of the native `DMME` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct PhysicsMeshEdge {
    /// Numeric edge classification; no Rust enum restricts its values.
    pub edge_type: u32,
    /// Position-array index for one endpoint of the edge.
    pub vertex_a: u32,
    /// Position-array index for the other endpoint of the edge.
    pub vertex_b: u32,
    /// Triangle-array index for one face incident on the edge.
    pub face_a: u32,
    /// Triangle-array index for the other face incident on the edge.
    pub face_b: u32,
}

impl Record for PhysicsMeshEdge {
    const TAG: [u8; 4] = *b"DMME";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `PhysicsShape` section revisions.
pub trait PhysicsShapeDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `legacy_reserved_0`; `()` omits the field.
    type LegacyReserved0: NativeField;
    /// Field representation for `legacy_reserved_1`; `()` omits the field.
    type LegacyReserved1: NativeField;
    /// Field representation for `reserved_0`; `()` omits the field.
    type Reserved0: NativeField;
    /// Field representation for `unused`; `()` omits the field.
    type Unused: NativeField;
    /// Field representation for `source_triangles`; `()` omits the field.
    type SourceTriangles: NativeField;
    /// Field representation for `planes`; `()` omits the field.
    type Planes: NativeField;
    /// Field representation for `hull_vertices`; `()` omits the field.
    type HullVertices: NativeField;
    /// Field representation for `hull_planes`; `()` omits the field.
    type HullPlanes: NativeField;
    /// Field representation for `hull_half_edges`; `()` omits the field.
    type HullHalfEdges: NativeField;
    /// Field representation for `hull_face_first_edges`; `()` omits the field.
    type HullFaceFirstEdges: NativeField;
    /// Field representation for `hull_centroid`; `()` omits the field.
    type HullCentroid: NativeField;
    /// Field representation for `hull_vertex_count`; `()` omits the field.
    type HullVertexCount: NativeField;
    /// Field representation for `hull_face_count`; `()` omits the field.
    type HullFaceCount: NativeField;
    /// Field representation for `hull_half_edge_count`; `()` omits the field.
    type HullHalfEdgeCount: NativeField;
    /// Field representation for `hull_volume`; `()` omits the field.
    type HullVolume: NativeField;
    /// Field representation for `hull_surface_area`; `()` omits the field.
    type HullSurfaceArea: NativeField;
    /// Field representation for `mesh_bvh_nodes`; `()` omits the field.
    type MeshBvhNodes: NativeField;
    /// Field representation for `mesh_vertex_positions`; `()` omits the field.
    type MeshVertexPositions: NativeField;
    /// Field representation for `mesh_face_indices16`; `()` omits the field.
    type MeshFaceIndices16: NativeField;
    /// Field representation for `mesh_face_indices32`; `()` omits the field.
    type MeshFaceIndices32: NativeField;
    /// Field representation for `mesh_bounds_center`; `()` omits the field.
    type MeshBoundsCenter: NativeField;
    /// Field representation for `mesh_bounds_extent`; `()` omits the field.
    type MeshBoundsExtent: NativeField;
    /// Field representation for `mesh_tolerance`; `()` omits the field.
    type MeshTolerance: NativeField;
    /// Field representation for `havok_tree`; `()` omits the field.
    type HavokTree: NativeField;
    /// Field representation for `mesh_normal_count`; `()` omits the field.
    type MeshNormalCount: NativeField;
    /// Field representation for `positions`; `()` omits the field.
    type Positions: NativeField;
    /// Field representation for `triangles`; `()` omits the field.
    type Triangles: NativeField;
    /// Field representation for `edges`; `()` omits the field.
    type Edges: NativeField;
    /// Field representation for `reserved_1`; `()` omits the field.
    type Reserved1: NativeField;
    /// Field representation for `mesh_vertex_count`; `()` omits the field.
    type MeshVertexCount: NativeField;
    /// Field representation for `mesh_face_index16_count`; `()` omits the field.
    type MeshFaceIndex16Count: NativeField;
    /// Field representation for `mesh_face_index32_count`; `()` omits the field.
    type MeshFaceIndex32Count: NativeField;
    /// Field representation for `reserved_2`; `()` omits the field.
    type Reserved2: NativeField;
    /// Field representation for `mesh_unknown1`; `()` omits the field.
    type MeshUnknown1: NativeField;
    /// Field representation for `mesh_reserved`; `()` omits the field.
    type MeshReserved: NativeField;
    /// Field representation for `mesh_tree_depth`; `()` omits the field.
    type MeshTreeDepth: NativeField;
    /// Field representation for `mesh_collision_margin`; `()` omits the field.
    type MeshCollisionMargin: NativeField;
}

impl PhysicsShapeDialect for V0 {
    const VERSION: u32 = 0;
    type LegacyReserved0 = FixedBytes<4>;
    type LegacyReserved1 = FixedBytes<3>;
    type Reserved0 = ();
    type Unused = ();
    type SourceTriangles = ();
    type Planes = ();
    type HullVertices = ();
    type HullPlanes = ();
    type HullHalfEdges = ();
    type HullFaceFirstEdges = ();
    type HullCentroid = ();
    type HullVertexCount = ();
    type HullFaceCount = ();
    type HullHalfEdgeCount = ();
    type HullVolume = ();
    type HullSurfaceArea = ();
    type MeshBvhNodes = ();
    type MeshVertexPositions = ();
    type MeshFaceIndices16 = ();
    type MeshFaceIndices32 = ();
    type MeshBoundsCenter = ();
    type MeshBoundsExtent = ();
    type MeshTolerance = ();
    type HavokTree = ();
    type MeshNormalCount = ();
    type Positions = ();
    type Triangles = ();
    type Edges = ();
    type Reserved1 = ();
    type MeshVertexCount = ();
    type MeshFaceIndex16Count = ();
    type MeshFaceIndex32Count = ();
    type Reserved2 = ();
    type MeshUnknown1 = ();
    type MeshReserved = ();
    type MeshTreeDepth = ();
    type MeshCollisionMargin = ();
}

impl PhysicsShapeDialect for V1 {
    const VERSION: u32 = 1;
    type LegacyReserved0 = FixedBytes<4>;
    type LegacyReserved1 = FixedBytes<3>;
    type Reserved0 = ();
    type Unused = Ref<targets::Byte>;
    type SourceTriangles = Ref<targets::U16>;
    type Planes = Ref<targets::Vector4>;
    type HullVertices = ();
    type HullPlanes = ();
    type HullHalfEdges = ();
    type HullFaceFirstEdges = ();
    type HullCentroid = ();
    type HullVertexCount = ();
    type HullFaceCount = ();
    type HullHalfEdgeCount = ();
    type HullVolume = ();
    type HullSurfaceArea = ();
    type MeshBvhNodes = ();
    type MeshVertexPositions = ();
    type MeshFaceIndices16 = ();
    type MeshFaceIndices32 = ();
    type MeshBoundsCenter = ();
    type MeshBoundsExtent = ();
    type MeshTolerance = ();
    type HavokTree = ();
    type MeshNormalCount = ();
    type Positions = ();
    type Triangles = ();
    type Edges = ();
    type Reserved1 = ();
    type MeshVertexCount = ();
    type MeshFaceIndex16Count = ();
    type MeshFaceIndex32Count = ();
    type Reserved2 = ();
    type MeshUnknown1 = ();
    type MeshReserved = ();
    type MeshTreeDepth = ();
    type MeshCollisionMargin = ();
}

impl PhysicsShapeDialect for V2 {
    const VERSION: u32 = 2;
    type LegacyReserved0 = ();
    type LegacyReserved1 = ();
    type Reserved0 = FixedBytes<3>;
    type Unused = ();
    type SourceTriangles = Ref<targets::U16>;
    type Planes = ();
    type HullVertices = Ref<targets::Vector3>;
    type HullPlanes = Ref<targets::Vector4>;
    type HullHalfEdges = Ref<targets::ConvexHullHalfEdge>;
    type HullFaceFirstEdges = Ref<targets::Byte>;
    type HullCentroid = [f32; 3];
    type HullVertexCount = u32;
    type HullFaceCount = u32;
    type HullHalfEdgeCount = u32;
    type HullVolume = f32;
    type HullSurfaceArea = f32;
    type MeshBvhNodes = ();
    type MeshVertexPositions = ();
    type MeshFaceIndices16 = ();
    type MeshFaceIndices32 = ();
    type MeshBoundsCenter = [f32; 3];
    type MeshBoundsExtent = [f32; 3];
    type MeshTolerance = [f32; 3];
    type HavokTree = Ref<targets::PhysicsMeshBvhNode>;
    type MeshNormalCount = ();
    type Positions = Ref<targets::Vector3>;
    type Triangles = Ref<targets::PhysicsMeshTriangle>;
    type Edges = Ref<targets::PhysicsMeshEdge>;
    type Reserved1 = FixedBytes<4>;
    type MeshVertexCount = u32;
    type MeshFaceIndex16Count = ();
    type MeshFaceIndex32Count = u32;
    type Reserved2 = FixedBytes<8>;
    type MeshUnknown1 = ();
    type MeshReserved = ();
    type MeshTreeDepth = u32;
    type MeshCollisionMargin = ();
}

impl PhysicsShapeDialect for V3 {
    const VERSION: u32 = 3;
    type LegacyReserved0 = ();
    type LegacyReserved1 = ();
    type Reserved0 = FixedBytes<3>;
    type Unused = ();
    type SourceTriangles = Ref<targets::U16>;
    type Planes = ();
    type HullVertices = Ref<targets::Vector3>;
    type HullPlanes = Ref<targets::Vector4>;
    type HullHalfEdges = Ref<targets::ConvexHullHalfEdge>;
    type HullFaceFirstEdges = Ref<targets::Byte>;
    type HullCentroid = [f32; 3];
    type HullVertexCount = u32;
    type HullFaceCount = u32;
    type HullHalfEdgeCount = u32;
    type HullVolume = f32;
    type HullSurfaceArea = f32;
    type MeshBvhNodes = Ref<targets::PhysicsMeshBvhNode>;
    type MeshVertexPositions = Ref<targets::Vector4>;
    type MeshFaceIndices16 = Ref<targets::MeshTriangle16>;
    type MeshFaceIndices32 = Ref<targets::MeshTriangle32>;
    type MeshBoundsCenter = [f32; 3];
    type MeshBoundsExtent = [f32; 3];
    type MeshTolerance = [f32; 3];
    type HavokTree = ();
    type MeshNormalCount = u32;
    type Positions = ();
    type Triangles = ();
    type Edges = ();
    type Reserved1 = ();
    type MeshVertexCount = u32;
    type MeshFaceIndex16Count = u32;
    type MeshFaceIndex32Count = u32;
    type Reserved2 = ();
    type MeshUnknown1 = u32;
    type MeshReserved = u32;
    type MeshTreeDepth = u32;
    type MeshCollisionMargin = f32;
}

/// Fields of the native `PHSH` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct PhysicsShape<D: PhysicsShapeDialect> {
    /// Matrix mapping shape coordinates into the attached body bone coordinates; scale may be present.
    pub transform: [[f32; 4]; 4],
    /// 4 uninterpreted bytes between `transform` and `shape_type`.
    pub legacy_reserved_0: D::LegacyReserved0,
    /// Shape discriminator: 0 box, 1 sphere, 2 capsule, 3 cylinder, 4 hull, 5 mesh.
    pub shape_type: u8,
    /// 3 uninterpreted bytes between `shape_type` and `source_points`.
    pub legacy_reserved_1: D::LegacyReserved1,
    /// 3 uninterpreted bytes between `shape_type` and `source_points`.
    pub reserved_0: D::Reserved0,
    /// VEC3 input points used to build collision geometry before the shape matrix is applied.
    pub source_points: Ref<targets::Vector3>,
    /// U8__ section link present in this shape revision; its purpose is unresolved.
    pub unused: D::Unused,
    /// U16_ input indices for cooking a mesh, grouped in triples per triangle.
    pub source_triangles: D::SourceTriangles,
    /// VEC4 plane data carried by the older shape layout.
    pub planes: D::Planes,
    /// Shape-dependent parameters: box half-extents, sphere radius, or capsule/cylinder radius and length.
    pub shape_dimensions: [f32; 3],
    /// VEC3 positions forming the cooked convex hull.
    pub hull_vertices: D::HullVertices,
    /// VEC4 hull planes; each entry stores a unit normal and its plane distance.
    pub hull_planes: D::HullPlanes,
    /// DMSE directed-edge records describing the cooked hull topology.
    pub hull_half_edges: D::HullHalfEdges,
    /// U8__ array selecting the initial half-edge for each hull face.
    pub hull_face_first_edges: D::HullFaceFirstEdges,
    /// Three coordinates of the hull volume centroid.
    pub hull_centroid: D::HullCentroid,
    /// Explicit hull vertex count, separate from the `hull_vertices` reference count.
    pub hull_vertex_count: D::HullVertexCount,
    /// Explicit hull face count, separate from the plane and face-edge reference counts.
    pub hull_face_count: D::HullFaceCount,
    /// Explicit hull edge count, separate from the `hull_half_edges` reference count.
    pub hull_half_edge_count: D::HullHalfEdgeCount,
    /// Precomputed enclosed volume of the hull.
    pub hull_volume: D::HullVolume,
    /// Precomputed area of the hull boundary.
    pub hull_surface_area: D::HullSurfaceArea,
    /// DMMN array associated with the cooked mesh tree.
    pub mesh_bvh_nodes: D::MeshBvhNodes,
    /// VEC4 cooked positions whose fourth component is zero.
    pub mesh_vertex_positions: D::MeshVertexPositions,
    /// MT16 face records used when triangle lanes fit in 16 bits.
    pub mesh_face_indices16: D::MeshFaceIndices16,
    /// MT32 face records used for 32-bit triangle lanes.
    pub mesh_face_indices32: D::MeshFaceIndices32,
    /// Center of the bounding box used by the cooked mesh tree.
    pub mesh_bounds_center: D::MeshBoundsCenter,
    /// Half-size of the cooked mesh tree bounding box along each axis.
    pub mesh_bounds_extent: D::MeshBoundsExtent,
    /// Quantization step for each axis, derived from the corresponding half-extent divided by 32767.
    pub mesh_tolerance: D::MeshTolerance,
    /// DMMN records in the older cooked mesh acceleration structure.
    pub havok_tree: D::HavokTree,
    /// Recorded DMMN node count, separate from the reference count.
    pub mesh_normal_count: D::MeshNormalCount,
    /// VEC3 mesh positions referenced by the older cooked mesh layout.
    pub positions: D::Positions,
    /// DMMT triangle records referenced by the older cooked mesh layout.
    pub triangles: D::Triangles,
    /// DMME edge records referenced by the older cooked mesh layout.
    pub edges: D::Edges,
    /// 4 uninterpreted bytes between `edges` and `mesh_vertex_count`.
    pub reserved_1: D::Reserved1,
    /// Explicit count of cooked mesh positions.
    pub mesh_vertex_count: D::MeshVertexCount,
    /// Explicit MT16 face count used when the 32-bit count is zero.
    pub mesh_face_index16_count: D::MeshFaceIndex16Count,
    /// Count of MT32 face records; zero selects the alternate MT16 representation.
    pub mesh_face_index32_count: D::MeshFaceIndex32Count,
    /// 8 uninterpreted bytes between `mesh_face_index32_count` and `mesh_tree_depth`.
    pub reserved_2: D::Reserved2,
    /// Additional mesh word after the face counts; its purpose is unresolved.
    pub mesh_unknown1: D::MeshUnknown1,
    /// Uninterpreted mesh word before the collision-margin parameter.
    pub mesh_reserved: D::MeshReserved,
    /// Recorded height of the cooked mesh acceleration tree.
    pub mesh_tree_depth: D::MeshTreeDepth,
    /// Collision-margin parameter recorded with the cooked mesh.
    pub mesh_collision_margin: D::MeshCollisionMargin,
}

impl<D: PhysicsShapeDialect> Record for PhysicsShape<D> {
    const TAG: [u8; 4] = *b"PHSH";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `RigidBody` section revisions.
pub trait RigidBodyDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `reserved_0`; `()` omits the field.
    type Reserved0: NativeField;
    /// Field representation for `simulation_type`; `()` omits the field.
    type SimulationType: NativeField;
    /// Field representation for `reserved_1`; `()` omits the field.
    type Reserved1: NativeField;
    /// Field representation for `physics_type`; `()` omits the field.
    type PhysicsType: NativeField;
    /// Field representation for `density`; `()` omits the field.
    type Density: NativeField;
    /// Field representation for `friction`; `()` omits the field.
    type Friction: NativeField;
    /// Field representation for `restitution`; `()` omits the field.
    type Restitution: NativeField;
    /// Field representation for `linear_damping`; `()` omits the field.
    type LinearDamping: NativeField;
    /// Field representation for `angular_damping`; `()` omits the field.
    type AngularDamping: NativeField;
    /// Field representation for `inertia_scale`; `()` omits the field.
    type InertiaScale: NativeField;
    /// Field representation for `dynamic_state`; `()` omits the field.
    type DynamicState: NativeField;
    /// Field representation for `dynamic_blend_out`; `()` omits the field.
    type DynamicBlendOut: NativeField;
    /// Field representation for `reserved_2`; `()` omits the field.
    type Reserved2: NativeField;
    /// Field representation for `flags`; `()` omits the field.
    type Flags: NativeField;
    /// Field representation for `reserved_3`; `()` omits the field.
    type Reserved3: NativeField;
    /// Field representation for `local_forces`; `()` omits the field.
    type LocalForces: NativeField;
    /// Field representation for `reserved_4`; `()` omits the field.
    type Reserved4: NativeField;
    /// Field representation for `world_forces`; `()` omits the field.
    type WorldForces: NativeField;
    /// Field representation for `reserved_5`; `()` omits the field.
    type Reserved5: NativeField;
    /// Field representation for `priority`; `()` omits the field.
    type Priority: NativeField;
}

impl RigidBodyDialect for V0 {
    const VERSION: u32 = 0;
    type Reserved0 = FixedBytes<36>;
    type SimulationType = ();
    type Reserved1 = FixedBytes<18>;
    type PhysicsType = ();
    type Density = ();
    type Friction = ();
    type Restitution = ();
    type LinearDamping = ();
    type AngularDamping = ();
    type InertiaScale = ();
    type DynamicState = ();
    type DynamicBlendOut = ();
    type Reserved2 = u8;
    type Flags = ();
    type Reserved3 = u8;
    type LocalForces = ();
    type Reserved4 = u8;
    type WorldForces = ();
    type Reserved5 = FixedBytes<1>;
    type Priority = ();
}

impl RigidBodyDialect for V1 {
    const VERSION: u32 = 1;
    type Reserved0 = FixedBytes<60>;
    type SimulationType = ();
    type Reserved1 = FixedBytes<18>;
    type PhysicsType = ();
    type Density = ();
    type Friction = ();
    type Restitution = ();
    type LinearDamping = ();
    type AngularDamping = ();
    type InertiaScale = ();
    type DynamicState = ();
    type DynamicBlendOut = ();
    type Reserved2 = u8;
    type Flags = ();
    type Reserved3 = u8;
    type LocalForces = ();
    type Reserved4 = u8;
    type WorldForces = ();
    type Reserved5 = FixedBytes<1>;
    type Priority = ();
}

impl RigidBodyDialect for V2 {
    const VERSION: u32 = 2;
    type Reserved0 = FixedBytes<60>;
    type SimulationType = ();
    type Reserved1 = FixedBytes<18>;
    type PhysicsType = ();
    type Density = ();
    type Friction = ();
    type Restitution = ();
    type LinearDamping = ();
    type AngularDamping = ();
    type InertiaScale = ();
    type DynamicState = ();
    type DynamicBlendOut = ();
    type Reserved2 = ();
    type Flags = u32;
    type Reserved3 = ();
    type LocalForces = u16;
    type Reserved4 = ();
    type WorldForces = u16;
    type Reserved5 = ();
    type Priority = u32;
}

impl RigidBodyDialect for V3 {
    const VERSION: u32 = 3;
    type Reserved0 = ();
    type SimulationType = u16;
    type Reserved1 = ();
    type PhysicsType = u32;
    type Density = f32;
    type Friction = f32;
    type Restitution = f32;
    type LinearDamping = f32;
    type AngularDamping = f32;
    type InertiaScale = f32;
    type DynamicState = ();
    type DynamicBlendOut = ();
    type Reserved2 = ();
    type Flags = u32;
    type Reserved3 = ();
    type LocalForces = u16;
    type Reserved4 = ();
    type WorldForces = u16;
    type Reserved5 = ();
    type Priority = u32;
}

impl RigidBodyDialect for V4 {
    const VERSION: u32 = 4;
    type Reserved0 = ();
    type SimulationType = u16;
    type Reserved1 = ();
    type PhysicsType = u32;
    type Density = f32;
    type Friction = f32;
    type Restitution = f32;
    type LinearDamping = f32;
    type AngularDamping = f32;
    type InertiaScale = f32;
    type DynamicState = AnimationValue<u32>;
    type DynamicBlendOut = f32;
    type Reserved2 = ();
    type Flags = u32;
    type Reserved3 = ();
    type LocalForces = u16;
    type Reserved4 = ();
    type WorldForces = u16;
    type Reserved5 = ();
    type Priority = u32;
}

/// Fields of the native `PHRB` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct RigidBody<D: RigidBodyDialect> {
    /// 60 uninterpreted bytes between the record start and `parent_bone_index`.
    pub reserved_0: D::Reserved0,
    /// Motion category: 0 dynamic, 1 kinematic, 2 static.
    pub simulation_type: D::SimulationType,
    /// BONE record to which this rigid body is attached.
    pub parent_bone_index: u16,
    /// 18 uninterpreted bytes between `parent_bone_index` and `rigid_body_shape`.
    pub reserved_1: D::Reserved1,
    /// Physics-material identity used to select material properties.
    pub physics_type: D::PhysicsType,
    /// Density parameter used with the collision shape to determine body mass.
    pub density: D::Density,
    /// Contact-friction parameter for this body.
    pub friction: D::Friction,
    /// Coefficient controlling velocity recovery after contact.
    pub restitution: D::Restitution,
    /// Attenuation parameter for translational velocity.
    pub linear_damping: D::LinearDamping,
    /// Attenuation parameter for rotational velocity.
    pub angular_damping: D::AngularDamping,
    /// Scale factor applied to body inertia.
    pub inertia_scale: D::InertiaScale,
    /// Animated simulation switch; the body flag word determines whether it is sampled.
    pub dynamic_state: D::DynamicState,
    /// Stored simulation-transition parameter; its interpretation is unresolved.
    pub dynamic_blend_out: D::DynamicBlendOut,
    /// PHSH array describing collision geometry for this rigid body.
    pub rigid_body_shape: Ref<targets::PhysicsShape>,
    /// 1 uninterpreted byte between `rigid_body_shape` and `reserved_3`.
    pub reserved_2: D::Reserved2,
    /// Body simulation-control bits.
    pub flags: D::Flags,
    /// 1 uninterpreted byte between `reserved_2` and `reserved_4`.
    pub reserved_3: D::Reserved3,
    /// Bitmask selecting local force channels for the body.
    pub local_forces: D::LocalForces,
    /// 1 uninterpreted byte between `reserved_3` and `reserved_5`.
    pub reserved_4: D::Reserved4,
    /// Bitmask selecting world force channels for the body.
    pub world_forces: D::WorldForces,
    /// 1 uninterpreted byte between `reserved_4` and the record end.
    pub reserved_5: D::Reserved5,
    /// Priority word present in the body layout; its operational meaning is unresolved.
    pub priority: D::Priority,
}

impl<D: RigidBodyDialect> Record for RigidBody<D> {
    const TAG: [u8; 4] = *b"PHRB";
    const VERSION: u32 = D::VERSION;
}

/// Fields of the native `PHYJ` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct PhysicsJoint {
    /// Joint discriminator: 0 spherical, 1 revolute, 2 cone-twist, 3 weld.
    pub joint_type: u32,
    /// BONE record on the first side of the joint.
    pub bone_index1: u32,
    /// BONE record on the second side of the joint.
    pub bone_index2: u32,
    /// Matrix expressing the joint frame relative to the first bone.
    pub matrix_body1: [[f32; 4]; 4],
    /// Matrix expressing the joint frame relative to the second bone.
    pub matrix_body2: [[f32; 4]; 4],
    /// Word whose low byte controls angular-limit enforcement.
    pub enable_limits: u32,
    /// Lower angular bound when joint limits are enabled.
    pub limit_min: f32,
    /// Upper angular bound when joint limits are enabled.
    pub limit_max: f32,
    /// Angular aperture of a cone-twist constraint.
    pub cone_angle: f32,
    /// Word whose low byte controls joint-friction enforcement.
    pub enable_friction: u32,
    /// Factor scaling the joint friction torque estimate.
    pub friction: f32,
    /// Dimensionless damping ratio for a weld-joint spring.
    pub damping_ratio: f32,
    /// Frequency parameter for a weld-joint spring.
    pub angular_frequency: f32,
    /// Stored joint breaking threshold; runtime application is unresolved.
    pub break_threshold: f32,
    /// Byte controlling collision between the connected bodies.
    pub enable_shape: u8,
    /// 3 uninterpreted bytes between `enable_shape` and the record end.
    pub reserved_0: [u8; 3],
}

impl Record for PhysicsJoint {
    const TAG: [u8; 4] = *b"PHYJ";
    const VERSION: u32 = 0;
}

/// Fields of the native `PHCC` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ClothCollider {
    /// Matrix placing the capsule geometry relative to its bone attachment.
    pub transform: [[f32; 4]; 4],
    /// Radial size of the cloth capsule collider.
    pub radius: f32,
    /// Total longitudinal size of the cloth capsule collider.
    pub height: f32,
    /// Attachment BONE index; `0xffff` attaches the collider to the model root.
    pub bone: u32,
}

impl Record for ClothCollider {
    const TAG: [u8; 4] = *b"PHCC";
    const VERSION: u32 = 0;
}

/// Fields of the native `PHAC` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ClothProxy {
    /// REGN index of the mesh receiving cloth deformation.
    pub proxy_index: u32,
    /// REGN index of the cloth cage used to deform the proxy mesh.
    pub cloth_index: u32,
    /// U64_ elements packing four cage-local u16 indices for each proxy vertex.
    pub proxy_vertices: Ref<targets::U64>,
    /// U32_ elements packing four u8 cage weights for each proxy vertex; weights use a denominator of 255.
    pub proxy_weights: Ref<targets::U32>,
}

impl Record for ClothProxy {
    const TAG: [u8; 4] = *b"PHAC";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `ClothPhysics` section revisions.
pub trait ClothPhysicsDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `revision0_reserved_0`; `()` omits the field.
    type Revision0Reserved0: NativeField;
    /// Field representation for `revision0_reserved_1`; `()` omits the field.
    type Revision0Reserved1: NativeField;
    /// Field representation for `proxies`; `()` omits the field.
    type Proxies: NativeField;
    /// Field representation for `revision0_reserved_2`; `()` omits the field.
    type Revision0Reserved2: NativeField;
    /// Field representation for `revision1_reserved_0`; `()` omits the field.
    type Revision1Reserved0: NativeField;
    /// Field representation for `revision2_reserved_0`; `()` omits the field.
    type Revision2Reserved0: NativeField;
    /// Field representation for `revision0_reserved_3`; `()` omits the field.
    type Revision0Reserved3: NativeField;
    /// Field representation for `revision1_reserved_1`; `()` omits the field.
    type Revision1Reserved1: NativeField;
    /// Field representation for `revision2_reserved_1`; `()` omits the field.
    type Revision2Reserved1: NativeField;
    /// Field representation for `revision3_reserved_0`; `()` omits the field.
    type Revision3Reserved0: NativeField;
    /// Field representation for `shear_stiffness`; `()` omits the field.
    type ShearStiffness: NativeField;
    /// Field representation for `drag_factor`; `()` omits the field.
    type DragFactor: NativeField;
    /// Field representation for `lift_factor`; `()` omits the field.
    type LiftFactor: NativeField;
    /// Field representation for `revision3_reserved_1`; `()` omits the field.
    type Revision3Reserved1: NativeField;
    /// Field representation for `legacy_active`; `()` omits the field.
    type LegacyActive: NativeField;
    /// Field representation for `revision3_reserved_2`; `()` omits the field.
    type Revision3Reserved2: NativeField;
    /// Field representation for `sphere_stiffness`; `()` omits the field.
    type SphereStiffness: NativeField;
    /// Field representation for `flatten`; `()` omits the field.
    type Flatten: NativeField;
    /// Field representation for `active`; `()` omits the field.
    type Active: NativeField;
    /// Field representation for `use_skin_collision`; `()` omits the field.
    type UseSkinCollision: NativeField;
    /// Field representation for `skin_offset`; `()` omits the field.
    type SkinOffset: NativeField;
    /// Field representation for `skin_exponent`; `()` omits the field.
    type SkinExponent: NativeField;
    /// Field representation for `skin_stiffness`; `()` omits the field.
    type SkinStiffness: NativeField;
    /// Field representation for `local_channels`; `()` omits the field.
    type LocalChannels: NativeField;
    /// Field representation for `local_wind`; `()` omits the field.
    type LocalWind: NativeField;
}

impl ClothPhysicsDialect for V0 {
    const VERSION: u32 = 0;
    type Revision0Reserved0 = Ref<targets::Any>;
    type Revision0Reserved1 = Ref<targets::Any>;
    type Proxies = ();
    type Revision0Reserved2 = FixedBytes<4>;
    type Revision1Reserved0 = ();
    type Revision2Reserved0 = ();
    type Revision0Reserved3 = FixedBytes<4>;
    type Revision1Reserved1 = ();
    type Revision2Reserved1 = ();
    type Revision3Reserved0 = ();
    type ShearStiffness = ();
    type DragFactor = ();
    type LiftFactor = ();
    type Revision3Reserved1 = ();
    type LegacyActive = ();
    type Revision3Reserved2 = ();
    type SphereStiffness = ();
    type Flatten = ();
    type Active = ();
    type UseSkinCollision = ();
    type SkinOffset = ();
    type SkinExponent = ();
    type SkinStiffness = ();
    type LocalChannels = ();
    type LocalWind = ();
}

impl ClothPhysicsDialect for V1 {
    const VERSION: u32 = 1;
    type Revision0Reserved0 = ();
    type Revision0Reserved1 = ();
    type Proxies = ();
    type Revision0Reserved2 = ();
    type Revision1Reserved0 = FixedBytes<4>;
    type Revision2Reserved0 = ();
    type Revision0Reserved3 = ();
    type Revision1Reserved1 = FixedBytes<4>;
    type Revision2Reserved1 = ();
    type Revision3Reserved0 = ();
    type ShearStiffness = ();
    type DragFactor = ();
    type LiftFactor = ();
    type Revision3Reserved1 = ();
    type LegacyActive = ();
    type Revision3Reserved2 = ();
    type SphereStiffness = ();
    type Flatten = ();
    type Active = ();
    type UseSkinCollision = ();
    type SkinOffset = ();
    type SkinExponent = ();
    type SkinStiffness = ();
    type LocalChannels = ();
    type LocalWind = ();
}

impl ClothPhysicsDialect for V2 {
    const VERSION: u32 = 2;
    type Revision0Reserved0 = ();
    type Revision0Reserved1 = ();
    type Proxies = Ref<targets::ClothProxy>;
    type Revision0Reserved2 = ();
    type Revision1Reserved0 = ();
    type Revision2Reserved0 = FixedBytes<4>;
    type Revision0Reserved3 = ();
    type Revision1Reserved1 = ();
    type Revision2Reserved1 = FixedBytes<4>;
    type Revision3Reserved0 = ();
    type ShearStiffness = ();
    type DragFactor = ();
    type LiftFactor = ();
    type Revision3Reserved1 = ();
    type LegacyActive = ();
    type Revision3Reserved2 = ();
    type SphereStiffness = ();
    type Flatten = ();
    type Active = ();
    type UseSkinCollision = ();
    type SkinOffset = ();
    type SkinExponent = ();
    type SkinStiffness = ();
    type LocalChannels = ();
    type LocalWind = ();
}

impl ClothPhysicsDialect for V3 {
    const VERSION: u32 = 3;
    type Revision0Reserved0 = ();
    type Revision0Reserved1 = ();
    type Proxies = Ref<targets::ClothProxy>;
    type Revision0Reserved2 = ();
    type Revision1Reserved0 = ();
    type Revision2Reserved0 = ();
    type Revision0Reserved3 = ();
    type Revision1Reserved1 = ();
    type Revision2Reserved1 = ();
    type Revision3Reserved0 = FixedBytes<4>;
    type ShearStiffness = f32;
    type DragFactor = f32;
    type LiftFactor = f32;
    type Revision3Reserved1 = FixedBytes<4>;
    type LegacyActive = AnimationValue<u32>;
    type Revision3Reserved2 = FixedBytes<8>;
    type SphereStiffness = f32;
    type Flatten = ();
    type Active = ();
    type UseSkinCollision = u32;
    type SkinOffset = f32;
    type SkinExponent = f32;
    type SkinStiffness = f32;
    type LocalChannels = u32;
    type LocalWind = ();
}

impl ClothPhysicsDialect for V4 {
    const VERSION: u32 = 4;
    type Revision0Reserved0 = ();
    type Revision0Reserved1 = ();
    type Proxies = Ref<targets::ClothProxy>;
    type Revision0Reserved2 = ();
    type Revision1Reserved0 = ();
    type Revision2Reserved0 = ();
    type Revision0Reserved3 = ();
    type Revision1Reserved1 = ();
    type Revision2Reserved1 = ();
    type Revision3Reserved0 = ();
    type ShearStiffness = f32;
    type DragFactor = f32;
    type LiftFactor = f32;
    type Revision3Reserved1 = ();
    type LegacyActive = ();
    type Revision3Reserved2 = ();
    type SphereStiffness = f32;
    type Flatten = u32;
    type Active = AnimationValue<u32>;
    type UseSkinCollision = u32;
    type SkinOffset = f32;
    type SkinExponent = f32;
    type SkinStiffness = f32;
    type LocalChannels = u32;
    type LocalWind = [f32; 3];
}

/// Fields of the native `PHCL` section, in serialized order.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ClothPhysics<D: ClothPhysicsDialect> {
    /// REGN index defining the simulated cloth cage.
    pub cage_region: u32,
    /// Stored anchor-bone count; its relationship to the `skin_bones` reference count is unresolved.
    pub skin_bone_count: u32,
    /// U16_ array of bone indices used by cloth anchors and colliders.
    pub skin_bones: Ref<targets::U16>,
    /// Section link of unresolved purpose; the expected target tag is not established.
    pub revision0_reserved_0: D::Revision0Reserved0,
    /// Section link of unresolved purpose; the expected target tag is not established.
    pub revision0_reserved_1: D::Revision0Reserved1,
    /// U8__ flags for cage particles; bit zero permits particle motion.
    pub sim_enabled: Ref<targets::Byte>,
    /// U32_ values packing four one-byte anchor-bone indices for each particle.
    pub vertex_bones: Ref<targets::U32>,
    /// U32_ values packing four one-byte anchor weights for each particle.
    pub vertex_weights: Ref<targets::U32>,
    /// PHCC capsules participating in cloth collision.
    pub colliders: Ref<targets::ClothCollider>,
    /// PHAC records relating simulated cage vertices to output mesh regions.
    pub proxies: D::Proxies,
    /// Density parameter used with the collision shape to determine body mass.
    pub density: f32,
    /// Parameter controlling how cloth tracks its anchors.
    pub tracking: f32,
    /// 4 uninterpreted bytes between `tracking` and `stretch_stiffness`.
    pub revision0_reserved_2: D::Revision0Reserved2,
    /// 4 uninterpreted bytes between `tracking` and `stretch_stiffness`.
    pub revision1_reserved_0: D::Revision1Reserved0,
    /// 4 uninterpreted bytes between `tracking` and `stretch_stiffness`.
    pub revision2_reserved_0: D::Revision2Reserved0,
    /// Resistance parameter for changes in cloth edge length.
    pub stretch_stiffness: f32,
    /// Stiffness parameter for the cloth horizontal constraint direction.
    pub horizontal_stiffness: f32,
    /// Resistance parameter for cloth bending constraints.
    pub bending_stiffness: f32,
    /// Cloth solver parameter attenuating particle motion.
    pub damping: f32,
    /// Contact-friction parameter for this body.
    pub friction: f32,
    /// Scale of the gravity contribution to cloth motion.
    pub gravity: f32,
    /// 4 uninterpreted bytes between `gravity` and `explosion_scale`.
    pub revision0_reserved_3: D::Revision0Reserved3,
    /// 4 uninterpreted bytes between `gravity` and `explosion_scale`.
    pub revision1_reserved_1: D::Revision1Reserved1,
    /// 4 uninterpreted bytes between `gravity` and `explosion_scale`.
    pub revision2_reserved_1: D::Revision2Reserved1,
    /// 4 uninterpreted bytes between `gravity` and `explosion_scale`.
    pub revision3_reserved_0: D::Revision3Reserved0,
    /// Multiplier for explosion forces applied to the cloth.
    pub explosion_scale: f32,
    /// Multiplier for wind forces applied to the cloth.
    pub wind_scale: f32,
    /// Resistance parameter for cloth shear constraints.
    pub shear_stiffness: D::ShearStiffness,
    /// Coefficient scaling aerodynamic drag on the cloth.
    pub drag_factor: D::DragFactor,
    /// Coefficient scaling aerodynamic lift on the cloth.
    pub lift_factor: D::LiftFactor,
    /// 4 uninterpreted bytes between `lift_factor` and `active`.
    pub revision3_reserved_1: D::Revision3Reserved1,
    /// Animated cloth-enable value; sampling is controlled by the cloth flag bits.
    pub legacy_active: D::LegacyActive,
    /// 8 uninterpreted bytes between `active` and `sphere_stiffness`.
    pub revision3_reserved_2: D::Revision3Reserved2,
    /// Response parameter for spherical collision constraints.
    pub sphere_stiffness: D::SphereStiffness,
    /// Numeric cloth flattening mode.
    pub flatten: D::Flatten,
    /// Animated cloth-enable value; sampling is controlled by the cloth flag bits.
    pub active: D::Active,
    /// Word enabling collision against the skinned mesh.
    pub use_skin_collision: D::UseSkinCollision,
    /// Offset parameter for collision against the skinned mesh.
    pub skin_offset: D::SkinOffset,
    /// Exponent parameter for the skinned-mesh collision response.
    pub skin_exponent: D::SkinExponent,
    /// Stiffness parameter for the skinned-mesh collision response.
    pub skin_stiffness: D::SkinStiffness,
    /// Stored local-channel word; channel assignments are unresolved.
    pub local_channels: D::LocalChannels,
    /// Local wind vector containing both direction and magnitude.
    pub local_wind: D::LocalWind,
}

impl<D: ClothPhysicsDialect> Record for ClothPhysics<D> {
    const TAG: [u8; 4] = *b"PHCL";
    const VERSION: u32 = D::VERSION;
}
