//! Native physics records.
use super::*;
/// Native `dmClothCapsuleDefMirror` (80 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmClothCapsuleDefMirror {
    /// Native `localTransform` at byte 0.
    pub local_transform: DmTransformMirror,
    /// Native `scale` at byte 32.
    #[d4(value)]
    pub scale: [f32; 4],
    /// Native `radius1` at byte 48.
    #[d4(value)]
    pub radius1: f32,
    /// Native `radius2` at byte 52.
    #[d4(value)]
    pub radius2: f32,
    /// Native `height` at byte 56.
    #[d4(value)]
    pub height: f32,
    /// Native `friction` at byte 60.
    #[d4(value)]
    pub friction: f32,
    /// Native `boneIndex` at byte 64.
    #[d4(value)]
    pub bone_index: u16,
    /// Native `solver` at byte 66.
    #[d4(value)]
    pub solver: u8,
    /// Native `hide` at byte 67.
    #[d4(value)]
    pub hide: u8,
    /// Native `padding` at byte 68.
    #[d4(value)]
    pub padding: [u16; 5],
    /// Unclassified native bytes at offset 78.
    #[d4(value)]
    pub reserved_78: [u8; 2],
}

/// Native `dmHeightDataMirror` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmHeightDataMirror {
    /// Native `countX` at byte 0.
    #[d4(value)]
    pub count_x: i32,
    /// Native `countY` at byte 4.
    #[d4(value)]
    pub count_y: i32,
    /// Native `cellWidth` at byte 8.
    #[d4(value)]
    pub cell_width: f32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `aSamples` at byte 16.
    #[d4(value)]
    pub a_samples: u64,
    /// Native `center` at byte 24.
    #[d4(value)]
    pub center: f32,
    /// Native `scale` at byte 28.
    #[d4(value)]
    pub scale: f32,
}

/// Native `CollisionShape` (80 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct CollisionShape<'data> {
    /// Native `dwFlags` at byte 0.
    #[d4(value)]
    pub flags: u32,
    /// Native `eType` at byte 4.
    #[d4(value)]
    pub type_: i32,
    /// Native `eMaterial` at byte 8.
    #[d4(value)]
    pub material: i32,
    /// Native `nDegrade` at byte 12.
    #[d4(value)]
    pub degrade: i32,
    /// Native `flDensityFactor` at byte 16.
    #[d4(value)]
    pub density_factor: f32,
    /// Native `flExplosionFactor` at byte 20.
    #[d4(value)]
    pub explosion_factor: f32,
    /// Native `flVelocityFactor` at byte 24.
    #[d4(value)]
    pub velocity_factor: f32,
    /// Unclassified native bytes at offset 28.
    #[d4(value)]
    pub reserved_28: [u8; 4],
    /// Native `pPolytopeData` at byte 32.
    #[d4(array)]
    pub p_polytope_data: Vec<PolytopeData<'data>>,
    /// Native `wp1` at byte 48.
    #[d4(value)]
    pub value_1: [f32; 3],
    /// Native `wp2` at byte 60.
    #[d4(value)]
    pub value_2: [f32; 3],
    /// Native `flRadius` at byte 72.
    #[d4(value)]
    pub radius: f32,
    /// Unclassified native bytes at offset 76.
    #[d4(value)]
    pub reserved_76: [u8; 4],
}

/// Native `dmMeshTriangleMirror` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmMeshTriangleMirror {
    /// Native `i1` at byte 0.
    #[d4(value)]
    pub i1: i32,
    /// Native `i2` at byte 4.
    #[d4(value)]
    pub i2: i32,
    /// Native `i3` at byte 8.
    #[d4(value)]
    pub i3: i32,
    /// Native `w1` at byte 12.
    #[d4(value)]
    pub w1: i32,
    /// Native `w2` at byte 16.
    #[d4(value)]
    pub w2: i32,
    /// Native `w3` at byte 20.
    #[d4(value)]
    pub w3: i32,
    /// Native `dwMaterialId` at byte 24.
    #[d4(value)]
    pub material_id: u16,
    /// Native `dwGameFlags` at byte 26.
    #[d4(value)]
    pub game_flags: u16,
    /// Native `dwIncludeMask` at byte 28.
    #[d4(value)]
    pub include_mask: u16,
    /// Unclassified native bytes at offset 30.
    #[d4(value)]
    pub reserved_30: [u8; 2],
}

