//! Native scene event records.
use super::*;
/// Native `TriggerEventAttachSpawnedActor` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventAttachSpawnedActor {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `dwID` at byte 96.
    #[d4(value)]
    pub id: u32,
    /// Native `tHardpointLink` at byte 100.
    pub hardpoint_link: HardpointLink,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventObjectTurnToFace` (128 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventObjectTurnToFace {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `dwObjectID` at byte 96.
    #[d4(value)]
    pub object_id: u32,
    /// Native `dwTargetID` at byte 100.
    #[d4(value)]
    pub target_id: u32,
    /// Native `aThreshold` at byte 104.
    #[d4(value)]
    pub a_threshold: f32,
    /// Native `unk_d74c75c` at byte 108.
    #[d4(value)]
    pub unk_d74c75c: f32,
    /// Native `fUseLookAtThreshold` at byte 112.
    #[d4(value)]
    pub use_look_at_threshold: i32,
    /// Native `unk_23237e1` at byte 116.
    #[d4(value)]
    pub unk_23237e1: i32,
    /// Native `unk_f1b6afc` at byte 120.
    #[d4(value)]
    pub unk_f1b6afc: f32,
    /// Unclassified native bytes at offset 124.
    #[d4(value)]
    pub reserved_124: [u8; 4],
}

/// Native `TriggerEventAddObject` (456 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventAddObject<'data> {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `fPlayOnOwner` at byte 144.
    #[d4(value)]
    pub play_on_owner: i32,
    /// Native `dwActorID` at byte 148.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `snoname` at byte 152.
    #[d4(value)]
    pub snoname: [i32; 2],
    /// Native `dwID` at byte 160.
    #[d4(value)]
    pub id: u32,
    /// Native `tHardpointLinks` at byte 164.
    #[d4(inline_array)]
    pub hardpoint_links: [HardpointLink; 2],
    /// Native `unk_e7c25e0` at byte 180.
    #[d4(value)]
    pub unk_e7c25e0: i32,
    /// Native `snoPowerEffectParam` at byte 184.
    #[d4(value)]
    pub power_effect_param: i32,
    /// Native `eAttractorVolume` at byte 188.
    #[d4(value)]
    pub attractor_volume: i32,
    /// Native `flAttractorVolumeRadius` at byte 192.
    #[d4(value)]
    pub attractor_volume_radius: f32,
    /// Native `flAttractorVolumeHeight` at byte 196.
    #[d4(value)]
    pub attractor_volume_height: f32,
    /// Native `dwSeed` at byte 200.
    #[d4(value)]
    pub seed: u32,
    /// Native `fBillboard` at byte 204.
    #[d4(value)]
    pub billboard: i32,
    /// Native `eAddType` at byte 208.
    #[d4(value)]
    pub add_type: i32,
    /// Native `fDoNotParent` at byte 212.
    #[d4(value)]
    pub do_not_parent: i32,
    /// Native `fKeepOriginal` at byte 216.
    #[d4(value)]
    pub keep_original: i32,
    /// Native `fEnableMovement` at byte 220.
    #[d4(value)]
    pub enable_movement: i32,
    /// Native `unk_773852b` at byte 224.
    #[d4(value)]
    pub unk_773852b: i32,
    /// Native `unk_62ab747` at byte 228.
    #[d4(value)]
    pub unk_62ab747: i32,
    /// Native `unk_cb43646` at byte 232.
    #[d4(value)]
    pub unk_cb43646: i32,
    /// Native `eAppearanceOverrideType` at byte 236.
    #[d4(value)]
    pub appearance_override_type: i32,
    /// Native `eAppearanceOverrideInventorySlot` at byte 240.
    #[d4(value)]
    pub appearance_override_inventory_slot: i32,
    /// Native `snoAppearanceOverride` at byte 244.
    #[d4(value)]
    pub appearance_override: i32,
    /// Native `fRandomLook` at byte 248.
    #[d4(value)]
    pub random_look: i32,
    /// Native `tLookLink` at byte 252.
    pub look_link: LookLink,
    /// Native `fOverrideActorVariant` at byte 256.
    #[d4(value)]
    pub override_actor_variant: i32,
    /// Native `eVariantTypeOverride` at byte 260.
    #[d4(value)]
    pub variant_type_override: i32,
    /// Native `snoAnimSetOverride` at byte 264.
    #[d4(value)]
    pub anim_set_override: i32,
    /// Unclassified native bytes at offset 268.
    #[d4(value)]
    pub reserved_268: [u8; 4],
    /// Native `snoPlayerAnimSetOverrides` at byte 272.
    #[d4(array)]
    pub player_anim_set_overrides: Vec<PlayerAnimSetOverride>,
    /// Native `dwSegmentsStep` at byte 288.
    #[d4(value)]
    pub segments_step: u32,
    /// Native `dwShaderMapOverride` at byte 292.
    #[d4(value)]
    pub shader_map_override: u32,
    /// Native `nPlayerIndex` at byte 296.
    #[d4(value)]
    pub player_index: i32,
    /// Native `flScale` at byte 300.
    #[d4(value)]
    pub scale: f32,
    /// Native `unk_ebd19c` at byte 304.
    #[d4(value)]
    pub unk_ebd19c: i32,
    /// Native `szSpawnLocatorName` at byte 308.
    #[d4(value)]
    pub spawn_locator_name: u32,
    /// Native `vecScalar` at byte 312.
    #[d4(value)]
    pub vec_scalar: [f32; 2],
    /// Native `transform` at byte 320.
    pub transform: PRTransform,
    /// Native `tEffectGroupHardpointLink` at byte 348.
    pub effect_group_hardpoint_link: HardpointLink,
    /// Native `unk_c7fa18a` at byte 356.
    #[d4(value)]
    pub unk_c7fa18a: [f32; 2],
    /// Native `idRActorAttachment` at byte 364.
    #[d4(value)]
    pub id_r_actor_attachment: i32,
    /// Native `nPermIndex` at byte 368.
    #[d4(value)]
    pub perm_index: i32,
    /// Native `snoCopyFromMarkerSet` at byte 372.
    #[d4(value)]
    pub copy_from_marker_set: i32,
    /// Native `nCopyFromMarkerID` at byte 376.
    #[d4(value)]
    pub copy_from_marker_id: i32,
    /// Native `nMaxAttachments` at byte 380.
    #[d4(value)]
    pub max_attachments: u32,
    /// Native `flBlendInTime` at byte 384.
    #[d4(value)]
    pub blend_in_time: f32,
    /// Native `flBlendOutTime` at byte 388.
    #[d4(value)]
    pub blend_out_time: f32,
    /// Native `nPriority` at byte 392.
    #[d4(value)]
    pub priority: i32,
    /// Native `tNPCComponentInstanceData` at byte 396.
    pub npc_component_instance_data: NPCComponentInstanceData,
    /// Native `unk_ebb9b95` at byte 436.
    #[d4(value)]
    pub unk_ebb9b95: i32,
    /// Native `ptLightInstanceData` at byte 440.
    #[d4(array)]
    pub light_instance_data: Vec<LightInstanceData<'data>>,
}

/// Native `TriggerEventDeleteObject` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventDeleteObject {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `fDeleteFromOwner` at byte 96.
    #[d4(value)]
    pub delete_from_owner: i32,
    /// Native `dwActorID` at byte 100.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `dwID` at byte 104.
    #[d4(value)]
    pub id: u32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventActorMovement` (224 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventActorMovement {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `szLocatorName` at byte 144.
    #[d4(value)]
    pub locator_name: u32,
    /// Native `dwActorID` at byte 148.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `eMoveToType` at byte 152.
    #[d4(value)]
    pub move_to_type: i32,
    /// Native `eOrientationMode` at byte 156.
    #[d4(value)]
    pub orientation_mode: i32,
    /// Native `eLookAtTargetType` at byte 160.
    #[d4(value)]
    pub look_at_target_type: i32,
    /// Native `szLookAtTargetLocatorName` at byte 164.
    #[d4(value)]
    pub look_at_target_locator_name: u32,
    /// Native `tHardpointLinkLookAtTarget` at byte 168.
    pub hardpoint_link_look_at_target: HardpointLink,
    /// Native `dwLookAtTargetID` at byte 176.
    #[d4(value)]
    pub look_at_target_id: u32,
    /// Native `eBlendEasing` at byte 180.
    #[d4(value)]
    pub blend_easing: i32,
    /// Native `eMoveRate` at byte 184.
    #[d4(value)]
    pub move_rate: i32,
    /// Native `eDynamicMovementSpeed` at byte 188.
    #[d4(value)]
    pub dynamic_movement_speed: i32,
    /// Native `fUseDynamicMovement` at byte 192.
    #[d4(value)]
    pub use_dynamic_movement: i32,
    /// Native `unk_17909b9` at byte 196.
    #[d4(value)]
    pub unk_17909b9: i32,
    /// Native `tHardpointLinkTarget` at byte 200.
    pub hardpoint_link_target: HardpointLink,
    /// Native `dwTargetID` at byte 208.
    #[d4(value)]
    pub target_id: u32,
    /// Native `unk_23237e1` at byte 212.
    #[d4(value)]
    pub unk_23237e1: i32,
    /// Native `unk_f1b6afc` at byte 216.
    #[d4(value)]
    pub unk_f1b6afc: f32,
    /// Unclassified native bytes at offset 220.
    #[d4(value)]
    pub reserved_220: [u8; 4],
}

/// Native `TriggerEventDisableRagdollAnimationOverride` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventDisableRagdollAnimationOverride {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
}

