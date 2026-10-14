//! Native appearance records.
use super::*;
/// Native `OBB` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct OBB {
    /// Native `transform` at byte 0.
    pub transform: PRTransform,
    /// Native `wvExt` at byte 28.
    #[d4(value)]
    pub ext: [f32; 3],
}

/// Native `AABB` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Bounds {
    /// Native `wp` at byte 0.
    #[d4(value)]
    pub wp: [f32; 3],
    /// Native `wvExt` at byte 12.
    #[d4(value)]
    pub ext: [f32; 3],
}

/// Native `FoliageSettings` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct FoliageSettings {
    /// Native `flExplosionFactor` at byte 0.
    #[d4(value)]
    pub explosion_factor: f32,
    /// Native `flFrequency` at byte 4.
    #[d4(value)]
    pub frequency: f32,
    /// Native `flDampingRatio` at byte 8.
    #[d4(value)]
    pub damping_ratio: f32,
    /// Native `flMaxOffsetFactor` at byte 12.
    #[d4(value)]
    pub max_offset_factor: f32,
    /// Native `flRadiusRatio` at byte 16.
    #[d4(value)]
    pub radius_ratio: f32,
    /// Native `snoSoundDisturbed` at byte 20.
    #[d4(value)]
    pub sound_disturbed: i32,
    /// Native `snoSoundImpact` at byte 24.
    #[d4(value)]
    pub sound_impact: i32,
    /// Native `snoAmbientSound` at byte 28.
    #[d4(value)]
    pub ambient_sound: i32,
    /// Native `unk_620b5c3` at byte 32.
    #[d4(value)]
    pub unk_620b5c3: f32,
    /// Native `flMinimumImpactThreshold` at byte 36.
    #[d4(value)]
    pub minimum_impact_threshold: f32,
}

/// Native `LookVariantMap` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct LookVariantMap {
    /// Native `dwLookVariant` at byte 0.
    #[d4(value)]
    pub look_variant: u32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `arLookVariantOverrides` at byte 8.
    #[d4(array)]
    pub look_variant_overrides: Vec<LookVariantOverride>,
}

/// Native `Structure` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Structure<'data> {
    /// Native `ptCollisionMeshes` at byte 0.
    #[d4(array, external)]
    pub collision_meshes: Vec<CollisionMesh<'data>>,
    /// Native `ptCollisionCapsules` at byte 16.
    #[d4(array, external)]
    pub collision_capsules: Vec<CollisionCapsule>,
    /// Native `ptChunks` at byte 32.
    #[d4(array)]
    pub chunks: Vec<GeoChunk<'data>>,
    /// Native `ptSceneData` at byte 48.
    #[d4(array)]
    pub scene_data: Vec<SceneData<'data>>,
    /// Native `ptBoneData` at byte 64.
    #[d4(array)]
    pub bone_data: Vec<BoneData<'data>>,
    /// Native `dwFlags` at byte 80.
    #[d4(value)]
    pub flags: u32,
    /// Native `flVertexAnimationRadius` at byte 84.
    #[d4(value)]
    pub vertex_animation_radius: f32,
    /// Native `wsBounds` at byte 88.
    pub bounds: Sphere,
    /// Native `aabbBounds` at byte 104.
    pub aabb_bounds: Bounds,
    /// Native `nLowQualityLOD` at byte 128.
    #[d4(value)]
    pub low_quality_lod: i32,
    /// Unclassified native bytes at offset 132.
    #[d4(value)]
    pub reserved_132: [u8; 4],
    /// Native `ptPostprocessed` at byte 136.
    #[d4(value)]
    pub postprocessed: u64,
}