/// Native `CollisionCapsule` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct CollisionCapsule {
    /// Native `flRadiusA` at byte 0.
    #[d4(value)]
    pub radius_a: f32,
    /// Native `flRadiusB` at byte 4.
    #[d4(value)]
    pub radius_b: f32,
    /// Native `flLength` at byte 8.
    #[d4(value)]
    pub length: f32,
    /// Native `tHardpoint` at byte 12.
    pub hardpoint: Hardpoint,
    /// Native `dwShapeFlags` at byte 56.
    #[d4(value)]
    pub shape_flags: u32,
    /// Native `eNavBehavior` at byte 60.
    #[d4(value)]
    pub nav_behavior: i32,
}

/// Native `dmClothDataMirror` (288 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmClothDataMirror {
    /// Native `bindVertices` at byte 0.
    #[d4(value)]
    pub bind_vertices: u64,
    /// Native `bindNormals` at byte 8.
    #[d4(value)]
    pub bind_normals: u64,
    /// Native `invMasses` at byte 16.
    #[d4(value)]
    pub inv_masses: u64,
    /// Native `blendWeights` at byte 24.
    #[d4(value)]
    pub blend_weights: u64,
    /// Native `animBlendFractions` at byte 32.
    #[d4(value)]
    pub anim_blend_fractions: u64,
    /// Native `deltaFrames` at byte 40.
    #[d4(value)]
    pub delta_frames: u64,
    /// Native `levels` at byte 48.
    #[d4(value)]
    pub levels: u64,
    /// Native `attachmentLengths` at byte 56.
    #[d4(value)]
    pub attachment_lengths: u64,
    /// Native `parentIndices` at byte 64.
    #[d4(value)]
    pub parent_indices: u64,
    /// Native `kinematicRoots` at byte 72.
    #[d4(value)]
    pub kinematic_roots: u64,
    /// Native `tangentIndices` at byte 80.
    #[d4(value)]
    pub tangent_indices: u64,
    /// Native `weights` at byte 88.
    #[d4(value)]
    pub weights: u64,
    /// Native `driverInfluences` at byte 96.
    #[d4(value)]
    pub driver_influences: u64,
    /// Native `followerIndices` at byte 104.
    #[d4(value)]
    pub follower_indices: u64,
    /// Native `triangles` at byte 112.
    #[d4(value)]
    pub triangles: u64,
    /// Native `constraintIndices` at byte 120.
    #[d4(value)]
    pub constraint_indices: u64,
    /// Native `constraintLengths` at byte 128.
    #[d4(value)]
    pub constraint_lengths: u64,
    /// Native `unk_d55a167` at byte 136.
    #[d4(value)]
    pub unk_d55a167: u64,
    /// Native `unk_e5fff43` at byte 144.
    #[d4(value)]
    pub unk_e5fff43: u64,
    /// Native `warpClusters` at byte 152.
    #[d4(value)]
    pub warp_clusters: u64,
    /// Native `weftClusters` at byte 160.
    #[d4(value)]
    pub weft_clusters: u64,
    /// Native `shearClusters` at byte 168.
    #[d4(value)]
    pub shear_clusters: u64,
    /// Native `bendClusters` at byte 176.
    #[d4(value)]
    pub bend_clusters: u64,
    /// Native `capsuleDefs` at byte 184.
    #[d4(value)]
    pub capsule_defs: u64,
    /// Native `planeDefs` at byte 192.
    #[d4(value)]
    pub plane_defs: u64,
    /// Native `driverBindPose` at byte 200.
    #[d4(value)]
    pub driver_bind_pose: u64,
    /// Native `driverMap` at byte 208.
    #[d4(value)]
    pub driver_map: u64,
    /// Native `name` at byte 216.
    #[d4(value)]
    pub name: [u8; 32],
    /// Native `density` at byte 248.
    #[d4(value)]
    pub density: f32,
    /// Native `vertexCount` at byte 252.
    #[d4(value)]
    pub vertex_count: u16,
    /// Native `vertexCapacity` at byte 254.
    #[d4(value)]
    pub vertex_capacity: u16,
    /// Native `kinematicCount` at byte 256.
    #[d4(value)]
    pub kinematic_count: u16,
    /// Native `triangleCount` at byte 258.
    #[d4(value)]
    pub triangle_count: u16,
    /// Native `warpClusterCount` at byte 260.
    #[d4(value)]
    pub warp_cluster_count: u16,
    /// Native `weftClusterCount` at byte 262.
    #[d4(value)]
    pub weft_cluster_count: u16,
    /// Native `shearClusterCount` at byte 264.
    #[d4(value)]
    pub shear_cluster_count: u16,
    /// Native `bendClusterCount` at byte 266.
    #[d4(value)]
    pub bend_cluster_count: u16,
    /// Native `constraintCount` at byte 268.
    #[d4(value)]
    pub constraint_count: u16,
    /// Native `unk_9460e91` at byte 270.
    #[d4(value)]
    pub unk_9460e91: u16,
    /// Native `maxLevel` at byte 272.
    #[d4(value)]
    pub max_level: u16,
    /// Native `boneCount` at byte 274.
    #[d4(value)]
    pub bone_count: u16,
    /// Native `driverCount` at byte 276.
    #[d4(value)]
    pub driver_count: u16,
    /// Native `capsuleCount` at byte 278.
    #[d4(value)]
    pub capsule_count: u16,
    /// Native `planeCount` at byte 280.
    #[d4(value)]
    pub plane_count: u16,
    /// Unclassified native bytes at offset 282.
    #[d4(value)]
    pub reserved_282: [u8; 6],
}

