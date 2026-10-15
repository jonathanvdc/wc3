//! Native common event records.
use super::*;
/// Native `TriggerEventObserverAdditionalTarget` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventObserverAdditionalTarget {
    /// Native `tHardpointLinkTarget` at byte 0.
    pub hardpoint_link_target: HardpointLink,
    /// Native `dwTargetID` at byte 8.
    #[d4(value)]
    pub target_id: u32,
    /// Native `flWeight` at byte 12.
    #[d4(value)]
    pub weight: f32,
}

/// Native `TriggerEventConstantFlash` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventConstantFlash {
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
    /// Native `tCleanup` at byte 96.
    pub cleanup: TriggerCleanup,
    /// Native `fEnable` at byte 120.
    #[d4(value)]
    pub enable: i32,
    /// Native `rgbaColorAdd` at byte 124.
    #[d4(value)]
    pub rgba_color_add: [u8; 4],
    /// Native `rgbaColorMul` at byte 128.
    #[d4(value)]
    pub rgba_color_mul: [u8; 4],
    /// Native `flConstantFlashIntensity` at byte 132.
    #[d4(value)]
    pub constant_flash_intensity: f32,
    /// Native `flConstantFlashAdditionalSunFactor` at byte 136.
    #[d4(value)]
    pub constant_flash_additional_sun_factor: f32,
    /// Native `eConstantFlashAddFresnel` at byte 140.
    #[d4(value)]
    pub constant_flash_add_fresnel: i32,
    /// Native `eConstantFlashMulFresnel` at byte 144.
    #[d4(value)]
    pub constant_flash_mul_fresnel: i32,
    /// Native `flFresnelPower` at byte 148.
    #[d4(value)]
    pub fresnel_power: f32,
    /// Native `flFresnelSlope` at byte 152.
    #[d4(value)]
    pub fresnel_slope: f32,
    /// Unclassified native bytes at offset 156.
    #[d4(value)]
    pub reserved_156: [u8; 4],
}

/// Native `TriggerEventFadeOtherPlayers` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFadeOtherPlayers {
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
}

/// Native `TriggerEventRelaxCloth` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventRelaxCloth {
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
    /// Native `dwActorID` at byte 100.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `unk_54489ef` at byte 104.
    #[d4(value)]
    pub unk_54489ef: i32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventSetAudioState` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetAudioState {
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
    /// Native `tAudioState` at byte 96.
    pub audio_state: WwiseSwitchValue,
}

/// Native `TriggerEventChangeShaderMapOverrideDuration` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventChangeShaderMapOverrideDuration {
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
    /// Native `dwShaderMapOverride` at byte 148.
    #[d4(value)]
    pub shader_map_override: u32,
    /// Native `fAffectOwner` at byte 152.
    #[d4(value)]
    pub affect_owner: i32,
    /// Unclassified native bytes at offset 156.
    #[d4(value)]
    pub reserved_156: [u8; 4],
}

/// Native `TriggerEventShadow` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventShadow {
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
    /// Native `dwTriggerFlags` at byte 96.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 100.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `bShadow` at byte 104.
    #[d4(value)]
    pub shadow: i32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventShadowDuration` (152 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventShadowDuration {
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
    /// Native `bShadow` at byte 144.
    #[d4(value)]
    pub shadow: i32,
    /// Native `bAffectChildren` at byte 148.
    #[d4(value)]
    pub affect_children: i32,
}

/// Native `TriggerEventChangeClothParameters` (176 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventChangeClothParameters {
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
    /// Native `unk_950c6ac` at byte 100.
    #[d4(value)]
    pub unk_950c6ac: i32,
    /// Native `dwActorID` at byte 104.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `snoNewClothParameters` at byte 108.
    #[d4(value)]
    pub new_cloth_parameters: i32,
    /// Native `szSubObjectName` at byte 112.
    #[d4(value)]
    pub sub_object_name: u32,
    /// Native `flBlendTime` at byte 116.
    #[d4(value)]
    pub blend_time: f32,
    /// Native `paramsOverride` at byte 120.
    pub params_override: ClothParamsOverride,
    /// Unclassified native bytes at offset 172.
    #[d4(value)]
    pub reserved_172: [u8; 4],
}