/// Native `BoneData` (416 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct BoneData<'data> {
    /// Native `unk_a3acec8` at byte 0.
    #[d4(value)]
    pub unk_a3acec8: u32,
    /// Native `nBaseBoneCount` at byte 4.
    #[d4(value)]
    pub base_bone_count: i32,
    /// Native `ptBoneStructure` at byte 8.
    #[d4(array, external)]
    pub bone_structure: Vec<BoneStructure<'data>>,
    /// Native `nClothBoneCount` at byte 24.
    #[d4(value)]
    pub cloth_bone_count: i32,
    /// Native `nMaxAnimLOD` at byte 28.
    #[d4(value)]
    pub max_anim_lod: i32,
    /// Native `bRawGroupBoneMaskingWeights` at byte 32.
    #[d4(inline_array)]
    pub raw_group_bone_masking_weights: [BoneMaskingRawWeights<'data>; 7],
    /// Native `ptGlobalComponentClothCapsules` at byte 144.
    #[d4(array, external)]
    pub global_component_cloth_capsules: Vec<GlobalComponentClothCapsule>,
    /// Native `tLookAtData` at byte 160.
    pub look_at_data: LookAtData,
    /// Native `tIKData` at byte 208.
    pub ik_data: IKData<'data>,
    /// Native `tLockedRotationBoneData` at byte 272.
    pub locked_rotation_bone_data: LockedRotationBoneData,
    /// Native `tGroundNormalTests` at byte 288.
    pub ground_normal_tests: GroundNormalTests,
    /// Native `ptHardpoints` at byte 312.
    #[d4(array)]
    pub hardpoints: Vec<Hardpoint>,
    /// Native `ptLoopConstraints` at byte 328.
    #[d4(array)]
    pub loop_constraints: Vec<ConstraintParameters>,
    /// Native `ptAdditionalDamageHardpoints` at byte 344.
    #[d4(array)]
    pub additional_damage_hardpoints: Vec<HardpointLink>,
    /// Native `ptNPCComponentHardpointOverrides` at byte 360.
    #[d4(array)]
    pub npc_component_hardpoint_overrides: Vec<NPCComponentHardpointOverride>,
    /// Native `ptHardpointMountedOffsets` at byte 376.
    #[d4(array)]
    pub hardpoint_mounted_offsets: Vec<HardpointOffset>,
    /// Native `ptNPCWeaponHardpointOverrides` at byte 392.
    #[d4(array)]
    pub npc_weapon_hardpoint_overrides: Vec<NPCWeaponHardpointOverride>,
    /// Native `unk_b98787b` at byte 408.
    #[d4(value)]
    pub unk_b98787b: i32,
    /// Native `uRagdollDegrade` at byte 412.
    #[d4(value)]
    pub u_ragdoll_degrade: u32,
}

/// Native `BoneName` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct BoneName {
    /// Native `szName` at byte 0.
    #[d4(value)]
    pub name: u32,
}

/// Native `SubObjectSegment` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SubObjectSegment<'data> {
    /// Native `pBoneIDs` at byte 0.
    #[d4(array)]
    pub bone_ids: ZeroVec<'data, i32>,
    /// Native `nVertCount` at byte 16.
    #[d4(value)]
    pub vert_count: i32,
    /// Native `nVertOffset` at byte 20.
    #[d4(value)]
    pub vert_offset: i32,
    /// Native `nIndexCount` at byte 24.
    #[d4(value)]
    pub index_count: i32,
    /// Native `nIndexOffset` at byte 28.
    #[d4(value)]
    pub index_offset: i32,
}

/// Native `VertexElem` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct VertexElem {
    /// Native `eSemantic` at byte 0.
    #[d4(value)]
    pub semantic: i32,
    /// Native `eFormat` at byte 4.
    #[d4(value)]
    pub format: i32,
    /// Native `nOffset` at byte 8.
    #[d4(value)]
    pub offset: i32,
}

/// Native `FoliageGrid` (48 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct FoliageGrid<'data> {
    /// Native `grid` at byte 0.
    #[d4(array)]
    pub grid: ZeroVec<'data, u32>,
    /// Native `aabbBounds` at byte 16.
    pub aabb_bounds: Bounds,
    /// Native `dwRows` at byte 40.
    #[d4(value)]
    pub rows: u32,
    /// Native `dwColumns` at byte 44.
    #[d4(value)]
    pub columns: u32,
}

