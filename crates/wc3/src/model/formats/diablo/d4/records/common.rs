//! Native common records.
use super::*;
/// Native `WwiseSwitchValue` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct WwiseSwitchValue {
    /// Native `tGroup` at byte 0.
    pub group: WwiseName,
    /// Native `tValue` at byte 4.
    pub value: WwiseName,
}

/// Native `EffectGroupOverrideByVisualRemap` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct EffectGroupOverrideByVisualRemap {
    /// Native `snoVisualRemap` at byte 0.
    #[d4(value)]
    pub visual_remap: i32,
    /// Native `snoEffectGroup` at byte 4.
    #[d4(value)]
    pub effect_group: i32,
}

/// Native `EffectGroupAttachPointOverride` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct EffectGroupAttachPointOverride {
    /// Native `tTargetLocatorName` at byte 0.
    #[d4(value)]
    pub target_locator_name: u32,
    /// Native `eTargetType` at byte 4.
    #[d4(value)]
    pub target_type: i32,
    /// Native `dwAttachedActorID` at byte 8.
    #[d4(value)]
    pub attached_actor_id: u32,
}

/// Native `GeoChunkIndexBuffer` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct GeoChunkIndexBuffer<'data> {
    /// Native `pdwChunkIndices` at byte 0.
    #[d4(array)]
    pub pdw_chunk_indices: ZeroVec<'data, u16>,
    /// Native `ibid` at byte 16.
    #[d4(value)]
    pub ibid: i32,
    /// Native `fOptional` at byte 20.
    #[d4(value)]
    pub optional: i32,
}

/// Native `BlendSettings` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct BlendSettings {
    /// Native `flBlendTime` at byte 0.
    #[d4(value)]
    pub blend_time: f32,
    /// Native `eBlendCurve` at byte 4.
    #[d4(value)]
    pub blend_curve: i32,
}

/// Native `NPCComponentHardpointOverride` (28 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct NPCComponentHardpointOverride {
    /// Native `tHardpointOverride` at byte 0.
    pub hardpoint_override: HardpointLink,
    /// Native `positionOffset` at byte 8.
    #[d4(value)]
    pub position_offset: [f32; 3],
    /// Native `eComponentType` at byte 20.
    #[d4(value)]
    pub component_type: i32,
    /// Native `szComponent` at byte 24.
    #[d4(value)]
    pub component: u32,
}

/// Native `WwiseName` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct WwiseName {
    /// Native `uID` at byte 0.
    #[d4(value)]
    pub u_id: u32,
}

/// Native `RopeEventAttachPoint` (48 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct RopeEventAttachPoint {
    /// Native `tHardpointLink` at byte 0.
    pub hardpoint_link: HardpointLink,
    /// Native `tTargetLocatorName` at byte 8.
    #[d4(value)]
    pub target_locator_name: u32,
    /// Native `eTargetType` at byte 12.
    #[d4(value)]
    pub target_type: i32,
    /// Native `dwAttachedActorID` at byte 16.
    #[d4(value)]
    pub attached_actor_id: u32,
    /// Native `transformLocalOffset` at byte 20.
    pub transform_local_offset: PRTransform,
}

/// Native `TranslationCurve` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TranslationCurve<'data> {
    /// Native `ptKeysComp` at byte 0.
    #[d4(array)]
    pub keys_comp: ZeroVec<'data, u8>,
}

/// Native `EnableRagdollParams` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct EnableRagdollParams {
    /// Native `flRagdollVelocityFactor` at byte 0.
    #[d4(value)]
    pub ragdoll_velocity_factor: f32,
    /// Native `flRagdollMomentumFactor` at byte 4.
    #[d4(value)]
    pub ragdoll_momentum_factor: f32,
    /// Native `flRagdollActorMomentumFactor` at byte 8.
    #[d4(value)]
    pub ragdoll_actor_momentum_factor: f32,
}

/// Native `StringLabelHandleEx` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct StringLabelHandleEx {
    /// Native `snoStringList` at byte 0.
    #[d4(value)]
    pub string_list: i32,
    /// Native `szLabel` at byte 4.
    #[d4(value)]
    pub label: u32,
}

/// Native `MonsterFixedAffixData` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct MonsterFixedAffixData {
    /// Native `snoAffix` at byte 0.
    #[d4(value)]
    pub affix: i32,
    /// Native `nMinLevelReq` at byte 4.
    #[d4(value)]
    pub min_level_req: i32,
}

/// Native `SurfaceGrid` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SurfaceGrid<'data> {
    /// Native `grid` at byte 0.
    #[d4(array)]
    pub grid: ZeroVec<'data, u16>,
    /// Native `arPrimitives` at byte 16.
    #[d4(array)]
    pub primitives: Vec<SurfaceGridPrimitive>,
    /// Native `aabbBounds` at byte 32.
    pub aabb_bounds: Bounds,
    /// Native `dwRows` at byte 56.
    #[d4(value)]
    pub rows: u32,
    /// Native `dwColumns` at byte 60.
    #[d4(value)]
    pub columns: u32,
}

/// Native `TriggerConditions` (64 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerConditions {
    /// Native `flChance` at byte 0.
    #[d4(value)]
    pub chance: f32,
    /// Native `bMute` at byte 4.
    #[d4(value)]
    pub mute: i32,
    /// Native `bMuteExternal` at byte 8.
    #[d4(value)]
    pub mute_external: i32,
    /// Native `tDelay` at byte 12.
    #[d4(value)]
    pub delay: f32,
    /// Native `tImpactVelocityRange` at byte 16.
    #[d4(value)]
    pub impact_velocity_range: [f32; 2],
    /// Native `tImpactWeightRange` at byte 24.
    #[d4(value)]
    pub impact_weight_range: [f32; 2],
    /// Native `eMaterial` at byte 32.
    #[d4(value)]
    pub material: i32,
    /// Native `dwConditionFlags` at byte 36.
    #[d4(value)]
    pub condition_flags: u32,
    /// Native `ePlayerCountCondition` at byte 40.
    #[d4(value)]
    pub player_count_condition: i32,
    /// Native `ePerformanceBehavior` at byte 44.
    #[d4(value)]
    pub performance_behavior: i32,
    /// Native `unk_b41b571` at byte 48.
    #[d4(array)]
    pub unk_b41b571: Vec<TrackConditions>,
}