/// Native `TriggerEventChangeShaderMapOverride` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventChangeShaderMapOverride {
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
    /// Native `dwShaderMapOverride` at byte 100.
    #[d4(value)]
    pub shader_map_override: u32,
    /// Native `fAffectOwner` at byte 104.
    #[d4(value)]
    pub affect_owner: i32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventHitFlashDuration` (200 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventHitFlashDuration {
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
    /// Native `rgbaColorAdd` at byte 144.
    #[d4(value)]
    pub rgba_color_add: [u8; 4],
    /// Native `nHitFlashAddDuration` at byte 148.
    #[d4(value)]
    pub hit_flash_add_duration: f32,
    /// Native `rgbaColorMul` at byte 152.
    #[d4(value)]
    pub rgba_color_mul: [u8; 4],
    /// Native `nHitFlashMulDuration` at byte 156.
    #[d4(value)]
    pub hit_flash_mul_duration: f32,
    /// Native `flHitFlashIntensity` at byte 160.
    #[d4(value)]
    pub hit_flash_intensity: f32,
    /// Native `flHitFlashAdditionalSunFactor` at byte 164.
    #[d4(value)]
    pub hit_flash_additional_sun_factor: f32,
    /// Native `flHitFlashIntensityDuration` at byte 168.
    #[d4(value)]
    pub hit_flash_intensity_duration: f32,
    /// Native `eHitFlashAddFresnel` at byte 172.
    #[d4(value)]
    pub hit_flash_add_fresnel: i32,
    /// Native `eHitFlashMulFresnel` at byte 176.
    #[d4(value)]
    pub hit_flash_mul_fresnel: i32,
    /// Native `eHitFlashCurve` at byte 180.
    #[d4(value)]
    pub hit_flash_curve: i32,
    /// Native `dwID` at byte 184.
    #[d4(value)]
    pub id: u32,
    /// Native `nMaxHitFlash` at byte 188.
    #[d4(value)]
    pub max_hit_flash: u32,
    /// Native `fIsCombatHitFlash` at byte 192.
    #[d4(value)]
    pub is_combat_hit_flash: i32,
    /// Native `unk_b19da4b` at byte 196.
    #[d4(value)]
    pub unk_b19da4b: i32,
}

/// Native `TriggerEventFrostBreath` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFrostBreath {
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
    /// Native `tHardpointLink` at byte 96.
    pub hardpoint_link: HardpointLink,
}

/// Native `TriggerEventAddBaseMountLoadout` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventAddBaseMountLoadout {
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
    /// Native `eCreditsVersion` at byte 96.
    #[d4(value)]
    pub credits_version: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventFlappyJawOverrideDuration` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFlappyJawOverrideDuration {
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
    /// Native `flMaximumThreshold` at byte 152.
    #[d4(value)]
    pub maximum_threshold: f32,
    /// Unclassified native bytes at offset 156.
    #[d4(value)]
    pub reserved_156: [u8; 4],
}

