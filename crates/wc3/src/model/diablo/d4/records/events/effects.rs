//! Native effects event records.
use super::*;
/// Native `TriggerEventIGCLightMode` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventIGCLightMode {
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

/// Native `TriggerEventAddTrail` (200 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventAddTrail {
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
    /// Native `snoTrail` at byte 148.
    #[d4(value)]
    pub trail: i32,
    /// Native `vecScalar` at byte 152.
    #[d4(value)]
    pub vec_scalar: [f32; 2],
    /// Native `trailBase` at byte 160.
    pub trail_base: HardpointLink,
    /// Native `snoPowerEffectParam` at byte 168.
    #[d4(value)]
    pub power_effect_param: i32,
    /// Unclassified native bytes at offset 172.
    #[d4(value)]
    pub reserved_172: [u8; 4],
    /// Native `unk_526c582` at byte 176.
    #[d4(array)]
    pub unk_526c582: Vec<HardpointLink>,
    /// Native `dwSeed` at byte 192.
    #[d4(value)]
    pub seed: u32,
    /// Unclassified native bytes at offset 196.
    #[d4(value)]
    pub reserved_196: [u8; 4],
}

/// Native `TriggerEventAddEffectGroup` (312 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventAddEffectGroup {
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
    /// Native `fOverrideScalarValues` at byte 100.
    #[d4(value)]
    pub override_scalar_values: i32,
    /// Native `fOverridePowerEffectSize` at byte 104.
    #[d4(value)]
    pub override_power_effect_size: i32,
    /// Native `fOverridePowerEffectIntensity` at byte 108.
    #[d4(value)]
    pub override_power_effect_intensity: i32,
    /// Native `fOverridePowerEffectDuration` at byte 112.
    #[d4(value)]
    pub override_power_effect_duration: i32,
    /// Native `fOverridePowerEffectLength` at byte 116.
    #[d4(value)]
    pub override_power_effect_length: i32,
    /// Native `fOverridePowerEffectWidth` at byte 120.
    #[d4(value)]
    pub override_power_effect_width: i32,
    /// Native `fOverridePowerEffectHeight` at byte 124.
    #[d4(value)]
    pub override_power_effect_height: i32,
    /// Native `fOverrideHardpoint` at byte 128.
    #[d4(value)]
    pub override_hardpoint: i32,
    /// Native `fOverrideHardpointsTargetType` at byte 132.
    #[d4(value)]
    pub override_hardpoints_target_type: i32,
    /// Native `fOverrideEffectGroupID` at byte 136.
    #[d4(value)]
    pub override_effect_group_id: i32,
    /// Native `unk_e312e34` at byte 140.
    #[d4(value)]
    pub unk_e312e34: u32,
    /// Native `dwActorID` at byte 144.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `dwID` at byte 148.
    #[d4(value)]
    pub id: u32,
    /// Native `unk_9758f2e` at byte 152.
    #[d4(value)]
    pub unk_9758f2e: i32,
    /// Native `unk_574dd8b` at byte 156.
    #[d4(value)]
    pub unk_574dd8b: i32,
    /// Native `snoEffectGroup` at byte 160.
    #[d4(value)]
    pub effect_group: i32,
    /// Native `tHardpointLinks` at byte 164.
    #[d4(inline_array)]
    pub hardpoint_links: [HardpointLink; 2],
    /// Native `dwTriggerFlags` at byte 180.
    #[d4(value)]
    pub trigger_flags: u32,
    /// Native `dwTriggerFlagsEx` at byte 184.
    #[d4(value)]
    pub trigger_flags_ex: u32,
    /// Native `tEffectGroupAttachPointsOverride` at byte 188.
    #[d4(inline_array)]
    pub effect_group_attach_points_override: [EffectGroupAttachPointOverride; 2],
    /// Native `szSpawnLocatorName` at byte 212.
    #[d4(value)]
    pub spawn_locator_name: u32,
    /// Native `transform` at byte 216.
    pub transform: PRTransform,
    /// Native `tEffectGroupHardpointLink` at byte 244.
    pub effect_group_hardpoint_link: HardpointLink,
    /// Native `snoPowerEffectParam` at byte 252.
    #[d4(value)]
    pub power_effect_param: i32,
    /// Native `vecScale` at byte 256.
    #[d4(value)]
    pub vec_scale: [f32; 3],
    /// Native `vecScalar` at byte 268.
    #[d4(value)]
    pub vec_scalar: [f32; 2],
    /// Native `flPowerEffectSizeOverride` at byte 276.
    #[d4(value)]
    pub power_effect_size_override: f32,
    /// Native `flPowerEffectIntensityOverride` at byte 280.
    #[d4(value)]
    pub power_effect_intensity_override: f32,
    /// Native `flPowerEffectDurationOverride` at byte 284.
    #[d4(value)]
    pub power_effect_duration_override: f32,
    /// Native `flPowerEffectLengthOverride` at byte 288.
    #[d4(value)]
    pub power_effect_length_override: f32,
    /// Native `flPowerEffectWidthOverride` at byte 292.
    #[d4(value)]
    pub power_effect_width_override: f32,
    /// Native `flPowerEffectHeightOverride` at byte 296.
    #[d4(value)]
    pub power_effect_height_override: f32,
    /// Native `unk_a633223` at byte 300.
    #[d4(value)]
    pub unk_a633223: [f32; 2],
    /// Native `dwUniqueID` at byte 308.
    #[d4(value)]
    pub unique_id: u32,
}

/// Native `TriggerEventSetLightingLabelOverride` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetLightingLabelOverride {
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
    /// Native `gbidLabelOverride` at byte 96.
    #[d4(value)]
    pub label_override: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventOutroEffectGroup` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventOutroEffectGroup {
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
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventFootstepTrail` (152 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFootstepTrail {
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