/// Native `ParametricSettings` (128 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ParametricSettings {
    /// Native `fCavity` at byte 0.
    #[d4(value)]
    pub cavity: i32,
    /// Native `eCavityMode` at byte 4.
    #[d4(value)]
    pub cavity_mode: i32,
    /// Native `flCavityTolerance` at byte 8.
    #[d4(value)]
    pub cavity_tolerance: f32,
    /// Native `flCavitySmoothness` at byte 12.
    #[d4(value)]
    pub cavity_smoothness: f32,
    /// Native `fHeight` at byte 16.
    #[d4(value)]
    pub height: i32,
    /// Native `flHeightMin` at byte 20.
    #[d4(value)]
    pub height_min: f32,
    /// Native `flHeightMinSmoothness` at byte 24.
    #[d4(value)]
    pub height_min_smoothness: f32,
    /// Native `flHeightMax` at byte 28.
    #[d4(value)]
    pub height_max: f32,
    /// Native `flHeightMaxSmoothness` at byte 32.
    #[d4(value)]
    pub height_max_smoothness: f32,
    /// Native `fSlope` at byte 36.
    #[d4(value)]
    pub slope: i32,
    /// Native `flSlopeMin` at byte 40.
    #[d4(value)]
    pub slope_min: f32,
    /// Native `flSlopeMinSmoothness` at byte 44.
    #[d4(value)]
    pub slope_min_smoothness: f32,
    /// Native `flSlopeMax` at byte 48.
    #[d4(value)]
    pub slope_max: f32,
    /// Native `flSlopeMaxSmoothness` at byte 52.
    #[d4(value)]
    pub slope_max_smoothness: f32,
    /// Native `fDirection` at byte 56.
    #[d4(value)]
    pub direction: i32,
    /// Native `flDirectionAltitude` at byte 60.
    #[d4(value)]
    pub direction_altitude: f32,
    /// Native `flDirectionAzimuth` at byte 64.
    #[d4(value)]
    pub direction_azimuth: f32,
    /// Native `flDirectionTolerance` at byte 68.
    #[d4(value)]
    pub direction_tolerance: f32,
    /// Native `flDirectionSmoothness` at byte 72.
    #[d4(value)]
    pub direction_smoothness: f32,
    /// Native `fNoise` at byte 76.
    #[d4(value)]
    pub noise: i32,
    /// Native `flNoiseFrequency` at byte 80.
    #[d4(value)]
    pub noise_frequency: f32,
    /// Native `flNoiseTileDistance` at byte 84.
    #[d4(value)]
    pub noise_tile_distance: f32,
    /// Native `flNoiseMin` at byte 88.
    #[d4(value)]
    pub noise_min: f32,
    /// Native `flNoiseMax` at byte 92.
    #[d4(value)]
    pub noise_max: f32,
    /// Native `vNoiseOffset` at byte 96.
    #[d4(value)]
    pub noise_offset: [f32; 2],
    /// Native `fDistanceToWater` at byte 104.
    #[d4(value)]
    pub distance_to_water: i32,
    /// Native `flDistanceToWaterMin` at byte 108.
    #[d4(value)]
    pub distance_to_water_min: f32,
    /// Native `flDistanceToWaterMinSmoothness` at byte 112.
    #[d4(value)]
    pub distance_to_water_min_smoothness: f32,
    /// Native `flDistanceToWaterMax` at byte 116.
    #[d4(value)]
    pub distance_to_water_max: f32,
    /// Native `flDistanceToWaterMaxSmoothness` at byte 120.
    #[d4(value)]
    pub distance_to_water_max_smoothness: f32,
    /// Native `fInvert` at byte 124.
    #[d4(value)]
    pub invert: i32,
}

/// Native `ConversationNodeReference` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ConversationNodeReference {
    /// Native `snoConversation` at byte 0.
    #[d4(value)]
    pub conversation: i32,
    /// Native `dwNodeID` at byte 4.
    #[d4(value)]
    pub node_id: u32,
}

/// Native `LightStateParams` (800 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct LightStateParams {
    /// Native `pathRadius` at byte 0.
    pub path_radius: InterpolationPathFloat,
    /// Native `pathSourceRadius` at byte 56.
    pub path_source_radius: InterpolationPathFloat,
    /// Native `pathSpecularSoftRadius` at byte 112.
    pub path_specular_soft_radius: InterpolationPathFloat,
    /// Native `pathFalloffBias` at byte 168.
    pub path_falloff_bias: InterpolationPathFloat,
    /// Native `unk_500fe1e` at byte 224.
    pub unk_500fe1e: InterpolationPathFloat,
    /// Native `pathWidth` at byte 280.
    pub path_width: InterpolationPathFloat,
    /// Native `pathHotspot` at byte 336.
    pub path_hotspot: InterpolationPathFloat,
    /// Native `pathFalloff` at byte 392.
    pub path_falloff: InterpolationPathFloat,
    /// Native `unk_7f495e` at byte 448.
    #[d4(value)]
    pub unk_7f495e: i32,
    /// Unclassified native bytes at offset 452.
    #[d4(value)]
    pub reserved_452: [u8; 4],
    /// Native `pathLightFalloffExponent` at byte 456.
    pub path_light_falloff_exponent: InterpolationPathFloat,
    /// Native `pathColor` at byte 512.
    pub path_color: InterpolationPathRGBAColor,
    /// Native `pathIntensity` at byte 568.
    pub path_intensity: InterpolationPathFloat,
    /// Native `flIntensityForFog` at byte 624.
    #[d4(value)]
    pub intensity_for_fog: f32,
    /// Native `unk_114300f` at byte 628.
    #[d4(value)]
    pub unk_114300f: f32,
    /// Native `pathSunIntensityFactor` at byte 632.
    pub path_sun_intensity_factor: InterpolationPathFloat,
    /// Native `pathShadowIntensity` at byte 688.
    pub path_shadow_intensity: InterpolationPathFloat,
    /// Native `pathPivotOffset` at byte 744.
    pub path_pivot_offset: VectorPath,
}

/// Native `FrameAnim` (36 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct FrameAnim {
    /// Native `ePlaybackMode` at byte 0.
    #[d4(value)]
    pub playback_mode: i32,
    /// Native `tFrameRateRange` at byte 4.
    #[d4(value)]
    pub frame_rate_range: [f32; 2],
    /// Native `tStartFrameRange` at byte 12.
    #[d4(value)]
    pub start_frame_range: [u32; 2],
    /// Native `dwFrameRandomSeed` at byte 20.
    #[d4(value)]
    pub frame_random_seed: u32,
    /// Native `tAnimationDelayRange` at byte 24.
    #[d4(value)]
    pub animation_delay_range: [f32; 2],
    /// Native `unk_ca71af4` at byte 32.
    #[d4(value)]
    pub unk_ca71af4: i32,
}