/// Native `HardpointOffsets` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct HardpointOffsets {
    /// Native `fHardpointOffsetsHinting` at byte 0.
    #[d4(value)]
    pub hardpoint_offsets_hinting: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `arMaleOffsets` at byte 8.
    #[d4(array)]
    pub male_offsets: Vec<HardpointOffset>,
    /// Native `arFemaleOffsets` at byte 24.
    #[d4(array)]
    pub female_offsets: Vec<HardpointOffset>,
}

/// Native `SubObjectAppearance` (72 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SubObjectAppearance<'data> {
    /// Native `dwFlags` at byte 0.
    #[d4(value)]
    pub flags: u32,
    /// Native `snoMaterial` at byte 4.
    #[d4(value)]
    pub material: i32,
    /// Native `snoOverrideMaterial` at byte 8.
    #[d4(value)]
    pub override_material: i32,
    /// Native `snoCloth` at byte 12.
    #[d4(value)]
    pub cloth: i32,
    /// Native `snoHighQualityClothOverride` at byte 16.
    #[d4(value)]
    pub high_quality_cloth_override: i32,
    /// Native `arMountedClothOverrides` at byte 20.
    #[d4(value)]
    pub mounted_cloth_overrides: [i32; 4],
    /// Native `snoEffectGroup` at byte 36.
    #[d4(value)]
    pub effect_group: i32,
    /// Native `arAnimClothOverrides` at byte 40.
    #[d4(array)]
    pub anim_cloth_overrides: Vec<AnimationClothOverride<'data>>,
    /// Native `arVariantMaterials` at byte 56.
    #[d4(array)]
    pub variant_materials: ZeroVec<'data, i32>,
}

/// Native `Hardpoint` (44 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Hardpoint {
    /// Native `tInfo` at byte 0.
    pub info: HardpointInfo,
    /// Native `nBoneIndex` at byte 8.
    #[d4(value)]
    pub bone_index: i32,
    /// Native `bIgnoreParentOrientation` at byte 12.
    #[d4(value)]
    pub ignore_parent_orientation: i32,
    /// Native `transform` at byte 16.
    pub transform: PRTransform,
}

/// Native `AppearanceDefinition` (312 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AppearanceRecord<'data> {
    /// Resource SNO identity from the root prefix.
    #[d4(value)]
    pub sno_id: i32,
    /// Root record flags.
    #[d4(value)]
    pub record_flags: u32,
    /// Native `tStructure` at byte 8.
    pub structure: Structure<'data>,
    /// Native `dwFlags` at byte 152.
    #[d4(value)]
    pub flags: u32,
    /// Unclassified native bytes at offset 156.
    #[d4(value)]
    pub reserved_156: [u8; 4],
    /// Native `ptAppearanceLooks` at byte 160.
    #[d4(array)]
    pub appearance_looks: Vec<AppearanceLook>,
    /// Native `ptAppearanceMaterials` at byte 176.
    #[d4(array)]
    pub appearance_materials: Vec<AppearanceMaterial<'data>>,
    /// Native `snoFusedPrefab` at byte 192.
    #[d4(value)]
    pub fused_prefab: i32,
    /// Unclassified native bytes at offset 196.
    #[d4(value)]
    pub reserved_196: [u8; 4],
    /// Native `arLookVariantMap` at byte 200.
    #[d4(array)]
    pub look_variant_map: Vec<LookVariantMap>,
    /// Native `dwLookVariant` at byte 216.
    #[d4(value)]
    pub look_variant: u32,
    /// Native `tFoliageSettings` at byte 220.
    pub foliage_settings: FoliageSettings,
    /// Unclassified native bytes at offset 260.
    #[d4(value)]
    pub reserved_260: [u8; 4],
    /// Native `ptSpeedTreeWind` at byte 264.
    #[d4(polymorphic = 2780951310)]
    pub speed_tree_wind: Vec<Polymorphic<'data>>,
    /// Native `flLODDistanceMultiplier` at byte 288.
    #[d4(value)]
    pub lod_distance_multiplier: f32,
    /// Native `flDeformationScale` at byte 292.
    #[d4(value)]
    pub deformation_scale: f32,
    /// Native `arBaseAppearanceTextureOverrides` at byte 296.
    #[d4(array)]
    pub base_appearance_texture_overrides: Vec<TextureOverride>,
}