/// Native `dmTransformMirror` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmTransformMirror {
    /// Native `q` at byte 0.
    #[d4(value)]
    pub q: [f32; 4],
    /// Native `p` at byte 16.
    #[d4(value)]
    pub p: [f32; 4],
}

/// Native `CollisionMesh` (128 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct CollisionMesh<'data> {
    /// Native `meshDataMirror` at byte 0.
    pub mesh_data_mirror: DmMeshDataMirror,
    /// Native `ptDominoVertices` at byte 48.
    #[d4(array, external)]
    pub domino_vertices: Vec<DmFloat4Mirror>,
    /// Native `ptDominoTriangles` at byte 64.
    #[d4(array, external)]
    pub domino_triangles: Vec<DmMeshTriangleMirror>,
    /// Native `ptDominoNodes` at byte 80.
    #[d4(array, external)]
    pub domino_nodes: Vec<DmMeshNodeMirror>,
    /// Native `unk_b2f6ec8` at byte 96.
    #[d4(value)]
    pub unk_b2f6ec8: i32,
    /// Native `uFlags` at byte 100.
    #[d4(value)]
    pub u_flags: u32,
    /// Native `uMaterial` at byte 104.
    #[d4(value)]
    pub u_material: u32,
    /// Native `uGroupIndex` at byte 108.
    #[d4(value)]
    pub u_group_index: u16,
    /// Unclassified native bytes at offset 110.
    #[d4(value)]
    pub reserved_110: [u8; 2],
    /// Native `unk_732e0b0` at byte 112.
    #[d4(array)]
    pub unk_732e0b0: ZeroVec<'data, u32>,
}

/// Native `dmHeightSampleMirror` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmHeightSampleMirror {
    /// Native `height` at byte 0.
    #[d4(value)]
    pub height: f32,
    /// Native `material` at byte 4.
    #[d4(value)]
    pub material: u16,
    /// Native `gameFlags` at byte 6.
    #[d4(value)]
    pub game_flags: u8,
    /// Native `unk_38adf8e` at byte 7.
    #[d4(value)]
    pub unk_38adf8e: u8,
}

