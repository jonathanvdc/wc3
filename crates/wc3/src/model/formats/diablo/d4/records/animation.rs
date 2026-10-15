//! Native animation records.
use super::*;
/// Native `IKGeneralBoneChain` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct IKGeneralBoneChain {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `eIKLimb` at byte 16.
    #[d4(value)]
    pub ik_limb: i32,
    /// Native `eIKTarget` at byte 20.
    #[d4(value)]
    pub ik_target: i32,
    /// Native `ptIKBones` at byte 24.
    #[d4(array)]
    pub ik_bones: Vec<IKBone>,
    /// Native `eIKTargetActor` at byte 40.
    #[d4(value)]
    pub ik_target_actor: i32,
    /// Native `tIKHardpoint` at byte 44.
    pub ik_hardpoint: HardpointLink,
    /// Unclassified native bytes at offset 52.
    #[d4(value)]
    pub reserved_52: [u8; 4],
}

/// Native `IKHip` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct IKHip {
    /// Native `tIKHipBone` at byte 0.
    pub ik_hip_bone: IKBone,
    /// Native `unk_155967` at byte 4.
    #[d4(value)]
    pub unk_155967: f32,
}

/// Native `AnimTreeNodeOverride` (120 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimTreeNodeOverride<'data> {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `dwID` at byte 16.
    #[d4(value)]
    pub id: u32,
    /// Native `eNodeType` at byte 20.
    #[d4(value)]
    pub node_type: i32,
    /// Native `eAnimLayer` at byte 24.
    #[d4(value)]
    pub anim_layer: i32,
    /// Unclassified native bytes at offset 28.
    #[d4(value)]
    pub reserved_28: [u8; 4],
    /// Native `ptChildren` at byte 32.
    #[d4(array)]
    pub children: Vec<AnimTreeNodeChild<'data>>,
    /// Native `eBlendParam` at byte 48.
    #[d4(value)]
    pub blend_param: i32,
    /// Native `eSecondaryBlendParam` at byte 52.
    #[d4(value)]
    pub secondary_blend_param: i32,
    /// Native `pt2DBlendTriangles` at byte 56.
    #[d4(array)]
    pub value_2_d_blend_triangles: Vec<AnimTreeBlendTriangle>,
    /// Native `tMinBlend` at byte 72.
    #[d4(value)]
    pub min_blend: [f32; 2],
    /// Native `tMaxBlend` at byte 80.
    #[d4(value)]
    pub max_blend: [f32; 2],
    /// Native `tBlendIn` at byte 88.
    pub blend_in: BlendSettings,
    /// Native `tBlendOut` at byte 96.
    pub blend_out: BlendSettings,
    /// Native `nDefaultChildIndex` at byte 104.
    #[d4(value)]
    pub default_child_index: i32,
    /// Native `nSyncGroupIndex` at byte 108.
    #[d4(value)]
    pub sync_group_index: i32,
    /// Native `nOverrideID` at byte 112.
    #[d4(value)]
    pub override_id: i32,
    /// Unclassified native bytes at offset 116.
    #[d4(value)]
    pub reserved_116: [u8; 4],
}

/// Native `IKBoneChainBase` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct IKBoneChainBase {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `eIKLimb` at byte 16.
    #[d4(value)]
    pub ik_limb: i32,
    /// Native `eIKTarget` at byte 20.
    #[d4(value)]
    pub ik_target: i32,
    /// Native `ptIKBones` at byte 24.
    #[d4(array)]
    pub ik_bones: Vec<IKBone>,
}