/// Native `LookLink` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct LookLink {
    /// Native `dwLookHash` at byte 0.
    #[d4(value)]
    pub look_hash: u32,
}

/// Native `PRTransform` (28 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct PRTransform {
    /// Native `q` at byte 0.
    pub q: BcQuat,
    /// Native `wp` at byte 16.
    #[d4(value)]
    pub wp: [f32; 3],
}

/// Native `LookVariantOverride` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct LookVariantOverride {
    /// Native `tBaseLook` at byte 0.
    pub base_look: LookLink,
    /// Native `tOverrideLook` at byte 4.
    pub override_look: LookLink,
}

/// Native `AppearanceUniqueSubObjectNameInfo` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AppearanceUniqueSubObjectNameInfo {
    /// Native `snoAppearance` at byte 0.
    #[d4(value)]
    pub appearance: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `arUniqueSubObjectNameInfos` at byte 8.
    #[d4(array)]
    pub unique_sub_object_name_infos: Vec<SubObjectNameInfo>,
}

/// Native `AppearanceLook` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AppearanceLook {
    /// Native `szLookName` at byte 0.
    #[d4(value)]
    pub look_name: u32,
}

/// Native `SubObjectNameInfo` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SubObjectNameInfo {
    /// Native `eType` at byte 0.
    #[d4(value)]
    pub type_: i32,
    /// Native `dwFlags` at byte 4.
    #[d4(value)]
    pub flags: u32,
    /// Native `dwPersona` at byte 8.
    #[d4(value)]
    pub persona: u8,
    /// Native `dwState` at byte 9.
    #[d4(value)]
    pub state: u8,
    /// Unclassified native bytes at offset 10.
    #[d4(value)]
    pub reserved_10: [u8; 2],
    /// Native `dwStyle` at byte 12.
    #[d4(value)]
    pub style: u32,
    /// Native `dwSlotHash` at byte 16.
    #[d4(value)]
    pub slot_hash: u32,
    /// Native `dwNPCComponentNameHash` at byte 20.
    #[d4(value)]
    pub npc_component_name_hash: u32,
    /// Native `dwDetailHash` at byte 24.
    #[d4(value)]
    pub detail_hash: u32,
    /// Native `unk_c15f7d2` at byte 28.
    #[d4(value)]
    pub unk_c15f7d2: u32,
    /// Native `unk_92fdd14` at byte 32.
    #[d4(value)]
    pub unk_92fdd14: u32,
    /// Native `unk_b0b7d46` at byte 36.
    #[d4(value)]
    pub unk_b0b7d46: u32,
}