/// Native `TriggerEventDeleteAllObjects` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventDeleteAllObjects {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
}

/// Native `TriggerEventSubObjectVisibility` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSubObjectVisibility {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `szSubObjectName` at byte 96.
    #[d4(value)]
    pub sub_object_name: u32,
    /// Native `bVisible` at byte 100.
    #[d4(value)]
    pub visible: i32,
}

/// Native `TriggerEventCancelLook` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCancelLook {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `dwActorID` at byte 96.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `snoAppearance` at byte 100.
    #[d4(value)]
    pub appearance: i32,
    /// Native `tLookLink` at byte 104.
    pub look_link: LookLink,
    /// Native `fAffectOwner` at byte 108.
    #[d4(value)]
    pub affect_owner: i32,
}

/// Native `TriggerEventActorPlayAnimation` (248 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventActorPlayAnimation {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `eAnimBlendTimeMode` at byte 144.
    #[d4(value)]
    pub anim_blend_time_mode: i32,
    /// Native `eAnimBlendOutTimeMode` at byte 148.
    #[d4(value)]
    pub anim_blend_out_time_mode: i32,
    /// Native `flOverriddenBlendTime` at byte 152.
    #[d4(value)]
    pub overridden_blend_time: f32,
    /// Native `flOverriddenBlendOutTime` at byte 156.
    #[d4(value)]
    pub overridden_blend_out_time: f32,
    /// Native `dwActorID` at byte 160.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `eAnimLayerMode` at byte 164.
    #[d4(value)]
    pub anim_layer_mode: i32,
    /// Native `eAnimBoneMasking` at byte 168.
    #[d4(value)]
    pub anim_bone_masking: i32,
    /// Native `fUseBoneMasking` at byte 172.
    #[d4(value)]
    pub use_bone_masking: i32,
    /// Native `bUsePowerAnimLayer` at byte 176.
    #[d4(value)]
    pub use_power_anim_layer: i32,
    /// Native `fResetIK` at byte 180.
    #[d4(value)]
    pub reset_ik: i32,
    /// Native `unk_9b2ead` at byte 184.
    #[d4(value)]
    pub unk_9b2ead: i32,
    /// Native `snonameAnim` at byte 188.
    #[d4(value)]
    pub snoname_anim: [i32; 2],
    /// Native `fPlayOnOwner` at byte 196.
    #[d4(value)]
    pub play_on_owner: i32,
    /// Native `fRandomPermutation` at byte 200.
    #[d4(value)]
    pub random_permutation: i32,
    /// Native `nPermutationIndex` at byte 204.
    #[d4(value)]
    pub permutation_index: i32,
    /// Native `unk_32a328c` at byte 208.
    #[d4(value)]
    pub unk_32a328c: i32,
    /// Native `flAnimSpeedScale` at byte 212.
    #[d4(value)]
    pub anim_speed_scale: f32,
    /// Native `dwAnimWeight` at byte 216.
    #[d4(value)]
    pub anim_weight: u8,
    /// Unclassified native bytes at offset 217.
    #[d4(value)]
    pub reserved_217: [u8; 3],
    /// Native `eAnimLoopMode` at byte 220.
    #[d4(value)]
    pub anim_loop_mode: i32,
    /// Native `unk_c86af88` at byte 224.
    #[d4(value)]
    pub unk_c86af88: f32,
    /// Native `unk_396c7d1` at byte 228.
    #[d4(value)]
    pub unk_396c7d1: f32,
    /// Native `unk_515279b` at byte 232.
    #[d4(value)]
    pub unk_515279b: i32,
    /// Native `szLocatorName` at byte 236.
    #[d4(value)]
    pub locator_name: u32,
    /// Native `flLocatorBlend` at byte 240.
    #[d4(value)]
    pub locator_blend: f32,
    /// Unclassified native bytes at offset 244.
    #[d4(value)]
    pub reserved_244: [u8; 4],
}