/// Native `AnimationDefinition` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimationRecord<'data> {
    /// Resource SNO identity from the root prefix.
    #[d4(value)]
    pub sno_id: i32,
    /// Root record flags.
    #[d4(value)]
    pub record_flags: u32,
    /// Native `dwNextID` at byte 8.
    #[d4(value)]
    pub next_id: u32,
    /// Native `dwFlags` at byte 12.
    #[d4(value)]
    pub flags: u32,
    /// Native `ePlaybackMode` at byte 16.
    #[d4(value)]
    pub playback_mode: i32,
    /// Native `unk_38fab0d` at byte 20.
    #[d4(value)]
    pub unk_38fab0d: i32,
    /// Native `eSelectionOrder` at byte 24.
    #[d4(value)]
    pub selection_order: i32,
    /// Native `unk_ba85a8e` at byte 28.
    #[d4(value)]
    pub unk_ba85a8e: i32,
    /// Native `bAlternatesAnims` at byte 32.
    #[d4(value)]
    pub alternates_anims: i32,
    /// Native `snoAppearance` at byte 36.
    #[d4(value)]
    pub appearance: i32,
    /// Native `ptPermutations` at byte 40.
    #[d4(array)]
    pub permutations: Vec<AnimationPermutation<'data>>,
    /// Native `arPermutationIndices` at byte 56.
    #[d4(array)]
    pub permutation_indices: ZeroVec<'data, i32>,
    /// Native `ptRepeatablePermutations` at byte 72.
    #[d4(array)]
    pub repeatable_permutations: ZeroVec<'data, u8>,
    /// Native `bHasPermutationGroups` at byte 88.
    #[d4(value)]
    pub has_permutation_groups: i32,
    /// Native `unk_5316586` at byte 92.
    #[d4(value)]
    pub unk_5316586: i32,
}

/// Native `AnimDepthOfField` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimDepthOfField {
    /// Native `flFStop` at byte 0.
    #[d4(value)]
    pub f_stop: f32,
    /// Native `flFocalDistance` at byte 4.
    #[d4(value)]
    pub focal_distance: f32,
}

/// Native `IKRopeTraversal` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct IKRopeTraversal {
    /// Native `wvLeftHandOffset` at byte 0.
    #[d4(value)]
    pub left_hand_offset: [f32; 3],
    /// Native `wvRightHandOffset` at byte 12.
    #[d4(value)]
    pub right_hand_offset: [f32; 3],
}

/// Native `InterpolationScalar` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationScalar {
    /// Native `nScalarFunction` at byte 0.
    #[d4(value)]
    pub scalar_function: i32,
    /// Native `dwFlags` at byte 4.
    #[d4(value)]
    pub flags: u32,
    /// Native `tInputRange` at byte 8.
    #[d4(value)]
    pub input_range: [f32; 2],
    /// Native `tRemapRange` at byte 16.
    #[d4(value)]
    pub remap_range: [f32; 2],
    /// Native `eRemapEasing` at byte 24.
    #[d4(value)]
    pub remap_easing: i32,
    /// Native `szFormulaName` at byte 28.
    #[d4(value)]
    pub formula_name: u32,
}

/// Native `InterpolationPath_bcVec3` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationPathBcVec3 {
    /// Native `tHeader` at byte 0.
    pub header: InterpolationPathHeader,
    /// Native `ptArrayNodes` at byte 40.
    #[d4(array)]
    pub array_nodes: Vec<InterpolationNodeBcVec3>,
}

/// Native `InterpolationPath_bcVec4` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationPathBcVec4 {
    /// Native `tHeader` at byte 0.
    pub header: InterpolationPathHeader,
    /// Native `ptArrayNodes` at byte 40.
    #[d4(array)]
    pub array_nodes: Vec<InterpolationNodeBcVec4>,
}

/// Native `AnimTreeNodeOverrideWrapper` (120 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimTreeNodeOverrideWrapper<'data> {
    /// Native `tChildOverride` at byte 0.
    pub child_override: AnimTreeNodeOverride<'data>,
}