/// Native `BoneStructure` (232 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct BoneStructure<'data> {
    /// Native `ptShapes` at byte 0.
    #[d4(array)]
    pub shapes: Vec<CollisionShape<'data>>,
    /// Native `ptConstraint` at byte 16.
    #[d4(array)]
    pub constraint: Vec<ConstraintParameters>,
    /// Native `dwHash` at byte 32.
    #[d4(value)]
    pub hash: u32,
    /// Native `dwFlags` at byte 36.
    #[d4(value)]
    pub flags: u32,
    /// Native `nParentIndex` at byte 40.
    #[d4(value)]
    pub parent_index: u16,
    /// Native `nLOD` at byte 42.
    #[d4(value)]
    pub lod: u16,
    /// Native `nTrueBoneLOD` at byte 44.
    #[d4(value)]
    pub true_bone_lod: u16,
    /// Native `unk_b03b557` at byte 46.
    #[d4(value)]
    pub unk_b03b557: u16,
    /// Native `snoParticleSystem` at byte 48.
    #[d4(value)]
    pub particle_system: i32,
    /// Native `wsBounds` at byte 52.
    pub bounds: Sphere,
    /// Native `transform` at byte 68.
    pub transform: PRSTransform,
    /// Native `transformInv` at byte 108.
    pub transform_inv: PRSTransform,
    /// Native `transformParentRel` at byte 148.
    pub transform_parent_rel: PRSTransform,
    /// Native `transformSkinningInv` at byte 188.
    pub transform_skinning_inv: PRSTransform,
    /// Unclassified native bytes at offset 228.
    #[d4(value)]
    pub reserved_228: [u8; 4],
}

/// Native `HardpointInfo` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct HardpointInfo {
    /// Native `dwHash` at byte 0.
    #[d4(value)]
    pub hash: u32,
    /// Native `dwHashFlag` at byte 4.
    #[d4(value)]
    pub hash_flag: u32,
}

/// Native `HardpointLink` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct HardpointLink {
    /// Native `tInfo` at byte 0.
    pub info: HardpointInfo,
}

/// Native `BoneMaskingRawWeights` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct BoneMaskingRawWeights<'data> {
    /// Native `rawWeights` at byte 0.
    #[d4(array)]
    pub raw_weights: ZeroVec<'data, f32>,
}

/// Native `HardpointOffset` (36 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct HardpointOffset {
    /// Native `tHardpointLink` at byte 0.
    pub hardpoint_link: HardpointLink,
    /// Native `vecOffset` at byte 8.
    #[d4(value)]
    pub vec_offset: [f32; 3],
    /// Native `qRotationOffset` at byte 20.
    pub rotation_offset: BcQuat,
}

/// Native `LookAtBone` (68 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct LookAtBone {
    /// Native `tHardpointLookAt` at byte 0.
    pub hardpoint_look_at: HardpointLink,
    /// Native `aYawMin` at byte 8.
    #[d4(value)]
    pub a_yaw_min: f32,
    /// Native `aYawMax` at byte 12.
    #[d4(value)]
    pub a_yaw_max: f32,
    /// Native `tCustomEnabledYaw` at byte 16.
    #[d4(value)]
    pub custom_enabled_yaw: [f32; 2],
    /// Native `tDisabledInnerYaw` at byte 24.
    #[d4(value)]
    pub disabled_inner_yaw: [f32; 2],
    /// Native `aPitchMin` at byte 32.
    #[d4(value)]
    pub a_pitch_min: f32,
    /// Native `aPitchMax` at byte 36.
    #[d4(value)]
    pub a_pitch_max: f32,
    /// Native `tDisabledInnerPitch` at byte 40.
    #[d4(value)]
    pub disabled_inner_pitch: [f32; 2],
    /// Native `aPitchOffset` at byte 48.
    #[d4(value)]
    pub a_pitch_offset: f32,
    /// Native `aSpeed` at byte 52.
    #[d4(value)]
    pub a_speed: f32,
    /// Native `bYawOnly` at byte 56.
    #[d4(value)]
    pub yaw_only: i32,
    /// Native `bDynamicPitchOffset` at byte 60.
    #[d4(value)]
    pub dynamic_pitch_offset: i32,
    /// Native `flDelay` at byte 64.
    #[d4(value)]
    pub delay: f32,
}

/// Native `LookAtData` (48 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct LookAtData {
    /// Native `flRange` at byte 0.
    #[d4(value)]
    pub range: f32,
    /// Native `unk_5115351` at byte 4.
    #[d4(value)]
    pub unk_5115351: f32,
    /// Native `arBones` at byte 8.
    #[d4(array)]
    pub bones: Vec<LookAtBone>,
    /// Native `tSlopeBone` at byte 24.
    pub slope_bone: SlopeBone,
    /// Native `tHardpointLinkLookAtTarget` at byte 36.
    pub hardpoint_link_look_at_target: HardpointLink,
    /// Unclassified native bytes at offset 44.
    #[d4(value)]
    pub reserved_44: [u8; 4],
}