/// Native `TriggerEventDisableConstraint` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventDisableConstraint {
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
    /// Native `tConstraintLink` at byte 96.
    pub constraint_link: ConstraintLink,
    /// Native `tHardpointLink` at byte 100.
    pub hardpoint_link: HardpointLink,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventFootstep` (120 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFootstep {
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
    /// Native `tHardpointLink` at byte 96.
    pub hardpoint_link: HardpointLink,
    /// Native `eFootstepType` at byte 104.
    #[d4(value)]
    pub footstep_type: i32,
    /// Native `eFootstepSide` at byte 108.
    #[d4(value)]
    pub footstep_side: i32,
    /// Native `snoFootstepSoundTable` at byte 112.
    #[d4(value)]
    pub footstep_sound_table: i32,
    /// Native `snoFootstepSurface` at byte 116.
    #[d4(value)]
    pub footstep_surface: i32,
}

/// Native `TriggerEventFadeRemotePlayers` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFadeRemotePlayers {
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
}

/// Native `TriggerEventIGCHideGroup` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventIGCHideGroup {
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
    /// Native `bHideAllGroups` at byte 96.
    #[d4(value)]
    pub hide_all_groups: i32,
    /// Native `nHideGroup` at byte 100.
    #[d4(value)]
    pub hide_group: u16,
    /// Unclassified native bytes at offset 102.
    #[d4(value)]
    pub reserved_102: [u8; 2],
    /// Native `unk_df8ad9c` at byte 104.
    #[d4(value)]
    pub unk_df8ad9c: i32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventHitFlash` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventHitFlash {
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
    /// Native `dwTriggerFlags` at byte 96.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 100.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `rgbaColorAdd` at byte 104.
    #[d4(value)]
    pub rgba_color_add: [u8; 4],
    /// Native `nHitFlashAddDuration` at byte 108.
    #[d4(value)]
    pub hit_flash_add_duration: f32,
    /// Native `rgbaColorMul` at byte 112.
    #[d4(value)]
    pub rgba_color_mul: [u8; 4],
    /// Native `nHitFlashMulDuration` at byte 116.
    #[d4(value)]
    pub hit_flash_mul_duration: f32,
    /// Native `flHitFlashIntensity` at byte 120.
    #[d4(value)]
    pub hit_flash_intensity: f32,
    /// Native `flHitFlashAdditionalSunFactor` at byte 124.
    #[d4(value)]
    pub hit_flash_additional_sun_factor: f32,
    /// Native `flHitFlashIntensityDuration` at byte 128.
    #[d4(value)]
    pub hit_flash_intensity_duration: f32,
    /// Native `eHitFlashAddFresnel` at byte 132.
    #[d4(value)]
    pub hit_flash_add_fresnel: i32,
    /// Native `eHitFlashMulFresnel` at byte 136.
    #[d4(value)]
    pub hit_flash_mul_fresnel: i32,
    /// Native `eHitFlashCurve` at byte 140.
    #[d4(value)]
    pub hit_flash_curve: i32,
    /// Native `dwID` at byte 144.
    #[d4(value)]
    pub id: u32,
    /// Native `nMaxHitFlash` at byte 148.
    #[d4(value)]
    pub max_hit_flash: u32,
    /// Native `fIsCombatHitFlash` at byte 152.
    #[d4(value)]
    pub is_combat_hit_flash: i32,
    /// Native `unk_b19da4b` at byte 156.
    #[d4(value)]
    pub unk_b19da4b: i32,
}

/// Native `TriggerEventFootstepDrag` (152 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFootstepDrag {
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
    /// Native `tHardpointLink` at byte 144.
    pub hardpoint_link: HardpointLink,
}

/// Native `TriggerEventFootstepLift` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFootstepLift {
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
    /// Native `tHardpointLink` at byte 96.
    pub hardpoint_link: HardpointLink,
}

/// Native `TriggerEventDisableCollisionMessages` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventDisableCollisionMessages {
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
    /// Native `dwTriggerFlags` at byte 96.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventSequenceEnded` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSequenceEnded {
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
    /// Native `idEffectGroup` at byte 96.
    #[d4(value)]
    pub id_effect_group: u32,
    /// Native `fLoop` at byte 100.
    #[d4(value)]
    pub loop_: i32,
}

/// Native `TriggerEventChangePhysicsParameters` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventChangePhysicsParameters {
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
    /// Native `snoNewPhysicsParameters` at byte 96.
    #[d4(value)]
    pub new_physics_parameters: i32,
    /// Native `flBlendTime` at byte 100.
    #[d4(value)]
    pub blend_time: f32,
    /// Native `dwActorID` at byte 104.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `fAffectOwner` at byte 108.
    #[d4(value)]
    pub affect_owner: i32,
}