/// Native `ClothParamsOverride` (52 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ClothParamsOverride {
    /// Native `unk_a16bcb3` at byte 0.
    #[d4(value)]
    pub unk_a16bcb3: i32,
    /// Native `fOverrideAnimBlendFraction` at byte 4.
    #[d4(value)]
    pub override_anim_blend_fraction: i32,
    /// Native `fOverrideActorTrackingFactor` at byte 8.
    #[d4(value)]
    pub override_actor_tracking_factor: i32,
    /// Native `fOverrideBoneTrackingFactor` at byte 12.
    #[d4(value)]
    pub override_bone_tracking_factor: i32,
    /// Native `unk_7a4300b` at byte 16.
    #[d4(value)]
    pub unk_7a4300b: i32,
    /// Native `unk_41b76dd` at byte 20.
    #[d4(value)]
    pub unk_41b76dd: i32,
    /// Native `unk_e6772b7` at byte 24.
    #[d4(value)]
    pub unk_e6772b7: i32,
    /// Native `flAnimBlendFraction` at byte 28.
    #[d4(value)]
    pub anim_blend_fraction: f32,
    /// Native `flActorTrackingFactor` at byte 32.
    #[d4(value)]
    pub actor_tracking_factor: f32,
    /// Native `flBoneTrackingFactor` at byte 36.
    #[d4(value)]
    pub bone_tracking_factor: f32,
    /// Native `flBlendHertz` at byte 40.
    #[d4(value)]
    pub blend_hertz: f32,
    /// Native `unk_b8886b` at byte 44.
    #[d4(value)]
    pub unk_b8886b: f32,
    /// Native `unk_a26e5e3` at byte 48.
    #[d4(value)]
    pub unk_a26e5e3: f32,
}

/// Native `ClothData` (720 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ClothData<'data> {
    /// Native `clothDataMirror` at byte 0.
    pub cloth_data_mirror: DmClothDataMirror,
    /// Native `ptBindVertices` at byte 288.
    #[d4(array)]
    pub bind_vertices: ZeroVec<'data, [f32; 4]>,
    /// Native `ptBindNormals` at byte 304.
    #[d4(array)]
    pub bind_normals: ZeroVec<'data, [f32; 4]>,
    /// Native `ptInvMasses` at byte 320.
    #[d4(array)]
    pub inv_masses: ZeroVec<'data, f32>,
    /// Native `ptBlendWeights` at byte 336.
    #[d4(array)]
    pub blend_weights: ZeroVec<'data, f32>,
    /// Native `ptAnimBlendFractions` at byte 352.
    #[d4(array)]
    pub anim_blend_fractions: ZeroVec<'data, f32>,
    /// Native `ptDeltaFrames` at byte 368.
    #[d4(array)]
    pub delta_frames: Vec<DmMtxMirror>,
    /// Native `ptLevels` at byte 384.
    #[d4(array)]
    pub levels: ZeroVec<'data, u16>,
    /// Native `ptAttachmentLengths` at byte 400.
    #[d4(array)]
    pub attachment_lengths: ZeroVec<'data, f32>,
    /// Native `ptParentIndices` at byte 416.
    #[d4(array)]
    pub parent_indices: ZeroVec<'data, u16>,
    /// Native `ptKinematicRoots` at byte 432.
    #[d4(array)]
    pub kinematic_roots: ZeroVec<'data, u16>,
    /// Native `ptTangentIndices` at byte 448.
    #[d4(array)]
    pub tangent_indices: ZeroVec<'data, u16>,
    /// Native `ptWeights` at byte 464.
    #[d4(array)]
    pub weights: ZeroVec<'data, [f32; 4]>,
    /// Native `ptDriverInfluences` at byte 480.
    #[d4(array)]
    pub driver_influences: ZeroVec<'data, u16>,
    /// Native `ptFollowerIndices` at byte 496.
    #[d4(array)]
    pub follower_indices: ZeroVec<'data, u16>,
    /// Native `ptTriangles` at byte 512.
    #[d4(array)]
    pub triangles: ZeroVec<'data, u16>,
    /// Native `ptConstraintIndices` at byte 528.
    #[d4(array)]
    pub constraint_indices: ZeroVec<'data, u16>,
    /// Native `ptConstraintLengths` at byte 544.
    #[d4(array)]
    pub constraint_lengths: ZeroVec<'data, f32>,
    /// Native `unk_8ecbb2b` at byte 560.
    #[d4(array)]
    pub unk_8ecbb2b: ZeroVec<'data, u16>,
    /// Native `unk_9f71907` at byte 576.
    #[d4(array)]
    pub unk_9f71907: ZeroVec<'data, f32>,
    /// Native `ptWarpClusters` at byte 592.
    #[d4(array)]
    pub warp_clusters: Vec<DmConstraintClusterMirror>,
    /// Native `ptWeftClusters` at byte 608.
    #[d4(array)]
    pub weft_clusters: Vec<DmConstraintClusterMirror>,
    /// Native `ptShearClusters` at byte 624.
    #[d4(array)]
    pub shear_clusters: Vec<DmConstraintClusterMirror>,
    /// Native `ptBendClusters` at byte 640.
    #[d4(array)]
    pub bend_clusters: Vec<DmConstraintClusterMirror>,
    /// Native `ptCapsuleDefs` at byte 656.
    #[d4(array)]
    pub capsule_defs: Vec<DmClothCapsuleDefMirror>,
    /// Native `ptPlaneDefs` at byte 672.
    #[d4(array)]
    pub plane_defs: Vec<DmClothPlaneDefMirror>,
    /// Native `ptDriverBindPose` at byte 688.
    #[d4(array)]
    pub driver_bind_pose: Vec<DmFrameMirror>,
    /// Native `ptDriverMap` at byte 704.
    #[d4(array)]
    pub driver_map: ZeroVec<'data, u16>,
}