/// Native `PRSTransform` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct PRSTransform {
    /// Native `q` at byte 0.
    pub q: BcQuat,
    /// Native `wp` at byte 16.
    #[d4(value)]
    pub wp: [f32; 3],
    /// Native `vScale` at byte 28.
    #[d4(value)]
    pub scale: [f32; 3],
}

/// Native `SubObject` (240 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SubObject<'data> {
    /// Native `dwFlags` at byte 0.
    #[d4(value)]
    pub flags: u32,
    /// Native `unk_334eb2d` at byte 4.
    #[d4(value)]
    pub unk_334eb2d: i32,
    /// Native `ptClothData` at byte 8.
    #[d4(array, external)]
    pub cloth_data: Vec<ClothData<'data>>,
    /// Native `unk_26f39c1` at byte 24.
    #[d4(array, external)]
    pub unk_26f39c1: Vec<Type72081710>,
    /// Native `dwBASegment` at byte 40.
    #[d4(value)]
    pub ba_segment: u32,
    /// Native `unk_87b4c64` at byte 44.
    #[d4(value)]
    pub unk_87b4c64: u32,
    /// Native `dwBASize` at byte 48.
    #[d4(value)]
    pub ba_size: u32,
    /// Native `unk_4c43adc` at byte 52.
    #[d4(value)]
    pub unk_4c43adc: u32,
    /// Native `tNameInfo` at byte 56.
    pub name_info: SubObjectNameInfo,
    /// Native `nMaterialIndex` at byte 96.
    #[d4(value)]
    pub material_index: i32,
    /// Native `dwSubObjectHash` at byte 100.
    #[d4(value)]
    pub sub_object_hash: u32,
    /// Native `nVertBufferIndex` at byte 104.
    #[d4(value)]
    pub vert_buffer_index: i32,
    /// Native `nIndexBufferIndex` at byte 108.
    #[d4(value)]
    pub index_buffer_index: i32,
    /// Native `nBaseLODSubObjectIndex` at byte 112.
    #[d4(value)]
    pub base_lod_sub_object_index: i32,
    /// Native `unk_a1b71f5` at byte 116.
    #[d4(value)]
    pub unk_a1b71f5: i32,
    /// Native `nSubObjectMaxLOD` at byte 120.
    #[d4(value)]
    pub sub_object_max_lod: i32,
    /// Native `dwShaderMapOverride` at byte 124.
    #[d4(value)]
    pub shader_map_override: u32,
    /// Native `snoCampaignVisibilityCondition` at byte 128.
    #[d4(value)]
    pub campaign_visibility_condition: i32,
    /// Native `aabbBounds` at byte 132.
    pub aabb_bounds: Bounds,
    /// Unclassified native bytes at offset 156.
    #[d4(value)]
    pub reserved_156: [u8; 4],
    /// Native `ptShapes` at byte 160.
    #[d4(array, external)]
    pub shapes: Vec<CollisionShape<'data>>,
    /// Native `ptBaseBoneInfluences` at byte 176.
    #[d4(array, external)]
    pub base_bone_influences: ZeroVec<'data, u8>,
    /// Native `ptPostprocessed` at byte 192.
    #[d4(value)]
    pub postprocessed: u64,
    /// Native `ptSegments` at byte 200.
    #[d4(array)]
    pub segments: Vec<SubObjectSegment<'data>>,
    /// Native `wpFixedPointPosOffset` at byte 216.
    #[d4(value)]
    pub fixed_point_pos_offset: [f32; 3],
    /// Native `wpFixedPointPosScale` at byte 228.
    #[d4(value)]
    pub fixed_point_pos_scale: [f32; 3],
}