/// Native `TriggerEventTransparency` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventTransparency {
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
    /// Native `dwTriggerFlags` at byte 96.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 100.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `flAlpha` at byte 104.
    #[d4(value)]
    pub alpha: f32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventMsg` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventMsg {
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
    /// Native `eMsgPassMethod` at byte 96.
    #[d4(value)]
    pub msg_pass_method: i32,
    /// Native `dwMsgKey` at byte 100.
    #[d4(value)]
    pub msg_key: i32,
}

/// Native `TriggerEvent` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEvent {
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

/// Native `TriggerEventAddRope` (288 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventAddRope {
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
    /// Native `dwID` at byte 144.
    #[d4(value)]
    pub id: u32,
    /// Native `snoRope` at byte 148.
    #[d4(value)]
    pub rope: i32,
    /// Native `snoPowerEffectParam` at byte 152.
    #[d4(value)]
    pub power_effect_param: i32,
    /// Native `tDefaultRopeAttachPoints` at byte 156.
    #[d4(inline_array)]
    pub default_rope_attach_points: [RopeEventAttachPoint; 2],
    /// Unclassified native bytes at offset 252.
    #[d4(value)]
    pub reserved_252: [u8; 4],
    /// Native `arAdditionalAttachPoints` at byte 256.
    #[d4(array)]
    pub additional_attach_points: Vec<RopeEventAttachPoint>,
    /// Native `vecScalar` at byte 272.
    #[d4(value)]
    pub vec_scalar: [f32; 2],
    /// Native `dwSeed` at byte 280.
    #[d4(value)]
    pub seed: u32,
    /// Unclassified native bytes at offset 284.
    #[d4(value)]
    pub reserved_284: [u8; 4],
}

/// Native `TriggerEventSetWeaponSheathed` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetWeaponSheathed {
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
    /// Native `bSheathed` at byte 104.
    #[d4(value)]
    pub sheathed: i32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventFadeMonsters` (152 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFadeMonsters {
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
    /// Native `bAffectNPCs` at byte 144.
    #[d4(value)]
    pub affect_np_cs: i32,
    /// Unclassified native bytes at offset 148.
    #[d4(value)]
    pub reserved_148: [u8; 4],
}

/// Native `TriggerEventFadePlayers` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFadePlayers {
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
}

/// Native `TriggerEventSetRTCDebugText` (128 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetRTCDebugText<'data> {
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
    /// Native `bOverridePosition` at byte 96.
    #[d4(value)]
    pub override_position: i32,
    /// Native `vecPosition` at byte 100.
    #[d4(value)]
    pub vec_position: [f32; 2],
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
    /// Native `szText` at byte 112.
    #[d4(bytes)]
    pub text: Cow<'data, [u8]>,
}

/// Native `TriggerEventScreenFade` (152 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventScreenFade {
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
    /// Native `rgbaFadeColor` at byte 144.
    #[d4(value)]
    pub rgba_fade_color: [u8; 4],
    /// Unclassified native bytes at offset 148.
    #[d4(value)]
    pub reserved_148: [u8; 4],
}

/// Native `TriggerEventFade` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFade {
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
    /// Native `fFadeOwner` at byte 144.
    #[d4(value)]
    pub fade_owner: i32,
    /// Native `bRollConnection` at byte 148.
    #[d4(value)]
    pub roll_connection: i32,
    /// Native `dwObjectID` at byte 152.
    #[d4(value)]
    pub object_id: u32,
    /// Unclassified native bytes at offset 156.
    #[d4(value)]
    pub reserved_156: [u8; 4],
}

/// Native `TriggerEventNull` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventNull {
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

/// Native `TriggerEventRoutingFunctionKeyFrames` (136 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventRoutingFunctionKeyFrames<'data> {
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
    /// Native `name` at byte 96.
    #[d4(value)]
    pub name: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
    /// Native `arKeys` at byte 104.
    #[d4(polymorphic = 2863656093)]
    pub keys: Vec<Polymorphic<'data>>,
    /// Native `flLastKeyTime` at byte 128.
    #[d4(value)]
    pub last_key_time: f32,
    /// Unclassified native bytes at offset 132.
    #[d4(value)]
    pub reserved_132: [u8; 4],
}

/// Native `TriggerEventPOISkip` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventPOISkip {
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
}

/// Native `TriggerEventPlayMovie` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventPlayMovie {
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
    /// Native `snoMovie` at byte 144.
    #[d4(value)]
    pub movie: i32,
    /// Native `fIntroMovie` at byte 148.
    #[d4(value)]
    pub intro_movie: i32,
    /// Native `unk_26e6b7e` at byte 152.
    #[d4(value)]
    pub unk_26e6b7e: i32,
    /// Unclassified native bytes at offset 156.
    #[d4(value)]
    pub reserved_156: [u8; 4],
}

/// Native `TriggerEventFadeGroup` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFadeGroup {
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
    /// Native `unk_8dea4d9` at byte 96.
    #[d4(value)]
    pub unk_8dea4d9: i32,
    /// Native `nFadeGroup` at byte 100.
    #[d4(value)]
    pub fade_group: i32,
    /// Native `eBehavior` at byte 104.
    #[d4(value)]
    pub behavior: i32,
    /// Native `unk_2b48a58` at byte 108.
    #[d4(value)]
    pub unk_2b48a58: i32,
}