/// Native `TriggerEventObjectLookAt` (152 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventObjectLookAt {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `dwObjectID` at byte 144.
    #[d4(value)]
    pub object_id: u32,
    /// Native `dwTargetID` at byte 148.
    #[d4(value)]
    pub target_id: u32,
}

/// Native `TriggerEventSetWeaponVisibility` (128 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetWeaponVisibility {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `fAffectOwner` at byte 96.
    #[d4(value)]
    pub affect_owner: i32,
    /// Native `dwActorID` at byte 100.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `eWeaponVisibilityTargetType` at byte 104.
    #[d4(value)]
    pub weapon_visibility_target_type: i32,
    /// Native `bHideMainHand` at byte 108.
    #[d4(value)]
    pub hide_main_hand: i32,
    /// Native `bHideOffhand` at byte 112.
    #[d4(value)]
    pub hide_offhand: i32,
    /// Native `bHideSheathed` at byte 116.
    #[d4(value)]
    pub hide_sheathed: i32,
    /// Native `unk_9f0a4d6` at byte 120.
    #[d4(value)]
    pub unk_9f0a4d6: i32,
    /// Unclassified native bytes at offset 124.
    #[d4(value)]
    pub reserved_124: [u8; 4],
}

/// Native `TriggerEventChangeLookDuration` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventChangeLookDuration {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `dwActorID` at byte 144.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `snoAppearance` at byte 148.
    #[d4(value)]
    pub appearance: i32,
    /// Native `tLookLink` at byte 152.
    pub look_link: LookLink,
    /// Native `fAffectOwner` at byte 156.
    #[d4(value)]
    pub affect_owner: i32,
}