/// Native `Type_4174a154` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Type4174a154 {
    /// Native `rtpcName` at byte 0.
    pub rtpc_name: WwiseName,
    /// Native `flValue` at byte 4.
    #[d4(value)]
    pub value: f32,
}

/// Native `ScaleCurve` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ScaleCurve<'data> {
    /// Native `ptKeysComp` at byte 0.
    #[d4(array)]
    pub keys_comp: ZeroVec<'data, u8>,
}

/// Native `GroundNormalTests` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct GroundNormalTests {
    /// Native `bEnabled` at byte 0.
    #[d4(value)]
    pub enabled: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `ptGroundTests` at byte 8.
    #[d4(array)]
    pub ground_tests: Vec<GroundTestPosition>,
}

/// Native `HairAndBeardState` (124 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct HairAndBeardState {
    /// Native `eHairState` at byte 0.
    #[d4(value)]
    pub hair_state: i32,
    /// Native `eBeardState` at byte 4.
    #[d4(value)]
    pub beard_state: i32,
    /// Native `eJewelryState` at byte 8.
    #[d4(value)]
    pub jewelry_state: i32,
    /// Native `ePersonaState` at byte 12.
    #[d4(value)]
    pub persona_state: i32,
    /// Native `eEyeState` at byte 16.
    #[d4(value)]
    pub eye_state: i32,
    /// Native `eMouthState` at byte 20.
    #[d4(value)]
    pub mouth_state: i32,
    /// Native `bUsesHairClippingVolume` at byte 24.
    #[d4(value)]
    pub uses_hair_clipping_volume: i32,
    /// Native `eHairCullShape` at byte 28.
    #[d4(value)]
    pub hair_cull_shape: i32,
    /// Native `tHairClippingSphere` at byte 32.
    pub hair_clipping_sphere: Sphere,
    /// Native `tHairClippingCylinder` at byte 48.
    pub hair_clipping_cylinder: AxialCylinder,
    /// Native `tHairCylinderRotation` at byte 68.
    pub hair_cylinder_rotation: BcQuat,
    /// Native `tHairClippingBox` at byte 84.
    pub hair_clipping_box: OBB,
}

/// Native `Triangle2D` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Triangle2D {
    /// Native `wp0` at byte 0.
    #[d4(value)]
    pub value_0: [f32; 2],
    /// Native `wp1` at byte 8.
    #[d4(value)]
    pub value_1: [f32; 2],
    /// Native `wp2` at byte 16.
    #[d4(value)]
    pub value_2: [f32; 2],
}

/// Native `MonsterDamageStateThresholds` (52 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct MonsterDamageStateThresholds {
    /// Native `eBreaksOn` at byte 0.
    #[d4(value)]
    pub breaks_on: i32,
    /// Native `unk_e714dff` at byte 4.
    #[d4(value)]
    pub unk_e714dff: f32,
    /// Native `snoEffect` at byte 8.
    #[d4(value)]
    pub effect: i32,
    /// Native `transform` at byte 12.
    pub transform: PRTransform,
    /// Native `vScale` at byte 40.
    #[d4(value)]
    pub scale: [f32; 3],
}

/// Native `PolytopeData` (128 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct PolytopeData<'data> {
    /// Native `polytopeDataMirror` at byte 0.
    pub polytope_data_mirror: DmPolytopeDataMirror,
    /// Native `ptVertices` at byte 64.
    #[d4(array)]
    pub vertices: ZeroVec<'data, [f32; 3]>,
    /// Native `ptPlanes` at byte 80.
    #[d4(array)]
    pub planes: Vec<DmPlaneMirror>,
    /// Native `ptSubEdges` at byte 96.
    #[d4(array)]
    pub sub_edges: Vec<DmSubEdgeMirror>,
    /// Native `ptFaceSubEdges` at byte 112.
    #[d4(array)]
    pub face_sub_edges: ZeroVec<'data, u8>,
}

/// Native `GroundTestPosition` (20 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct GroundTestPosition {
    /// Native `tBone` at byte 0.
    pub bone: IKBone,
    /// Native `wpPosOffset` at byte 4.
    #[d4(value)]
    pub pos_offset: [f32; 3],
    /// Native `flGroundDistance` at byte 16.
    #[d4(value)]
    pub ground_distance: f32,
}

/// Native `DecalLookVariantMap` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct DecalLookVariantMap {
    /// Native `dwLookVariant` at byte 0.
    #[d4(value)]
    pub look_variant: u32,
    /// Native `snoMaterialOverride` at byte 4.
    #[d4(value)]
    pub material_override: i32,
}

/// Native `AIPropPowerPairing` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AIPropPowerPairing {
    /// Native `snoPowerActivator` at byte 0.
    #[d4(value)]
    pub power_activator: i32,
    /// Native `snoPowerSelf` at byte 4.
    #[d4(value)]
    pub power_self: i32,
    /// Native `snoSummonedEncounterOverridePower` at byte 8.
    #[d4(value)]
    pub summoned_encounter_override_power: i32,
    /// Native `flDefaultActivateRange` at byte 12.
    #[d4(value)]
    pub default_activate_range: f32,
    /// Native `ePowerSyncType` at byte 16.
    #[d4(value)]
    pub power_sync_type: i32,
    /// Native `unk_c0c1492` at byte 20.
    #[d4(value)]
    pub unk_c0c1492: i32,
    /// Native `sPropPairingName` at byte 24.
    #[d4(value)]
    pub s_prop_pairing_name: u32,
    /// Native `unk_c7bd259` at byte 28.
    #[d4(value)]
    pub unk_c7bd259: u32,
}

/// Native `HitReactionOverride` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct HitReactionOverride {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `snoInputPower` at byte 16.
    #[d4(value)]
    pub input_power: i32,
    /// Native `snoOverridePower` at byte 20.
    #[d4(value)]
    pub override_power: i32,
}

/// Native `PersonaSkinColor` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct PersonaSkinColor<'data> {
    /// Native `eBase` at byte 0.
    #[d4(value)]
    pub base: i32,
    /// Native `flHue` at byte 4.
    #[d4(value)]
    pub hue: f32,
    /// Native `flSaturation` at byte 8.
    #[d4(value)]
    pub saturation: f32,
    /// Native `flValue` at byte 12.
    #[d4(value)]
    pub value: f32,
    /// Native `flDarken` at byte 16.
    #[d4(value)]
    pub darken: f32,
    /// Native `rgbaUIDisplayColor` at byte 20.
    #[d4(value)]
    pub rgba_ui_display_color: [u8; 4],
    /// Native `szLabel` at byte 24.
    #[d4(bytes)]
    pub label: Cow<'data, [u8]>,
}