/// Native `AnimPermutation` (376 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimationPermutation<'data> {
    /// Native `uID` at byte 0.
    #[d4(value)]
    pub u_id: u32,
    /// Native `dwFlags` at byte 4.
    #[d4(value)]
    pub flags: u32,
    /// Native `flFrameRate` at byte 8.
    #[d4(value)]
    pub frame_rate: f32,
    /// Native `flCompression` at byte 12.
    #[d4(value)]
    pub compression: f32,
    /// Native `dwBlendTime` at byte 16.
    #[d4(value)]
    pub blend_time: f32,
    /// Native `flBlendOutOverrideTime` at byte 20.
    #[d4(value)]
    pub blend_out_override_time: f32,
    /// Native `eBlendCurve` at byte 24.
    #[d4(value)]
    pub blend_curve: i32,
    /// Native `dwWeight` at byte 28.
    #[d4(value)]
    pub weight: u32,
    /// Native `tSpeedMultRange` at byte 32.
    #[d4(value)]
    pub speed_mult_range: [f32; 2],
    /// Native `nBoneCount` at byte 40.
    #[d4(value)]
    pub bone_count: i32,
    /// Unclassified native bytes at offset 44.
    #[d4(value)]
    pub reserved_44: [u8; 4],
    /// Native `ptPayloadData` at byte 48.
    #[d4(array, external)]
    pub payload_data: Vec<AnimPayloadData<'data>>,
    /// Native `tPhaseName` at byte 64.
    #[d4(value)]
    pub phase_name: u32,
    /// Native `ePhasePlaybackMode` at byte 68.
    #[d4(value)]
    pub phase_playback_mode: i32,
    /// Native `arContactFrames` at byte 72.
    #[d4(inline_array)]
    pub contact_frames: [AnimContactFrame; 4],
    /// Native `arFacingWindows` at byte 136.
    #[d4(array)]
    pub facing_windows: Vec<AnimFacingWindow>,
    /// Native `arIKFrames` at byte 152.
    #[d4(array)]
    pub ik_frames: Vec<IKFrames>,
    /// Native `unk_c61b9d1` at byte 168.
    #[d4(value)]
    pub unk_c61b9d1: f32,
    /// Native `arInterruptFrames` at byte 172.
    #[d4(value)]
    pub interrupt_frames: [i32; 5],
    /// Native `unk_75e1ebe` at byte 192.
    #[d4(value)]
    pub unk_75e1ebe: [i32; 2],
    /// Native `tRangeFacingWindowMirror` at byte 200.
    #[d4(value)]
    pub range_facing_window_mirror: [i32; 2],
    /// Native `unk_ddce6df` at byte 208.
    #[d4(value)]
    pub unk_ddce6df: i32,
    /// Native `eAnimationScalingType` at byte 212.
    #[d4(value)]
    pub animation_scaling_type: i32,
    /// Native `flMinimumAnimationScale` at byte 216.
    #[d4(value)]
    pub minimum_animation_scale: f32,
    /// Native `flMaximumAnimationScale` at byte 220.
    #[d4(value)]
    pub maximum_animation_scale: f32,
    /// Native `unk_4f89e54` at byte 224.
    #[d4(value)]
    pub unk_4f89e54: f32,
    /// Native `unk_40c83b5` at byte 228.
    #[d4(value)]
    pub unk_40c83b5: f32,
    /// Native `unk_41ae864` at byte 232.
    #[d4(value)]
    pub unk_41ae864: f32,
    /// Native `unk_f219e46` at byte 236.
    #[d4(value)]
    pub unk_f219e46: i32,
    /// Native `unk_acf4ed5` at byte 240.
    #[d4(value)]
    pub unk_acf4ed5: i32,
    /// Native `unk_779d247` at byte 244.
    #[d4(value)]
    pub unk_779d247: i32,
    /// Native `nPermutationGroup` at byte 248.
    #[d4(value)]
    pub permutation_group: u8,
    /// Unclassified native bytes at offset 249.
    #[d4(value)]
    pub reserved_249: [u8; 3],
    /// Native `nKeyframeCount` at byte 252.
    #[d4(value)]
    pub keyframe_count: i32,
    /// Native `nCycleCount` at byte 256.
    #[d4(value)]
    pub cycle_count: u32,
    /// Native `snoEffectGroup` at byte 260.
    #[d4(value)]
    pub effect_group: i32,
    /// Native `bEffectGroupUntracked` at byte 264.
    #[d4(value)]
    pub effect_group_untracked: i32,
    /// Unclassified native bytes at offset 268.
    #[d4(value)]
    pub reserved_268: [u8; 4],
    /// Native `arEffectGroupOverridesByVisualRemap` at byte 272.
    #[d4(array)]
    pub effect_group_overrides_by_visual_remap: Vec<EffectGroupOverrideByVisualRemap>,
    /// Native `unk_536659a` at byte 288.
    #[d4(value)]
    pub unk_536659a: i32,
    /// Native `unk_b4c58a6` at byte 292.
    #[d4(value)]
    pub unk_b4c58a6: i32,
    /// Native `ptSweptBones` at byte 296.
    #[d4(array)]
    pub swept_bones: Vec<AnimSweptBone>,
    /// Native `wvAvgVel` at byte 312.
    #[d4(value)]
    pub avg_vel: [f32; 3],
    /// Native `transInitialOffset` at byte 324.
    pub trans_initial_offset: PRTransform,
    /// Native `tHardpointLink` at byte 352.
    pub hardpoint_link: HardpointLink,
    /// Native `unk_bd4a157` at byte 360.
    #[d4(value)]
    pub unk_bd4a157: f32,
    /// Native `unk_a3acec8` at byte 364.
    #[d4(value)]
    pub unk_a3acec8: u32,
    /// Native `pnAnimToStructure` at byte 368.
    #[d4(value)]
    pub pn_anim_to_structure: u64,
}