/// Native `TriggerEventSetOtherScenesVisibility` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetOtherScenesVisibility {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `bHide` at byte 96.
    #[d4(value)]
    pub hide: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventSetWeaponVisibilityDuration` (176 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetWeaponVisibilityDuration {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `fAffectOwner` at byte 144.
    #[d4(value)]
    pub affect_owner: i32,
    /// Native `dwActorID` at byte 148.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `unk_94bd986` at byte 152.
    #[d4(value)]
    pub unk_94bd986: i32,
    /// Native `bHideMainHand` at byte 156.
    #[d4(value)]
    pub hide_main_hand: i32,
    /// Native `bHideOffhand` at byte 160.
    #[d4(value)]
    pub hide_offhand: i32,
    /// Native `bHideSheathed` at byte 164.
    #[d4(value)]
    pub hide_sheathed: i32,
    /// Native `unk_9f0a4d6` at byte 168.
    #[d4(value)]
    pub unk_9f0a4d6: i32,
    /// Unclassified native bytes at offset 172.
    #[d4(value)]
    pub reserved_172: [u8; 4],
}

/// Native `TriggerEventSpawnObject` (288 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSpawnObject<'data> {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `dwID` at byte 96.
    #[d4(value)]
    pub id: u32,
    /// Native `snoname` at byte 100.
    #[d4(value)]
    pub snoname: [i32; 2],
    /// Native `tHardpointLinks` at byte 108.
    #[d4(inline_array)]
    pub hardpoint_links: [HardpointLink; 2],
    /// Native `dwTriggerFlags` at byte 124.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 128.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `fRandomLook` at byte 132.
    #[d4(value)]
    pub random_look: i32,
    /// Native `tLookLink` at byte 136.
    pub look_link: LookLink,
    /// Native `fOverrideActorVariant` at byte 140.
    #[d4(value)]
    pub override_actor_variant: i32,
    /// Native `eVariantTypeOverride` at byte 144.
    #[d4(value)]
    pub variant_type_override: i32,
    /// Native `dwSegmentsStep` at byte 148.
    #[d4(value)]
    pub segments_step: u32,
    /// Native `dwShaderMapOverride` at byte 152.
    #[d4(value)]
    pub shader_map_override: u32,
    /// Native `flScale` at byte 156.
    #[d4(value)]
    pub scale: f32,
    /// Native `dwSeed` at byte 160.
    #[d4(value)]
    pub seed: u32,
    /// Native `szSpawnLocatorName` at byte 164.
    #[d4(value)]
    pub spawn_locator_name: u32,
    /// Native `vecScalar` at byte 168.
    #[d4(value)]
    pub vec_scalar: [f32; 2],
    /// Native `transform` at byte 176.
    pub transform: PRTransform,
    /// Native `tEffectGroupHardpointLink` at byte 204.
    pub effect_group_hardpoint_link: HardpointLink,
    /// Native `unk_c7fa18a` at byte 212.
    #[d4(value)]
    pub unk_c7fa18a: [f32; 2],
    /// Native `snoPowerEffectParam` at byte 220.
    #[d4(value)]
    pub power_effect_param: i32,
    /// Native `dwSpawnedActorLife` at byte 224.
    #[d4(value)]
    pub spawned_actor_life: f32,
    /// Native `tNPCComponentInstanceData` at byte 228.
    pub npc_component_instance_data: NPCComponentInstanceData,
    /// Unclassified native bytes at offset 268.
    #[d4(value)]
    pub reserved_268: [u8; 4],
    /// Native `ptLightInstanceData` at byte 272.
    #[d4(array)]
    pub light_instance_data: Vec<LightInstanceData<'data>>,
}

/// Native `TriggerEventEnableRagdollAnimationOverride` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventEnableRagdollAnimationOverride {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
}

/// Native `TriggerEventDetachObject` (128 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventDetachObject {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `fDetachFromOwner` at byte 96.
    #[d4(value)]
    pub detach_from_owner: i32,
    /// Native `dwActorID` at byte 100.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `dwID` at byte 104.
    #[d4(value)]
    pub id: u32,
    /// Native `fEnableRagdoll` at byte 108.
    #[d4(value)]
    pub enable_ragdoll: i32,
    /// Native `tRagdollParams` at byte 112.
    pub ragdoll_params: EnableRagdollParams,
    /// Unclassified native bytes at offset 124.
    #[d4(value)]
    pub reserved_124: [u8; 4],
}

/// Native `TriggerEventSetHelmVisibility` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetHelmVisibility {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `fAffectOwner` at byte 96.
    #[d4(value)]
    pub affect_owner: i32,
    /// Native `dwActorID` at byte 100.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `bHideHelm` at byte 104.
    #[d4(value)]
    pub hide_helm: i32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventObjectMoveToLocator` (200 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventObjectMoveToLocator {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `dwObjectID` at byte 144.
    #[d4(value)]
    pub object_id: u32,
    /// Native `szLocatorName` at byte 148.
    #[d4(value)]
    pub locator_name: u32,
    /// Native `eOrientationMode` at byte 152.
    #[d4(value)]
    pub orientation_mode: i32,
    /// Native `eLookAtTargetType` at byte 156.
    #[d4(value)]
    pub look_at_target_type: i32,
    /// Native `szLookAtTargetLocatorName` at byte 160.
    #[d4(value)]
    pub look_at_target_locator_name: u32,
    /// Native `tHardpointLinkLookAtTarget` at byte 164.
    pub hardpoint_link_look_at_target: HardpointLink,
    /// Native `dwLookAtTargetID` at byte 172.
    #[d4(value)]
    pub look_at_target_id: u32,
    /// Native `eBlendEasing` at byte 176.
    #[d4(value)]
    pub blend_easing: i32,
    /// Native `eMoveRate` at byte 180.
    #[d4(value)]
    pub move_rate: i32,
    /// Native `eDynamicMovementSpeed` at byte 184.
    #[d4(value)]
    pub dynamic_movement_speed: i32,
    /// Native `fUseDynamicMovement` at byte 188.
    #[d4(value)]
    pub use_dynamic_movement: i32,
    /// Native `unk_23237e1` at byte 192.
    #[d4(value)]
    pub unk_23237e1: i32,
    /// Native `unk_f1b6afc` at byte 196.
    #[d4(value)]
    pub unk_f1b6afc: f32,
}

/// Native `TriggerEventObjectKeyFrame` (168 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventObjectKeyFrame<'data> {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `fPlayOnOwner` at byte 96.
    #[d4(value)]
    pub play_on_owner: i32,
    /// Native `dwObjectID` at byte 100.
    #[d4(value)]
    pub object_id: u32,
    /// Native `unk_8989856` at byte 104.
    #[d4(value)]
    pub unk_8989856: i32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
    /// Native `arKeys` at byte 112.
    #[d4(polymorphic = 2863656093)]
    pub keys: Vec<Polymorphic<'data>>,
    /// Native `tCleanup` at byte 136.
    pub cleanup: TriggerCleanup,
    /// Native `flLastKeyTime` at byte 160.
    #[d4(value)]
    pub last_key_time: f32,
    /// Native `flDuration` at byte 164.
    #[d4(value)]
    pub duration: f32,
}