/// Native `Type_72081710` (208 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Type72081710 {
    /// Native `unk_9807ffc` at byte 0.
    #[d4(value)]
    pub unk_9807ffc: f32,
    /// Native `unk_e3bcdd9` at byte 4.
    #[d4(value)]
    pub unk_e3bcdd9: f32,
    /// Native `unk_959fcde` at byte 8.
    #[d4(value)]
    pub unk_959fcde: f32,
    /// Native `unk_8ea8e1a` at byte 12.
    #[d4(value)]
    pub unk_8ea8e1a: f32,
    /// Native `unk_129b8df` at byte 16.
    #[d4(value)]
    pub unk_129b8df: f32,
    /// Native `unk_38138dc` at byte 20.
    #[d4(value)]
    pub unk_38138dc: f32,
    /// Native `unk_d04ea13` at byte 24.
    #[d4(value)]
    pub unk_d04ea13: f32,
    /// Native `unk_15de174` at byte 28.
    #[d4(value)]
    pub unk_15de174: f32,
    /// Native `unk_6192f51` at byte 32.
    #[d4(value)]
    pub unk_6192f51: f32,
    /// Native `unk_d00a056` at byte 36.
    #[d4(value)]
    pub unk_d00a056: f32,
    /// Native `unk_2ed848a` at byte 40.
    #[d4(value)]
    pub unk_2ed848a: f32,
    /// Native `unk_f597cdf` at byte 44.
    #[d4(value)]
    pub unk_f597cdf: f32,
    /// Native `unk_db72561` at byte 48.
    #[d4(value)]
    pub unk_db72561: f32,
    /// Native `unk_8f70873` at byte 52.
    #[d4(value)]
    pub unk_8f70873: f32,
    /// Native `unk_754b0f5` at byte 56.
    #[d4(value)]
    pub unk_754b0f5: f32,
    /// Native `unk_4ce8a9b` at byte 60.
    #[d4(value)]
    pub unk_4ce8a9b: f32,
    /// Native `unk_cbaf386` at byte 64.
    #[d4(value)]
    pub unk_cbaf386: f32,
    /// Native `unk_e7daef4` at byte 68.
    #[d4(value)]
    pub unk_e7daef4: f32,
    /// Native `unk_497f7bf` at byte 72.
    #[d4(value)]
    pub unk_497f7bf: f32,
    /// Native `unk_d19c35c` at byte 76.
    #[d4(value)]
    pub unk_d19c35c: f32,
    /// Native `unk_5456aec` at byte 80.
    #[d4(value)]
    pub unk_5456aec: f32,
    /// Native `unk_b918f5a` at byte 84.
    #[d4(value)]
    pub unk_b918f5a: f32,
    /// Native `unk_57d4c7e` at byte 88.
    #[d4(value)]
    pub unk_57d4c7e: f32,
    /// Native `unk_8a373c0` at byte 92.
    #[d4(value)]
    pub unk_8a373c0: f32,
    /// Native `unk_a1812c5` at byte 96.
    #[d4(value)]
    pub unk_a1812c5: f32,
    /// Native `unk_7124f86` at byte 100.
    #[d4(value)]
    pub unk_7124f86: f32,
    /// Native `unk_2e0c8f7` at byte 104.
    #[d4(value)]
    pub unk_2e0c8f7: f32,
    /// Native `unk_7d8fd49` at byte 108.
    #[d4(value)]
    pub unk_7d8fd49: f32,
    /// Native `unk_8a6d88` at byte 112.
    #[d4(value)]
    pub unk_8a6d88: f32,
    /// Native `unk_aeea64d` at byte 116.
    #[d4(value)]
    pub unk_aeea64d: f32,
    /// Native `unk_b918f5b` at byte 120.
    #[d4(value)]
    pub unk_b918f5b: f32,
    /// Native `unk_57d4c7f` at byte 124.
    #[d4(value)]
    pub unk_57d4c7f: f32,
    /// Native `unk_2e0c8f8` at byte 128.
    #[d4(value)]
    pub unk_2e0c8f8: f32,
    /// Native `unk_7d8fd4a` at byte 132.
    #[d4(value)]
    pub unk_7d8fd4a: f32,
    /// Native `unk_8a6d89` at byte 136.
    #[d4(value)]
    pub unk_8a6d89: f32,
    /// Native `unk_aeea64e` at byte 140.
    #[d4(value)]
    pub unk_aeea64e: f32,
    /// Native `unk_a09e1ed` at byte 144.
    #[d4(value)]
    pub unk_a09e1ed: f32,
    /// Native `unk_d28a307` at byte 148.
    #[d4(value)]
    pub unk_d28a307: f32,
    /// Native `unk_cc5d963` at byte 152.
    #[d4(value)]
    pub unk_cc5d963: f32,
    /// Native `unk_131818d` at byte 156.
    #[d4(value)]
    pub unk_131818d: f32,
    /// Native `unk_2352a09` at byte 160.
    #[d4(value)]
    pub unk_2352a09: f32,
    /// Native `unk_a09e1ee` at byte 164.
    #[d4(value)]
    pub unk_a09e1ee: f32,
    /// Native `unk_d28a308` at byte 168.
    #[d4(value)]
    pub unk_d28a308: f32,
    /// Native `unk_cc5d964` at byte 172.
    #[d4(value)]
    pub unk_cc5d964: f32,
    /// Native `unk_131818e` at byte 176.
    #[d4(value)]
    pub unk_131818e: f32,
    /// Native `unk_2352a0a` at byte 180.
    #[d4(value)]
    pub unk_2352a0a: f32,
    /// Native `unk_8522f61` at byte 184.
    #[d4(value)]
    pub unk_8522f61: f32,
    /// Native `unk_1bb10d9` at byte 188.
    #[d4(value)]
    pub unk_1bb10d9: f32,
    /// Native `unk_426ab57` at byte 192.
    #[d4(value)]
    pub unk_426ab57: f32,
    /// Native `flPad0` at byte 196.
    #[d4(value)]
    pub pad0: f32,
    /// Native `flPad1` at byte 200.
    #[d4(value)]
    pub pad1: f32,
    /// Native `flPad2` at byte 204.
    #[d4(value)]
    pub pad2: f32,
}