/// Native `InterpolationPath_RGBAColor` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationPathRGBAColor {
    /// Native `tHeader` at byte 0.
    pub header: InterpolationPathHeader,
    /// Native `ptArrayNodes` at byte 40.
    #[d4(array)]
    pub array_nodes: Vec<InterpolationNodeRGBAColor>,
}

/// Native `AnimTreeBlendTriangle` (36 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimTreeBlendTriangle {
    /// Native `nChildIndex1` at byte 0.
    #[d4(value)]
    pub child_index1: u32,
    /// Native `nChildIndex2` at byte 4.
    #[d4(value)]
    pub child_index2: u32,
    /// Native `dwDisintegrative` at byte 8.
    #[d4(value)]
    pub disintegrative: u32,
    /// Native `tTriangle` at byte 12.
    pub triangle: Triangle2D,
}

/// Native `InterpolationNode_bcVec3` (28 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationNodeBcVec3 {
    /// Native `tValueMin` at byte 0.
    #[d4(value)]
    pub value_min: [f32; 3],
    /// Native `tValueMax` at byte 12.
    #[d4(value)]
    pub value_max: [f32; 3],
    /// Native `tTime` at byte 24.
    #[d4(value)]
    pub time: f32,
}

/// Native `InterpolationNode_bcVec4` (36 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationNodeBcVec4 {
    /// Native `tValueMin` at byte 0.
    #[d4(value)]
    pub value_min: [f32; 4],
    /// Native `tValueMax` at byte 16.
    #[d4(value)]
    pub value_max: [f32; 4],
    /// Native `tTime` at byte 32.
    #[d4(value)]
    pub time: f32,
}

/// Native `IKLegBoneChain` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct IKLegBoneChain {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `eIKLimb` at byte 16.
    #[d4(value)]
    pub ik_limb: i32,
    /// Native `eIKTarget` at byte 20.
    #[d4(value)]
    pub ik_target: i32,
    /// Native `ptIKBones` at byte 24.
    #[d4(array)]
    pub ik_bones: Vec<IKBone>,
    /// Native `bUseAnimatedOffset` at byte 40.
    #[d4(value)]
    pub use_animated_offset: i32,
    /// Native `flOffsetHeight` at byte 44.
    #[d4(value)]
    pub offset_height: f32,
    /// Native `tAlternateBone` at byte 48.
    pub alternate_bone: IKBone,
    /// Native `flAlternateBoneHeight` at byte 52.
    #[d4(value)]
    pub alternate_bone_height: f32,
}

/// Native `InterpolationNode_float` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationNodeFloat {
    /// Native `tValueMin` at byte 0.
    #[d4(value)]
    pub value_min: f32,
    /// Native `tValueMax` at byte 4.
    #[d4(value)]
    pub value_max: f32,
    /// Native `tTime` at byte 8.
    #[d4(value)]
    pub time: f32,
}

/// Native `InterpolationNode_int32` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationNodeInt32 {
    /// Native `tValueMin` at byte 0.
    #[d4(value)]
    pub value_min: i32,
    /// Native `tValueMax` at byte 4.
    #[d4(value)]
    pub value_max: i32,
    /// Native `tTime` at byte 8.
    #[d4(value)]
    pub time: f32,
}