/// Native `TriggerEventSetOcclusionShapeVisibility` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetOcclusionShapeVisibility {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `bHide` at byte 96.
    #[d4(value)]
    pub hide: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventOutroAllObjects` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventOutroAllObjects {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
}

/// Native `TriggerEventObjectTurnToOriginalFacing` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventObjectTurnToOriginalFacing {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `dwObjectID` at byte 96.
    #[d4(value)]
    pub object_id: u32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventChangeLook` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventChangeLook {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `dwActorID` at byte 96.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `snoAppearance` at byte 100.
    #[d4(value)]
    pub appearance: i32,
    /// Native `tLookLink` at byte 104.
    pub look_link: LookLink,
    /// Native `fAffectOwner` at byte 108.
    #[d4(value)]
    pub affect_owner: i32,
}

/// Native `TriggerEventChangeAttachmentHardpointDuration` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventChangeAttachmentHardpointDuration {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `attachmentId` at byte 144.
    #[d4(value)]
    pub attachment_id: u32,
    /// Native `hardpointLink` at byte 148.
    pub hardpoint_link: HardpointLink,
    /// Native `playOnOwner` at byte 156.
    #[d4(value)]
    pub play_on_owner: i32,
}

/// Native `TriggerEventChangeAttachmentHardpoint` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventChangeAttachmentHardpoint {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `attachmentId` at byte 96.
    #[d4(value)]
    pub attachment_id: u32,
    /// Native `hardpointLink` at byte 100.
    pub hardpoint_link: HardpointLink,
    /// Native `playOnOwner` at byte 108.
    #[d4(value)]
    pub play_on_owner: i32,
}