/// Native `TriggerCleanup` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerCleanup {
    /// Native `eOnOwnerDeath` at byte 0.
    #[d4(value)]
    pub on_owner_death: i32,
    /// Native `eOnAnimationEnd` at byte 4.
    #[d4(value)]
    pub on_animation_end: i32,
    /// Native `eOnSequenceEnd` at byte 8.
    #[d4(value)]
    pub on_sequence_end: i32,
    /// Native `eOnOwnerResurrection` at byte 12.
    #[d4(value)]
    pub on_owner_resurrection: i32,
    /// Native `eOnOwnerDeletion` at byte 16.
    #[d4(value)]
    pub on_owner_deletion: i32,
    /// Native `eOnOwnerPaused` at byte 20.
    #[d4(value)]
    pub on_owner_paused: i32,
}

/// Native `OctreePrimitive` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct OctreePrimitive {
    /// Native `uUserData` at byte 0.
    #[d4(value)]
    pub u_user_data: u64,
}

/// Native `SceneData` (72 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SceneData<'data> {
    /// Native `ptCollisionHeightFields` at byte 0.
    #[d4(array, external)]
    pub collision_height_fields: Vec<CollisionHeightField<'data>>,
    /// Native `unk_b3a0ae7` at byte 16.
    #[d4(value)]
    pub unk_b3a0ae7: i32,
    /// Unclassified native bytes at offset 20.
    #[d4(value)]
    pub reserved_20: [u8; 4],
    /// Native `ptSurfaceOctree` at byte 24.
    #[d4(array, external)]
    pub surface_octree: Vec<Octree>,
    /// Native `ptSurfaceGrid` at byte 40.
    #[d4(array, external)]
    pub surface_grid: Vec<SurfaceGrid<'data>>,
    /// Native `ptFoliageGrid` at byte 56.
    #[d4(array, external)]
    pub foliage_grid: Vec<FoliageGrid<'data>>,
}

/// Native `FxAnimatedInfluences` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct FxAnimatedInfluences {
    /// Native `eIndices` at byte 0.
    #[d4(value)]
    pub indices: [i32; 8],
}

/// Native `AIBroadcast` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AIBroadcast {
    /// Native `snoAIState` at byte 0.
    #[d4(value)]
    pub ai_state: i32,
    /// Native `flRadius` at byte 4.
    #[d4(value)]
    pub radius: f32,
    /// Native `snoCondition` at byte 8.
    #[d4(value)]
    pub condition: i32,
}

/// Native `NPCComponentGroup` (44 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct NPCComponentGroup {
    /// Native `sName` at byte 0.
    #[d4(value)]
    pub s_name: u32,
    /// Native `arComponents` at byte 4.
    #[d4(value)]
    pub components: [u32; 4],
    /// Native `tColorSet` at byte 20.
    pub color_set: NPCColorSet,
}

/// Native `OctreeCube` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct OctreeCube {
    /// Native `wpOrigin` at byte 0.
    #[d4(value)]
    pub origin: [f32; 3],
    /// Native `wdSize` at byte 12.
    #[d4(value)]
    pub wd_size: f32,
}

/// Native `OctreeLeaf` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct OctreeLeaf {
    /// Native `nPrimitiveIndex` at byte 0.
    #[d4(value)]
    pub primitive_index: i32,
    /// Native `nPrimitiveCount` at byte 4.
    #[d4(value)]
    pub primitive_count: i32,
}

/// Native `OctreeNode` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct OctreeNode {
    /// Native `pdwChildren` at byte 0.
    #[d4(value)]
    pub pdw_children: [u32; 8],
}

/// Native `AppBone` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AppBone {
    /// Native `dwBoneHash` at byte 0.
    #[d4(value)]
    pub bone_hash: u32,
}

/// Native `VectorPath` (56 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct VectorPath {
    /// Native `tPathData` at byte 0.
    pub path_data: InterpolationPathBcVec3,
}

/// Native `NPCColorSet` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct NPCColorSet {
    /// Native `arEquipmentDyes` at byte 0.
    #[d4(value)]
    pub equipment_dyes: [i32; 4],
    /// Native `nHairColor` at byte 16.
    #[d4(value)]
    pub hair_color: i32,
    /// Native `nSkinColor` at byte 20.
    #[d4(value)]
    pub skin_color: i32,
}

/// Native `PlayerAnimSetOverride` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct PlayerAnimSetOverride {
    /// Native `snoPlayerActor` at byte 0.
    #[d4(value)]
    pub player_actor: i32,
    /// Native `snoAnimSetOverride` at byte 4.
    #[d4(value)]
    pub anim_set_override: i32,
}

/// Native `ValidComponent` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ValidComponent {
    /// Native `tComponent` at byte 0.
    #[d4(value)]
    pub component: u32,
}

/// Native `EffectGroupKeyData` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct EffectGroupKeyData {
    /// Unclassified native bytes at offset 0.
    #[d4(value)]
    pub reserved_0: [u8; 8],
    /// Native `dwType` at byte 8.
    #[d4(value)]
    pub type_: u32,
    /// Unclassified native bytes at offset 12.
    #[d4(value)]
    pub reserved_12: [u8; 4],
    /// Native `eEffectGroupKeyDataType` at byte 16.
    #[d4(value)]
    pub effect_group_key_data_type: i32,
    /// Unclassified native bytes at offset 20.
    #[d4(value)]
    pub reserved_20: [u8; 4],
    /// Native `arKeyItems` at byte 24.
    #[d4(array)]
    pub key_items: Vec<EffectGroupKey>,
}

/// Native `HairStateGenderOverrides` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct HairStateGenderOverrides {
    /// Native `eHairStateGenderOverride` at byte 0.
    #[d4(value)]
    pub hair_state_gender_override: i32,
    /// Native `eBeardStateGenderOverride` at byte 4.
    #[d4(value)]
    pub beard_state_gender_override: i32,
    /// Native `eJewleryStateGenderOverride` at byte 8.
    #[d4(value)]
    pub jewlery_state_gender_override: i32,
    /// Native `ePersonaStateGenderOverride` at byte 12.
    #[d4(value)]
    pub persona_state_gender_override: i32,
    /// Native `eEyeStateGenderOverride` at byte 16.
    #[d4(value)]
    pub eye_state_gender_override: i32,
    /// Native `eMouthStateGenderOverride` at byte 20.
    #[d4(value)]
    pub mouth_state_gender_override: i32,
}