/// Native `AnimPayloadData` (208 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimPayloadData<'data> {
    /// Native `ptBoneNames` at byte 0.
    #[d4(array)]
    pub bone_names: Vec<BoneName>,
    /// Native `unk_a9ead38` at byte 16.
    #[d4(array)]
    pub unk_a9ead38: ZeroVec<'data, f32>,
    /// Native `unk_8c9e18f` at byte 32.
    #[d4(array)]
    pub unk_8c9e18f: ZeroVec<'data, f32>,
    /// Native `pwvNonlinearOffset` at byte 48.
    #[d4(array)]
    pub pwv_nonlinear_offset: ZeroVec<'data, [f32; 3]>,
    /// Native `unk_5cd81c8` at byte 64.
    #[d4(array)]
    pub unk_5cd81c8: ZeroVec<'data, [f32; 3]>,
    /// Native `unk_fa7ecfb` at byte 80.
    #[d4(array)]
    pub unk_fa7ecfb: ZeroVec<'data, [f32; 3]>,
    /// Native `pflFacingYaw` at byte 96.
    #[d4(array)]
    pub pfl_facing_yaw: ZeroVec<'data, f32>,
    /// Native `pflRootScale` at byte 112.
    #[d4(array)]
    pub pfl_root_scale: ZeroVec<'data, f32>,
    /// Native `unk_7c60205` at byte 128.
    #[d4(array)]
    pub unk_7c60205: ZeroVec<'data, f32>,
    /// Native `ptDepthOfField` at byte 144.
    #[d4(array)]
    pub depth_of_field: Vec<AnimDepthOfField>,
    /// Native `ptTranslationCurves` at byte 160.
    #[d4(array)]
    pub translation_curves: Vec<TranslationCurve<'data>>,
    /// Native `ptRotationCurves` at byte 176.
    #[d4(array)]
    pub rotation_curves: Vec<RotationCurve<'data>>,
    /// Native `ptScaleCurves` at byte 192.
    #[d4(array)]
    pub scale_curves: Vec<ScaleCurve<'data>>,
}

/// Native `InterpolationPathHeader` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationPathHeader {
    /// Native `tLoopPointStart` at byte 0.
    #[d4(value)]
    pub loop_point_start: f32,
    /// Native `tLoopPointEnd` at byte 4.
    #[d4(value)]
    pub loop_point_end: f32,
    /// Native `nDistributionFunction` at byte 8.
    #[d4(value)]
    pub distribution_function: i32,
    /// Native `nMinMaxBehavior` at byte 12.
    #[d4(value)]
    pub min_max_behavior: i32,
    /// Native `arInterpScalar` at byte 16.
    #[d4(array)]
    pub interp_scalar: Vec<InterpolationScalar>,
    /// Native `dwFlags` at byte 32.
    #[d4(value)]
    pub flags: u32,
    /// Unclassified native bytes at offset 36.
    #[d4(value)]
    pub reserved_36: [u8; 4],
}

/// Native `InterpolationNode_RGBAColor` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationNodeRGBAColor {
    /// Native `tValueMin` at byte 0.
    #[d4(value)]
    pub value_min: [u8; 4],
    /// Native `tValueMax` at byte 4.
    #[d4(value)]
    pub value_max: [u8; 4],
    /// Native `tTime` at byte 8.
    #[d4(value)]
    pub time: f32,
}

/// Native `IKFrames` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct IKFrames {
    /// Native `eIKLimb` at byte 0.
    #[d4(value)]
    pub ik_limb: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `arIKWindows` at byte 8.
    #[d4(array)]
    pub ik_windows: Vec<IKWindow>,
    /// Native `arIKLockedPositionWindows` at byte 24.
    #[d4(array)]
    pub ik_locked_position_windows: Vec<IKWindow>,
}

/// Native `AnimSweptBone` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimSweptBone {
    /// Native `szBoneName` at byte 0.
    #[d4(value)]
    pub bone_name: u32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `ptOffsetList` at byte 8.
    #[d4(array, external)]
    pub offset_list: Vec<PRTransform>,
}

/// Native `IKBone` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct IKBone {
    /// Native `nBoneIndex` at byte 0.
    #[d4(value)]
    pub bone_index: i32,
}