/// Native `TriggerEventSetTimeOfDay` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetTimeOfDay {
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
    /// Native `flTimeOfDay` at byte 96.
    #[d4(value)]
    pub time_of_day: f32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventEnableRagdoll` (120 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventEnableRagdoll {
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
    /// Native `tRagdollParams` at byte 96.
    pub ragdoll_params: EnableRagdollParams,
    /// Native `dwActorID` at byte 108.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `fAffectOwner` at byte 112.
    #[d4(value)]
    pub affect_owner: i32,
    /// Unclassified native bytes at offset 116.
    #[d4(value)]
    pub reserved_116: [u8; 4],
}

/// Native `TriggerEventSetWeaponSheathedDuration` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetWeaponSheathedDuration {
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
    /// Native `bSheathed` at byte 152.
    #[d4(value)]
    pub sheathed: i32,
    /// Unclassified native bytes at offset 156.
    #[d4(value)]
    pub reserved_156: [u8; 4],
}

/// Native `TriggerEventRestoreTimeOfDay` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventRestoreTimeOfDay {
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

/// Native `MsgTriggeredEvent` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct MsgTriggeredEvent<'data> {
    /// Native `dwMsgKey` at byte 0.
    #[d4(value)]
    pub msg_key: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `ptTriggerEvent` at byte 8.
    #[d4(polymorphic = 1976572630)]
    pub trigger_event: Vec<Polymorphic<'data>>,
    /// Native `tCooldown` at byte 32.
    #[d4(value)]
    pub cooldown: f32,
    /// Native `dwID` at byte 36.
    #[d4(value)]
    pub id: u32,
}

/// Native `TriggerEventListenerFollowSpline` (224 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventListenerFollowSpline {
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
    /// Native `szSplineName` at byte 144.
    #[d4(value)]
    pub spline_name: u32,
    /// Native `eSplineRangeMode` at byte 148.
    #[d4(value)]
    pub spline_range_mode: i32,
    /// Native `szSplineFirstNode` at byte 152.
    #[d4(value)]
    pub spline_first_node: u32,
    /// Native `szSplineLastNode` at byte 156.
    #[d4(value)]
    pub spline_last_node: u32,
    /// Native `flFirstPercent` at byte 160.
    #[d4(value)]
    pub first_percent: f32,
    /// Native `flLastPercent` at byte 164.
    #[d4(value)]
    pub last_percent: f32,
    /// Native `flFirstDistance` at byte 168.
    #[d4(value)]
    pub first_distance: f32,
    /// Native `flLastDistance` at byte 172.
    #[d4(value)]
    pub last_distance: f32,
    /// Native `nSplineFirstNodeIndex` at byte 176.
    #[d4(value)]
    pub spline_first_node_index: i32,
    /// Native `nSplineLastNodeIndex` at byte 180.
    #[d4(value)]
    pub spline_last_node_index: i32,
    /// Native `eEasing` at byte 184.
    #[d4(value)]
    pub easing: i32,
    /// Native `tEasingWeight` at byte 188.
    pub easing_weight: SplineEasingWeight,
    /// Native `eOrientationMode` at byte 200.
    #[d4(value)]
    pub orientation_mode: i32,
    /// Native `eEasingType` at byte 204.
    #[d4(value)]
    pub easing_type: i32,
    /// Native `eFollowSplineInterpolation` at byte 208.
    #[d4(value)]
    pub follow_spline_interpolation: i32,
    /// Native `flInTangentWeight` at byte 212.
    #[d4(value)]
    pub in_tangent_weight: f32,
    /// Native `flOutTangentWeight` at byte 216.
    #[d4(value)]
    pub out_tangent_weight: f32,
    /// Unclassified native bytes at offset 220.
    #[d4(value)]
    pub reserved_220: [u8; 4],
}

/// Native `TriggerEventOutroServicesDisable` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventOutroServicesDisable {
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
    /// Native `unk_a82d707` at byte 96.
    #[d4(value)]
    pub unk_a82d707: i32,
    /// Native `flDuration` at byte 100.
    #[d4(value)]
    pub duration: f32,
    /// Native `flDistance` at byte 104.
    #[d4(value)]
    pub distance: f32,
    /// Native `eBlendEasing` at byte 108.
    #[d4(value)]
    pub blend_easing: i32,
}

/// Native `TriggerEventRangedDuration` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventRangedDuration {
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
}