/// Native `NPCItems` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct NPCItems {
    /// Native `snoMainHand` at byte 0.
    #[d4(value)]
    pub main_hand: i32,
    /// Native `snoOffHand` at byte 4.
    #[d4(value)]
    pub off_hand: i32,
}

/// Native `GeoSet` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct GeoSet<'data> {
    /// Native `ptSubObjects` at byte 0.
    #[d4(array)]
    pub sub_objects: Vec<SubObject<'data>>,
    /// Native `ptBaseSubObjectIndexMap` at byte 16.
    #[d4(array)]
    pub base_sub_object_index_map: ZeroVec<'data, i32>,
    /// Native `flLODDistance` at byte 32.
    #[d4(value)]
    pub lod_distance: f32,
    /// Unclassified native bytes at offset 36.
    #[d4(value)]
    pub reserved_36: [u8; 4],
}

/// Native `RotationCurve` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct RotationCurve<'data> {
    /// Native `ptKeysComp` at byte 0.
    #[d4(array)]
    pub keys_comp: ZeroVec<'data, u8>,
}

/// Native `ConstraintLink` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ConstraintLink {
    /// Native `dwHash` at byte 0.
    #[d4(value)]
    pub hash: u32,
}

/// Native `SummonActorEntry` (32 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SummonActorEntry {
    /// Native `dwWeight` at byte 0.
    #[d4(value)]
    pub weight: u32,
    /// Native `snoSummonActor` at byte 4.
    #[d4(value)]
    pub summon_actor: i32,
    /// Native `unk_ce494ef` at byte 8.
    #[d4(value)]
    pub unk_ce494ef: i32,
    /// Native `tSummonRange` at byte 12.
    #[d4(value)]
    pub summon_range: [u32; 2],
    /// Native `dwMaxConcurrentCount` at byte 20.
    #[d4(value)]
    pub max_concurrent_count: u32,
    /// Native `snoCondition` at byte 24.
    #[d4(value)]
    pub condition: i32,
    /// Native `bCanBeMinion` at byte 28.
    #[d4(value)]
    pub can_be_minion: i32,
}

/// Native `AxialCylinder` (20 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AxialCylinder {
    /// Native `wpBase` at byte 0.
    #[d4(value)]
    pub base: [f32; 3],
    /// Native `wdHeight` at byte 12.
    #[d4(value)]
    pub wd_height: f32,
    /// Native `wdRadius` at byte 16.
    #[d4(value)]
    pub wd_radius: f32,
}

/// Native `AttributeSelection` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct AttributeSelection {
    /// Native `eAttribute` at byte 0.
    #[d4(value)]
    pub attribute: i32,
    /// Native `nParam` at byte 4.
    #[d4(value)]
    pub param: i32,
}

/// Native `LightInstanceData` (1048 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct LightInstanceData<'data> {
    /// Native `eType` at byte 0.
    #[d4(value)]
    pub type_: i32,
    /// Native `unk_88f1871` at byte 4.
    #[d4(value)]
    pub unk_88f1871: i32,
    /// Native `unk_152a21b` at byte 8.
    #[d4(value)]
    pub unk_152a21b: f32,
    /// Native `unk_e97490d` at byte 12.
    #[d4(value)]
    pub unk_e97490d: f32,
    /// Native `unk_f27447e` at byte 16.
    pub unk_f27447e: InterpolationPathFloat,
    /// Native `unk_813e8bf` at byte 72.
    #[d4(value)]
    pub unk_813e8bf: f32,
    /// Native `eMobility` at byte 76.
    #[d4(value)]
    pub mobility: i32,
    /// Native `eLightGroup` at byte 80.
    #[d4(value)]
    pub light_group: i32,
    /// Native `dwFlags` at byte 84.
    #[d4(value)]
    pub flags: u32,
    /// Native `dwAffectsFlags` at byte 88.
    #[d4(value)]
    pub affects_flags: u32,
    /// Native `eAffectsGeometry` at byte 92.
    #[d4(value)]
    pub affects_geometry: i32,
    /// Native `unk_56127ed` at byte 96.
    #[d4(value)]
    pub unk_56127ed: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
    /// Native `tMatTexProjected` at byte 104.
    pub mat_tex_projected: MaterialTexture<'data>,
    /// Native `tStateA` at byte 144.
    pub state_a: LightStateParams,
    /// Native `tFlickerAmount` at byte 944.
    #[d4(value)]
    pub flicker_amount: [f32; 2],
    /// Native `tFlickerDelay` at byte 952.
    #[d4(value)]
    pub flicker_delay: [f32; 2],
    /// Native `flDanceRadius` at byte 960.
    #[d4(value)]
    pub dance_radius: f32,
    /// Native `tDanceDelay` at byte 964.
    #[d4(value)]
    pub dance_delay: [f32; 2],
    /// Native `flShadowZBias` at byte 972.
    #[d4(value)]
    pub shadow_z_bias: f32,
    /// Native `flShadowZSlopeScaleBias` at byte 976.
    #[d4(value)]
    pub shadow_z_slope_scale_bias: f32,
    /// Native `unk_6b1c15` at byte 980.
    #[d4(value)]
    pub unk_6b1c15: f32,
    /// Native `flShadowNearPlane` at byte 984.
    #[d4(value)]
    pub shadow_near_plane: f32,
    /// Native `flShadowFadeStartRatio` at byte 988.
    #[d4(value)]
    pub shadow_fade_start_ratio: f32,
    /// Native `flShadowFadeEndRatio` at byte 992.
    #[d4(value)]
    pub shadow_fade_end_ratio: f32,
    /// Native `nSystemDuration` at byte 996.
    #[d4(value)]
    pub system_duration: f32,
    /// Native `flOutroDuration` at byte 1000.
    #[d4(value)]
    pub outro_duration: f32,
    /// Native `tSystemDurationScalar` at byte 1004.
    pub system_duration_scalar: InterpolationScalar,
    /// Native `flSoftShadowRadius` at byte 1036.
    #[d4(value)]
    pub soft_shadow_radius: f32,
    /// Native `unk_c8e249b` at byte 1040.
    #[d4(value)]
    pub unk_c8e249b: u32,
    /// Unclassified native bytes at offset 1044.
    #[d4(value)]
    pub reserved_1044: [u8; 4],
}