/// Native `IKData` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct IKData<'data> {
    /// Native `flIKBlendTime` at byte 0.
    #[d4(value)]
    pub ik_blend_time: f32,
    /// Native `flGeneralIKBlendTime` at byte 4.
    #[d4(value)]
    pub general_ik_blend_time: f32,
    /// Native `arIKLimbs` at byte 8.
    #[d4(polymorphic = 540107926)]
    pub ik_limbs: Vec<Polymorphic<'data>>,
    /// Native `tHipAdjust` at byte 32.
    pub hip_adjust: IKHip,
    /// Native `tRopeTraversalAdjust` at byte 40.
    pub rope_traversal_adjust: IKRopeTraversal,
}

/// Native `AnimTreeOverride` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimTreeOverride<'data> {
    /// Native `ptNodeOverrides` at byte 0.
    #[d4(array)]
    pub node_overrides: Vec<AnimTreeNodeOverrideWrapper<'data>>,
}

/// Native `AnimTreeNodeChild` (80 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimTreeNodeChild<'data> {
    /// Native `tBlendCase` at byte 0.
    pub blend_case: AnimTreeBlendCase,
    /// Native `tSecondaryBlendCase` at byte 20.
    pub secondary_blend_case: AnimTreeBlendCase,
    /// Native `tCondition` at byte 40.
    pub condition: ConditionWrapper<'data>,
    /// Native `dwChildIndex` at byte 72.
    #[d4(value)]
    pub child_index: u32,
    /// Unclassified native bytes at offset 76.
    #[d4(value)]
    pub reserved_76: [u8; 4],
}

/// Native `AnimationClothOverride` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimationClothOverride<'data> {
    /// Native `arSnoAnimations` at byte 0.
    #[d4(array)]
    pub sno_animations: ZeroVec<'data, i32>,
    /// Native `snoClothOverride` at byte 16.
    #[d4(value)]
    pub cloth_override: i32,
    /// Native `flBlendTime` at byte 20.
    #[d4(value)]
    pub blend_time: f32,
}

/// Native `IKWindow` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct IKWindow {
    /// Native `tIKActiveFrames` at byte 0.
    #[d4(value)]
    pub ik_active_frames: [u32; 2],
}

/// Native `InterpolationPath_float` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationPathFloat {
    /// Native `tHeader` at byte 0.
    pub header: InterpolationPathHeader,
    /// Native `ptArrayNodes` at byte 40.
    #[d4(array)]
    pub array_nodes: Vec<InterpolationNodeFloat>,
}

/// Native `InterpolationPath_int32` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct InterpolationPathInt32 {
    /// Native `tHeader` at byte 0.
    pub header: InterpolationPathHeader,
    /// Native `ptArrayNodes` at byte 40.
    #[d4(array)]
    pub array_nodes: Vec<InterpolationNodeInt32>,
}

/// Native `AnimContactFrame` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimContactFrame {
    /// Native `nFrameNumber` at byte 0.
    #[d4(value)]
    pub frame_number: i32,
    /// Native `vLocalOffset` at byte 4.
    #[d4(value)]
    pub local_offset: [f32; 3],
}

/// Native `AnimTreeBlendCase` (20 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimTreeBlendCase {
    /// Native `eParam` at byte 0.
    #[d4(value)]
    pub param: i32,
    /// Native `nIntLiteral` at byte 4.
    #[d4(value)]
    pub int_literal: i32,
    /// Native `flFloatLiteral` at byte 8.
    #[d4(value)]
    pub float_literal: f32,
    /// Native `unk_707f056` at byte 12.
    #[d4(value)]
    pub unk_707f056: [f32; 2],
}

/// Native `AnimFacingWindow` (20 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AnimFacingWindow {
    /// Native `nFrameStart` at byte 0.
    #[d4(value)]
    pub frame_start: i32,
    /// Native `nFrameEnd` at byte 4.
    #[d4(value)]
    pub frame_end: i32,
    /// Native `flMaxYawSpeed` at byte 8.
    #[d4(value)]
    pub max_yaw_speed: f32,
    /// Native `unk_963cba8` at byte 12.
    #[d4(value)]
    pub unk_963cba8: i32,
    /// Native `unk_65d9350` at byte 16.
    #[d4(value)]
    pub unk_65d9350: i32,
}