/// Native `CollisionSettings` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct CollisionSettings {
    /// Native `snoCollision` at byte 0.
    #[d4(value)]
    pub collision: i32,
    /// Native `tCustomCollFlags` at byte 4.
    pub custom_coll_flags: ActorCollisionFlags,
    /// Native `unk_a882afd` at byte 12.
    #[d4(value)]
    pub unk_a882afd: i32,
    /// Native `unk_54403ff` at byte 16.
    #[d4(value)]
    pub unk_54403ff: i32,
    /// Native `unk_eab2483` at byte 20.
    #[d4(value)]
    pub unk_eab2483: i32,
}

/// Native `dmMtxMirror` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmMtxMirror {
    /// Native `rx` at byte 0.
    #[d4(value)]
    pub rx: [f32; 4],
    /// Native `ry` at byte 16.
    #[d4(value)]
    pub ry: [f32; 4],
    /// Native `rz` at byte 32.
    #[d4(value)]
    pub rz: [f32; 4],
    /// Native `p` at byte 48.
    #[d4(value)]
    pub p: [f32; 4],
}

/// Native `dmMeshNodeMirror` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmMeshNodeMirror {
    /// Native `nLowerX` at byte 0.
    #[d4(value)]
    pub lower_x: f32,
    /// Native `nLowerY` at byte 4.
    #[d4(value)]
    pub lower_y: f32,
    /// Native `nLowerZ` at byte 8.
    #[d4(value)]
    pub lower_z: f32,
    /// Native `nUpperX` at byte 12.
    #[d4(value)]
    pub upper_x: f32,
    /// Native `nUpperY` at byte 16.
    #[d4(value)]
    pub upper_y: f32,
    /// Native `nUpperZ` at byte 20.
    #[d4(value)]
    pub upper_z: f32,
    /// Native `nIndex` at byte 24.
    #[d4(value)]
    pub index: i32,
    /// Native `dwIncludeMask` at byte 28.
    #[d4(value)]
    pub include_mask: u16,
    /// Native `dwAxis` at byte 30.
    #[d4(value)]
    pub axis: u8,
    /// Native `dwTriangleCount` at byte 31.
    #[d4(value)]
    pub triangle_count: u8,
}

/// Native `dmMeshDataMirror` (48 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmMeshDataMirror {
    /// Native `aNodes` at byte 0.
    #[d4(value)]
    pub a_nodes: u64,
    /// Native `aVertices` at byte 8.
    #[d4(value)]
    pub a_vertices: u64,
    /// Native `aTriangles` at byte 16.
    #[d4(value)]
    pub a_triangles: u64,
    /// Native `nNodeCount` at byte 24.
    #[d4(value)]
    pub node_count: i32,
    /// Native `nVertexCount` at byte 28.
    #[d4(value)]
    pub vertex_count: i32,
    /// Native `nTriangleCount` at byte 32.
    #[d4(value)]
    pub triangle_count: i32,
    /// Native `height` at byte 36.
    #[d4(value)]
    pub height: i32,
    /// Native `surfaceArea` at byte 40.
    #[d4(value)]
    pub surface_area: f32,
    /// Native `padding` at byte 44.
    #[d4(value)]
    pub padding: i32,
}