/// Native `Octree` (88 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Octree {
    /// Native `nMaxNodes` at byte 0.
    #[d4(value)]
    pub max_nodes: i32,
    /// Native `nMaxLeaves` at byte 4.
    #[d4(value)]
    pub max_leaves: i32,
    /// Native `nMaxPrimitives` at byte 8.
    #[d4(value)]
    pub max_primitives: i32,
    /// Native `nNodeCount` at byte 12.
    #[d4(value)]
    pub node_count: i32,
    /// Native `nLeafCount` at byte 16.
    #[d4(value)]
    pub leaf_count: i32,
    /// Native `nPrimitiveCount` at byte 20.
    #[d4(value)]
    pub primitive_count: i32,
    /// Native `tRootCube` at byte 24.
    pub root_cube: OctreeCube,
    /// Native `ptNodes` at byte 40.
    #[d4(array)]
    pub nodes: Vec<OctreeNode>,
    /// Native `ptLeaves` at byte 56.
    #[d4(array)]
    pub leaves: Vec<OctreeLeaf>,
    /// Native `ptPrimitives` at byte 72.
    #[d4(array)]
    pub primitives: Vec<OctreePrimitive>,
}

/// Native `NPCServiceData` (52 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct NPCServiceData {
    /// Native `eServiceType` at byte 0.
    #[d4(value)]
    pub service_type: i32,
    /// Native `eVendorType` at byte 4.
    #[d4(value)]
    pub vendor_type: i32,
    /// Native `snoVendor` at byte 8.
    #[d4(value)]
    pub vendor: i32,
    /// Native `eCrafterType` at byte 12.
    #[d4(value)]
    pub crafter_type: i32,
    /// Native `eMercenaryClass` at byte 16.
    #[d4(value)]
    pub mercenary_class: i32,
    /// Native `flPriceMarkup` at byte 20.
    #[d4(value)]
    pub price_markup: f32,
    /// Native `eCurrencyType` at byte 24.
    #[d4(value)]
    pub currency_type: i32,
    /// Native `unk_299f311` at byte 28.
    #[d4(value)]
    pub unk_299f311: i32,
    /// Native `snoClassPassiveReputation` at byte 32.
    #[d4(value)]
    pub class_passive_reputation: i32,
    /// Native `eDruidSpiritType` at byte 36.
    #[d4(value)]
    pub druid_spirit_type: i32,
    /// Native `hSpiritIcon` at byte 40.
    #[d4(value)]
    pub h_spirit_icon: u32,
    /// Native `hSpiritBondIcon` at byte 44.
    #[d4(value)]
    pub h_spirit_bond_icon: u32,
    /// Native `snoReputation` at byte 48.
    #[d4(value)]
    pub reputation: i32,
}

/// Native `Sphere` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct Sphere {
    /// Native `wpCenter` at byte 0.
    #[d4(value)]
    pub center: [f32; 3],
    /// Native `wdRadius` at byte 12.
    #[d4(value)]
    pub wd_radius: f32,
}

/// Native `MirrorItemState` (4 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct MirrorItemState {
    /// Native `dwMirrorStateFlags` at byte 0.
    #[d4(value)]
    pub mirror_state_flags: u32,
}

/// Native `TrackConditions` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TrackConditions {
    /// Native `eBehavior` at byte 0.
    #[d4(value)]
    pub behavior: i32,
    /// Native `eToggle` at byte 4.
    #[d4(value)]
    pub toggle: i32,
}

/// Native `GeoChunkVertexBuffer` (80 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct GeoChunkVertexBuffer<'data> {
    /// Native `eVBFormat` at byte 0.
    #[d4(value)]
    pub vb_format: i32,
    /// Native `dwVertStride` at byte 4.
    #[d4(value)]
    pub vert_stride: u32,
    /// Native `ptVertexElems` at byte 8.
    #[d4(array)]
    pub vertex_elems: Vec<VertexElem>,
    /// Native `pnVertexElemPerSemantic` at byte 24.
    #[d4(array)]
    pub vertex_element_per_semantic: ZeroVec<'data, u8>,
    /// Native `vfid` at byte 40.
    #[d4(value)]
    pub vfid: i32,
    /// Unclassified native bytes at offset 44.
    #[d4(value)]
    pub reserved_44: [u8; 4],
    /// Native `ptChunkVertices` at byte 48.
    #[d4(array)]
    pub chunk_vertices: ZeroVec<'data, u8>,
    /// Native `vbid` at byte 64.
    #[d4(value)]
    pub vbid: i32,
    /// Native `baid` at byte 68.
    #[d4(value)]
    pub baid: i32,
    /// Native `unk_4c43adc` at byte 72.
    #[d4(value)]
    pub unk_4c43adc: u32,
    /// Native `fOptional` at byte 76.
    #[d4(value)]
    pub optional: i32,
}

/// Native `SlopeBone` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SlopeBone {
    /// Native `tBoneName` at byte 0.
    pub bone_name: AppBone,
    /// Native `aPitchMin` at byte 4.
    #[d4(value)]
    pub a_pitch_min: f32,
    /// Native `aPitchMax` at byte 8.
    #[d4(value)]
    pub a_pitch_max: f32,
}

/// Native `NPCWeaponHardpointOverride` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct NPCWeaponHardpointOverride {
    /// Native `offset` at byte 0.
    pub offset: HardpointOffset,
    /// Native `snoItemType` at byte 36.
    #[d4(value)]
    pub item_type: i32,
}