/// Native `TriggerEventObjectMoveToObject` (216 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventObjectMoveToObject {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `dwObjectID` at byte 144.
    #[d4(value)]
    pub object_id: u32,
    /// Native `unk_17909b9` at byte 148.
    #[d4(value)]
    pub unk_17909b9: i32,
    /// Native `tHardpointLinkTarget` at byte 152.
    pub hardpoint_link_target: HardpointLink,
    /// Native `dwTargetID` at byte 160.
    #[d4(value)]
    pub target_id: u32,
    /// Native `eOrientationMode` at byte 164.
    #[d4(value)]
    pub orientation_mode: i32,
    /// Native `eLookAtTargetType` at byte 168.
    #[d4(value)]
    pub look_at_target_type: i32,
    /// Native `szLookAtTargetLocatorName` at byte 172.
    #[d4(value)]
    pub look_at_target_locator_name: u32,
    /// Native `tHardpointLinkLookAtTarget` at byte 176.
    pub hardpoint_link_look_at_target: HardpointLink,
    /// Native `dwLookAtTargetID` at byte 184.
    #[d4(value)]
    pub look_at_target_id: u32,
    /// Native `eBlendEasing` at byte 188.
    #[d4(value)]
    pub blend_easing: i32,
    /// Native `eMoveRate` at byte 192.
    #[d4(value)]
    pub move_rate: i32,
    /// Native `eDynamicMovementSpeed` at byte 196.
    #[d4(value)]
    pub dynamic_movement_speed: i32,
    /// Native `fUseDynamicMovement` at byte 200.
    #[d4(value)]
    pub use_dynamic_movement: i32,
    /// Native `unk_23237e1` at byte 204.
    #[d4(value)]
    pub unk_23237e1: i32,
    /// Native `unk_f1b6afc` at byte 208.
    #[d4(value)]
    pub unk_f1b6afc: f32,
    /// Unclassified native bytes at offset 212.
    #[d4(value)]
    pub reserved_212: [u8; 4],
}