/// Native `dmClothPlaneDefMirror` (48 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmClothPlaneDefMirror {
    /// Native `localTransform` at byte 0.
    pub local_transform: DmTransformMirror,
    /// Native `stiffness` at byte 32.
    #[d4(value)]
    pub stiffness: f32,
    /// Native `friction` at byte 36.
    #[d4(value)]
    pub friction: f32,
    /// Native `boneIndex` at byte 40.
    #[d4(value)]
    pub bone_index: u16,
    /// Native `padding` at byte 42.
    #[d4(value)]
    pub padding: [u16; 3],
}

/// Native `dmFloat4Mirror` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmFloat4Mirror {
    /// Native `x` at byte 0.
    #[d4(value)]
    pub x: f32,
    /// Native `y` at byte 4.
    #[d4(value)]
    pub y: f32,
    /// Native `z` at byte 8.
    #[d4(value)]
    pub z: f32,
    /// Native `w` at byte 12.
    #[d4(value)]
    pub w: f32,
}

/// Native `CollisionHeightField` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct CollisionHeightField<'data> {
    /// Native `transform` at byte 0.
    pub transform: PRTransform,
    /// Native `fWater` at byte 28.
    #[d4(value)]
    pub water: i32,
    /// Native `fLandscape` at byte 32.
    #[d4(value)]
    pub landscape: i32,
    /// Unclassified native bytes at offset 36.
    #[d4(value)]
    pub reserved_36: [u8; 4],
    /// Native `heightDataMirror` at byte 40.
    pub height_data_mirror: DmHeightDataMirror,
    /// Native `ptDominoHeightSamples` at byte 72.
    #[d4(array, external)]
    pub domino_height_samples: Vec<DmHeightSampleMirror>,
    /// Native `ptWaterFlowDirection` at byte 88.
    #[d4(array, external)]
    pub water_flow_direction: ZeroVec<'data, [f32; 2]>,
}

/// Native `dmPolytopeDataMirror` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmPolytopeDataMirror {
    /// Native `m_aVertices` at byte 0.
    #[d4(value)]
    pub m_a_vertices: u64,
    /// Native `m_aPlanes` at byte 8.
    #[d4(value)]
    pub m_a_planes: u64,
    /// Native `m_aSubEdges` at byte 16.
    #[d4(value)]
    pub m_a_sub_edges: u64,
    /// Native `m_aFaceSubEdges` at byte 24.
    #[d4(value)]
    pub m_a_face_sub_edges: u64,
    /// Native `m_centroid` at byte 32.
    #[d4(value)]
    pub m_centroid: [f32; 3],
    /// Native `m_VertexCount` at byte 44.
    #[d4(value)]
    pub m_vertex_count: i32,
    /// Native `m_FaceCount` at byte 48.
    #[d4(value)]
    pub m_face_count: i32,
    /// Native `m_SubEdgeCount` at byte 52.
    #[d4(value)]
    pub m_sub_edge_count: i32,
    /// Native `m_volume` at byte 56.
    #[d4(value)]
    pub m_volume: f32,
    /// Native `m_area` at byte 60.
    #[d4(value)]
    pub m_area: f32,
}

/// Native `dmSubEdgeMirror` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmSubEdgeMirror {
    /// Native `twinOffset` at byte 0.
    #[d4(value)]
    pub twin_offset: u8,
    /// Native `tail` at byte 1.
    #[d4(value)]
    pub tail: u8,
    /// Native `left` at byte 2.
    #[d4(value)]
    pub left: u8,
    /// Native `next` at byte 3.
    #[d4(value)]
    pub next: u8,
}

/// Native `dmFrameMirror` (48 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmFrameMirror {
    /// Native `q` at byte 0.
    #[d4(value)]
    pub q: [f32; 4],
    /// Native `p` at byte 16.
    #[d4(value)]
    pub p: [f32; 4],
    /// Native `s` at byte 32.
    #[d4(value)]
    pub s: [f32; 4],
}

/// Native `dmPlaneMirror` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmPlaneMirror {
    /// Native `normal` at byte 0.
    #[d4(value)]
    pub normal: [f32; 3],
    /// Native `offset` at byte 12.
    #[d4(value)]
    pub offset: f32,
}

/// Native `dmConstraintClusterMirror` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DmConstraintClusterMirror {
    /// Native `startIndex` at byte 0.
    #[d4(value)]
    pub start_index: u16,
    /// Native `endIndex` at byte 2.
    #[d4(value)]
    pub end_index: u16,
}