/// Native `ConstraintParameters` (176 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct ConstraintParameters {
    /// Native `dwHash` at byte 0.
    #[d4(value)]
    pub hash: u32,
    /// Native `dwHashPadding` at byte 4.
    #[d4(value)]
    pub hash_padding: u32,
    /// Native `dwFlags` at byte 8.
    #[d4(value)]
    pub flags: u32,
    /// Native `eType` at byte 12.
    #[d4(value)]
    pub type_: i32,
    /// Native `nBone1` at byte 16.
    #[d4(value)]
    pub bone1: i32,
    /// Native `nBone2` at byte 20.
    #[d4(value)]
    pub bone2: i32,
    /// Native `localFrame1` at byte 24.
    pub local_frame1: PRTransform,
    /// Native `localFrame2` at byte 52.
    pub local_frame2: PRTransform,
    /// Native `flRevoluteAngleLow` at byte 80.
    #[d4(value)]
    pub revolute_angle_low: f32,
    /// Native `flRevoluteAngleHigh` at byte 84.
    #[d4(value)]
    pub revolute_angle_high: f32,
    /// Native `flRagdollConeAngle` at byte 88.
    #[d4(value)]
    pub ragdoll_cone_angle: f32,
    /// Native `flRagdollTwistLow` at byte 92.
    #[d4(value)]
    pub ragdoll_twist_low: f32,
    /// Native `flRagdollTwistHigh` at byte 96.
    #[d4(value)]
    pub ragdoll_twist_high: f32,
    /// Native `unk_9d701d6` at byte 100.
    #[d4(value)]
    pub unk_9d701d6: i32,
    /// Native `flFriction` at byte 104.
    #[d4(value)]
    pub friction: f32,
    /// Native `flCompliance` at byte 108.
    #[d4(value)]
    pub compliance: f32,
    /// Native `flAngularFrequencyHz` at byte 112.
    #[d4(value)]
    pub angular_frequency_hz: f32,
    /// Native `flAngularDampingRatio` at byte 116.
    #[d4(value)]
    pub angular_damping_ratio: f32,
    /// Native `flLinearFrequencyHz` at byte 120.
    #[d4(value)]
    pub linear_frequency_hz: f32,
    /// Native `flLinearDampingRatio` at byte 124.
    #[d4(value)]
    pub linear_damping_ratio: f32,
    /// Native `unk_66d7e12` at byte 128.
    #[d4(value)]
    pub unk_66d7e12: i32,
    /// Native `flDuration` at byte 132.
    #[d4(value)]
    pub duration: f32,
    /// Native `flMotorDampingRatio` at byte 136.
    #[d4(value)]
    pub motor_damping_ratio: f32,
    /// Native `flStrengthRange` at byte 140.
    #[d4(value)]
    pub strength_range: [f32; 2],
    /// Native `fIgnoreGravity` at byte 148.
    #[d4(value)]
    pub ignore_gravity: i32,
    /// Native `qTwistAxis` at byte 152.
    pub twist_axis: BcQuat,
    /// Native `snoParticleSystemSno` at byte 168.
    #[d4(value)]
    pub particle_system_sno: i32,
    /// Native `unk_5cad7b9` at byte 172.
    #[d4(value)]
    pub unk_5cad7b9: f32,
}

/// Native `EffectGroupKey` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct EffectGroupKey {
    /// Native `flTime` at byte 0.
    #[d4(value)]
    pub time: f32,
    /// Native `eEffectGroupKeyDataType` at byte 4.
    #[d4(value)]
    pub effect_group_key_data_type: i32,
    /// Native `flValue` at byte 8.
    #[d4(value)]
    pub value: f32,
    /// Native `rgbaColor` at byte 12.
    #[d4(value)]
    pub rgba_color: [u8; 4],
    /// Native `vecInTangent` at byte 16.
    #[d4(value)]
    pub vec_in_tangent: [f32; 2],
    /// Native `vecOutTangent` at byte 24.
    #[d4(value)]
    pub vec_out_tangent: [f32; 2],
    /// Native `stringLabel` at byte 32.
    pub string_label: StringLabelHandleEx,
}

/// Native `Type_e97cce1f` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TypeE97cce1f {
    /// Native `unk_9568563` at byte 0.
    #[d4(value)]
    pub unk_9568563: i32,
    /// Native `bOverrideFixedCameraOffset` at byte 4.
    #[d4(value)]
    pub override_fixed_camera_offset: i32,
    /// Native `flCameraOffset` at byte 8.
    #[d4(value)]
    pub camera_offset: f32,
}

/// Native `bcQuat` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct BcQuat {
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

/// Native `GeoChunk` (88 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct GeoChunk<'data> {
    /// Native `aabbBounds` at byte 0.
    pub aabb_bounds: Bounds,
    /// Native `ptLODs` at byte 24.
    #[d4(array)]
    pub lods: Vec<GeoSet<'data>>,
    /// Native `ptChunkVertexBuffers` at byte 40.
    #[d4(array)]
    pub chunk_vertex_buffers: Vec<GeoChunkVertexBuffer<'data>>,
    /// Native `ptChunkIndexBuffers` at byte 56.
    #[d4(array)]
    pub chunk_index_buffers: Vec<GeoChunkIndexBuffer<'data>>,
    /// Native `ptLODDistancePerSubObject` at byte 72.
    #[d4(array)]
    pub lod_distance_per_sub_object: ZeroVec<'data, f32>,
}

/// Native `GlobalComponentClothCapsule` (68 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct GlobalComponentClothCapsule {
    /// Native `tCapsule` at byte 0.
    pub capsule: CollisionCapsule,
    /// Native `dwSlotHash` at byte 64.
    #[d4(value)]
    pub slot_hash: u32,
}

/// Native `SplineEasingWeight` (12 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SplineEasingWeight {
    /// Native `flFirstWeight` at byte 0.
    #[d4(value)]
    pub first_weight: f32,
    /// Native `flMiddleWeight` at byte 4.
    #[d4(value)]
    pub middle_weight: f32,
    /// Native `flLastWeight` at byte 8.
    #[d4(value)]
    pub last_weight: f32,
}

/// Native `NPCComponentInstanceData` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct NPCComponentInstanceData {
    /// Native `tNPCColorSet` at byte 0.
    pub npc_color_set: NPCColorSet,
    /// Native `ptComponents` at byte 24.
    #[d4(inline_array)]
    pub components: [ValidComponent; 4],
}

/// Native `MountHardpointOffsets` (24 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct MountHardpointOffsets {
    /// Native `unk_b71ce07` at byte 0.
    #[d4(value)]
    pub unk_b71ce07: i32,
    /// Unclassified native bytes at offset 4.
    #[d4(value)]
    pub reserved_4: [u8; 4],
    /// Native `arMountOffsetsPerType` at byte 8.
    #[d4(array)]
    pub mount_offsets_per_type: Vec<HardpointOffset>,
}

/// Native `BindingPath` (40 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct BindingPath<'data> {
    /// Native `sDataStore` at byte 0.
    #[d4(bytes)]
    pub s_data_store: Cow<'data, [u8]>,
    /// Native `sDataPath` at byte 16.
    #[d4(bytes)]
    pub s_data_path: Cow<'data, [u8]>,
    /// Native `dwFlags` at byte 32.
    #[d4(value)]
    pub flags: u32,
    /// Unclassified native bytes at offset 36.
    #[d4(value)]
    pub reserved_36: [u8; 4],
}

/// Native `SurfaceGridPrimitive` (8 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct SurfaceGridPrimitive {
    /// Native `dwHeight` at byte 0.
    #[d4(value)]
    pub height: u8,
    /// Unclassified native bytes at offset 1.
    #[d4(value)]
    pub reserved_1: [u8; 3],
    /// Native `snoSurface` at byte 4.
    #[d4(value)]
    pub surface: u32,
}