/// Native `TriggerEventObjectFollowSpline` (280 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventObjectFollowSpline {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `tDuration` at byte 96.
    #[d4(value)]
    pub duration: f32,
    /// Native `dwTriggerFlags` at byte 100.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 104.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tCleanup` at byte 108.
    pub cleanup: TriggerCleanup,
    /// Native `nPreviousInTrack` at byte 132.
    #[d4(value)]
    pub previous_in_track: i32,
    /// Native `nNextInTrack` at byte 136.
    #[d4(value)]
    pub next_in_track: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `dwObjectID` at byte 144.
    #[d4(value)]
    pub object_id: u32,
    /// Native `flBlendTime` at byte 148.
    #[d4(value)]
    pub blend_time: f32,
    /// Native `szSplineName` at byte 152.
    #[d4(value)]
    pub spline_name: u32,
    /// Native `eSplineRangeMode` at byte 156.
    #[d4(value)]
    pub spline_range_mode: i32,
    /// Native `szSplineFirstNode` at byte 160.
    #[d4(value)]
    pub spline_first_node: u32,
    /// Native `szSplineLastNode` at byte 164.
    #[d4(value)]
    pub spline_last_node: u32,
    /// Native `flFirstPercent` at byte 168.
    #[d4(value)]
    pub first_percent: f32,
    /// Native `flLastPercent` at byte 172.
    #[d4(value)]
    pub last_percent: f32,
    /// Native `flFirstDistance` at byte 176.
    #[d4(value)]
    pub first_distance: f32,
    /// Native `flLastDistance` at byte 180.
    #[d4(value)]
    pub last_distance: f32,
    /// Native `nSplineFirstNodeIndex` at byte 184.
    #[d4(value)]
    pub spline_first_node_index: i32,
    /// Native `nSplineLastNodeIndex` at byte 188.
    #[d4(value)]
    pub spline_last_node_index: i32,
    /// Native `eEasing` at byte 192.
    #[d4(value)]
    pub easing: i32,
    /// Native `tEasingWeight` at byte 196.
    pub easing_weight: SplineEasingWeight,
    /// Native `eOrientationMode` at byte 208.
    #[d4(value)]
    pub orientation_mode: i32,
    /// Native `transformOffset` at byte 212.
    pub transform_offset: PRTransform,
    /// Native `eLookAtTargetType` at byte 240.
    #[d4(value)]
    pub look_at_target_type: i32,
    /// Native `szLookAtTargetLocatorName` at byte 244.
    #[d4(value)]
    pub look_at_target_locator_name: u32,
    /// Native `tHardpointLinkLookAtTarget` at byte 248.
    pub hardpoint_link_look_at_target: HardpointLink,
    /// Native `dwLookAtTargetID` at byte 256.
    #[d4(value)]
    pub look_at_target_id: u32,
    /// Native `eEasingType` at byte 260.
    #[d4(value)]
    pub easing_type: i32,
    /// Native `eBlendEasing` at byte 264.
    #[d4(value)]
    pub blend_easing: i32,
    /// Native `eFollowSplineInterpolation` at byte 268.
    #[d4(value)]
    pub follow_spline_interpolation: i32,
    /// Native `flInTangentWeight` at byte 272.
    #[d4(value)]
    pub in_tangent_weight: f32,
    /// Native `flOutTangentWeight` at byte 276.
    #[d4(value)]
    pub out_tangent_weight: f32,
}

/// Native `TriggerEventOutroObject` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventOutroObject {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `tConditions` at byte 16.
    pub conditions: TriggerConditions,
    /// Native `fGenerated` at byte 80.
    #[d4(value)]
    pub generated: i32,
    /// Native `fInCreation` at byte 84.
    #[d4(value)]
    pub in_creation: i32,
    /// Native `dwParentActorID` at byte 88.
    #[d4(value)]
    pub parent_actor_id: u32,
    /// Unclassified native bytes at offset 92.
    #[d4(value)]
    pub reserved_92: [u8; 4],
    /// Native `dwID` at byte 96.
    #[d4(value)]
    pub id: u32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}
