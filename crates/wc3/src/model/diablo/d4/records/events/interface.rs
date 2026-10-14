//! Native interface event records.
use super::*;
/// Native `TriggerEventStoreUIIntro` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventStoreUIIntro {
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

/// Native `TriggerEventHidePoIUI` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventHidePoIUI {
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

/// Native `TriggerEventCameraChangeNearFarPlanes` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCameraChangeNearFarPlanes {
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
    /// Native `flNearPlane` at byte 144.
    #[d4(value)]
    pub near_plane: f32,
    /// Native `flFarPlane` at byte 148.
    #[d4(value)]
    pub far_plane: f32,
    /// Native `eBlendEasing` at byte 152.
    #[d4(value)]
    pub blend_easing: i32,
    /// Unclassified native bytes at offset 156.
    #[d4(value)]
    pub reserved_156: [u8; 4],
}

/// Native `TriggerEventCutsceneUIKeyFrame` (176 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCutsceneUIKeyFrame<'data> {
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
    /// Native `unk_880ba72` at byte 96.
    pub unk_880ba72: BindingPath<'data>,
    /// Native `snoUI` at byte 136.
    #[d4(value)]
    pub ui: i32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
    /// Native `arKeys` at byte 144.
    #[d4(polymorphic = 2863656093)]
    pub keys: Vec<Polymorphic<'data>>,
    /// Native `flLastKeyTime` at byte 168.
    #[d4(value)]
    pub last_key_time: f32,
    /// Unclassified native bytes at offset 172.
    #[d4(value)]
    pub reserved_172: [u8; 4],
}

/// Native `TriggerEventHideWarpUI` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventHideWarpUI {
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
    /// Native `snoUI` at byte 96.
    #[d4(value)]
    pub ui: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventCameraMoveToCameraMarker` (208 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCameraMoveToCameraMarker {
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
    /// Native `szCameraMarker` at byte 144.
    #[d4(value)]
    pub camera_marker: u32,
    /// Native `unk_8f65c3f` at byte 148.
    #[d4(value)]
    pub unk_8f65c3f: i32,
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
    /// Native `vecTargetOffset` at byte 176.
    #[d4(value)]
    pub vec_target_offset: [f32; 3],
    /// Native `eMicrophoneMode` at byte 188.
    #[d4(value)]
    pub microphone_mode: i32,
    /// Native `vecMicrophoneOffset` at byte 192.
    #[d4(value)]
    pub vec_microphone_offset: [f32; 3],
    /// Native `eBlendEasing` at byte 204.
    #[d4(value)]
    pub blend_easing: i32,
}

/// Native `TriggerEventCameraTransitionToObserver` (200 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCameraTransitionToObserver {
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
    /// Native `snoObserver` at byte 144.
    #[d4(value)]
    pub observer: i32,
    /// Native `unk_74b7ed1` at byte 148.
    #[d4(value)]
    pub unk_74b7ed1: i32,
    /// Native `eTargetType` at byte 152.
    #[d4(value)]
    pub target_type: i32,
    /// Native `szTargetLocatorName` at byte 156.
    #[d4(value)]
    pub target_locator_name: u32,
    /// Native `tHardpointLinkTarget` at byte 160.
    pub hardpoint_link_target: HardpointLink,
    /// Native `dwTargetID` at byte 168.
    #[d4(value)]
    pub target_id: u32,
    /// Unclassified native bytes at offset 172.
    #[d4(value)]
    pub reserved_172: [u8; 4],
    /// Native `arAdditionalTargets` at byte 176.
    #[d4(array)]
    pub additional_targets: Vec<TriggerEventObserverAdditionalTarget>,
    /// Native `eBlendEasing` at byte 192.
    #[d4(value)]
    pub blend_easing: i32,
    /// Native `flRadiusConstraint` at byte 196.
    #[d4(value)]
    pub radius_constraint: f32,
}

/// Native `TriggerEventHideCaptionUI` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventHideCaptionUI {
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

/// Native `TriggerEventShowCaptionUIOptionalFieldMapDefaults` (16 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventShowCaptionUIOptionalFieldMapDefaults<'data> {
    /// Native `szIDStoryboard` at byte 0.
    #[d4(bytes)]
    pub id_storyboard: Cow<'data, [u8]>,
}

/// Native `TriggerEventCameraStayOnSpline` (296 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCameraStayOnSpline {
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
    /// Native `eProximityType` at byte 184.
    #[d4(value)]
    pub proximity_type: i32,
    /// Native `szProximityLocatorName` at byte 188.
    #[d4(value)]
    pub proximity_locator_name: u32,
    /// Native `tHardpointLinkProximityObject` at byte 192.
    pub hardpoint_link_proximity_object: HardpointLink,
    /// Native `dwProximityObjectID` at byte 200.
    #[d4(value)]
    pub proximity_object_id: u32,
    /// Native `vecProximityOffset` at byte 204.
    #[d4(value)]
    pub vec_proximity_offset: [f32; 3],
    /// Native `unk_8ff6495` at byte 216.
    #[d4(value)]
    pub unk_8ff6495: i32,
    /// Native `fUseNodeOrientation` at byte 220.
    #[d4(value)]
    pub use_node_orientation: i32,
    /// Native `flNodeDistance` at byte 224.
    #[d4(value)]
    pub node_distance: f32,
    /// Native `flMaxMovementSpeed` at byte 228.
    #[d4(value)]
    pub max_movement_speed: f32,
    /// Native `unk_802b7b` at byte 232.
    #[d4(value)]
    pub unk_802b7b: i32,
    /// Native `snoObserverForOrientation` at byte 236.
    #[d4(value)]
    pub observer_for_orientation: i32,
    /// Native `eOrientationMode` at byte 240.
    #[d4(value)]
    pub orientation_mode: i32,
    /// Native `eLookAtTargetType` at byte 244.
    #[d4(value)]
    pub look_at_target_type: i32,
    /// Native `szLookAtTargetLocatorName` at byte 248.
    #[d4(value)]
    pub look_at_target_locator_name: u32,
    /// Native `tHardpointLinkLookAtTarget` at byte 252.
    pub hardpoint_link_look_at_target: HardpointLink,
    /// Native `dwLookAtTargetID` at byte 260.
    #[d4(value)]
    pub look_at_target_id: u32,
    /// Native `vecTargetOffset` at byte 264.
    #[d4(value)]
    pub vec_target_offset: [f32; 3],
    /// Native `eMicrophoneMode` at byte 276.
    #[d4(value)]
    pub microphone_mode: i32,
    /// Native `vecMicrophoneOffset` at byte 280.
    #[d4(value)]
    pub vec_microphone_offset: [f32; 3],
    /// Native `eBlendEasing` at byte 292.
    #[d4(value)]
    pub blend_easing: i32,
}

/// Native `TriggerEventCameraChangeFieldOfView` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCameraChangeFieldOfView {
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
    /// Native `unk_7903775` at byte 144.
    #[d4(value)]
    pub unk_7903775: i32,
    /// Native `aBeginFOV` at byte 148.
    #[d4(value)]
    pub a_begin_fov: f32,
    /// Native `aEndFOV` at byte 152.
    #[d4(value)]
    pub a_end_fov: f32,
    /// Native `eBlendEasing` at byte 156.
    #[d4(value)]
    pub blend_easing: i32,
}

/// Native `TriggerEventShowWarpUI` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventShowWarpUI {
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
    /// Native `snoUI` at byte 96.
    #[d4(value)]
    pub ui: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventShowCaptionUI` (120 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventShowCaptionUI {
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
    /// Native `snoUI` at byte 96.
    #[d4(value)]
    pub ui: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
    /// Native `hOptionalTriggerEventShowCaptionUIFields` at byte 104.
    pub h_optional_trigger_event_show_caption_ui_fields: EmptyTagMap,
}

/// Native `TriggerEventCameraChangeTarget` (200 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCameraChangeTarget {
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
    /// Native `eTargetType` at byte 144.
    #[d4(value)]
    pub target_type: i32,
    /// Native `szTargetLocatorName` at byte 148.
    #[d4(value)]
    pub target_locator_name: u32,
    /// Native `tHardpointLinkTarget` at byte 152.
    pub hardpoint_link_target: HardpointLink,
    /// Native `dwTargetID` at byte 160.
    #[d4(value)]
    pub target_id: u32,
    /// Native `vecTargetOffset` at byte 164.
    #[d4(value)]
    pub vec_target_offset: [f32; 3],
    /// Native `eMicrophoneMode` at byte 176.
    #[d4(value)]
    pub microphone_mode: i32,
    /// Native `vecMicrophoneOffset` at byte 180.
    #[d4(value)]
    pub vec_microphone_offset: [f32; 3],
    /// Native `eBlendEasing` at byte 192.
    #[d4(value)]
    pub blend_easing: i32,
    /// Unclassified native bytes at offset 196.
    #[d4(value)]
    pub reserved_196: [u8; 4],
}

/// Native `TriggerEventRestoreCamera` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventRestoreCamera {
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

/// Native `TriggerEventShowPoIUI` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventShowPoIUI {
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
    /// Native `hPoILabel` at byte 96.
    pub h_po_i_label: StringLabelHandleEx,
    /// Native `snoPOICamera` at byte 104.
    #[d4(value)]
    pub poi_camera: i32,
    /// Unclassified native bytes at offset 108.
    #[d4(value)]
    pub reserved_108: [u8; 4],
}

/// Native `TriggerEventCameraChangeDepthOfField` (192 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCameraChangeDepthOfField {
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
    /// Native `unk_7902ee3` at byte 144.
    #[d4(value)]
    pub unk_7902ee3: i32,
    /// Native `flBeginFocalDistance` at byte 148.
    #[d4(value)]
    pub begin_focal_distance: f32,
    /// Native `flBeginFStop` at byte 152.
    #[d4(value)]
    pub begin_f_stop: f32,
    /// Native `flBeginNearTransition` at byte 156.
    #[d4(value)]
    pub begin_near_transition: f32,
    /// Native `flBeginFocalRegion` at byte 160.
    #[d4(value)]
    pub begin_focal_region: f32,
    /// Native `flBeginFarTransition` at byte 164.
    #[d4(value)]
    pub begin_far_transition: f32,
    /// Native `flEndFocalDistance` at byte 168.
    #[d4(value)]
    pub end_focal_distance: f32,
    /// Native `flEndFStop` at byte 172.
    #[d4(value)]
    pub end_f_stop: f32,
    /// Native `flEndNearTransition` at byte 176.
    #[d4(value)]
    pub end_near_transition: f32,
    /// Native `flEndFocalRegion` at byte 180.
    #[d4(value)]
    pub end_focal_region: f32,
    /// Native `flEndFarTransition` at byte 184.
    #[d4(value)]
    pub end_far_transition: f32,
    /// Native `eBlendEasing` at byte 188.
    #[d4(value)]
    pub blend_easing: i32,
}

/// Native `TriggerEventAttachCamera` (192 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventAttachCamera {
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
    /// Native `eObjectType` at byte 144.
    #[d4(value)]
    pub object_type: i32,
    /// Native `tHardpointLinkTarget` at byte 148.
    pub hardpoint_link_target: HardpointLink,
    /// Native `dwObjectID` at byte 156.
    #[d4(value)]
    pub object_id: u32,
    /// Native `tHardpointLink` at byte 160.
    pub hardpoint_link: HardpointLink,
    /// Native `fUseAnimationFieldOfView` at byte 168.
    #[d4(value)]
    pub use_animation_field_of_view: i32,
    /// Native `fUseAnimationDepthOfField` at byte 172.
    #[d4(value)]
    pub use_animation_depth_of_field: i32,
    /// Native `eBlendEasing` at byte 176.
    #[d4(value)]
    pub blend_easing: i32,
    /// Native `unk_10734fd` at byte 180.
    pub unk_10734fd: TypeE97cce1f,
}

/// Native `TriggerEventCameraKeyFrame` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCameraKeyFrame<'data> {
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
    /// Native `eMicrophoneMode` at byte 96.
    #[d4(value)]
    pub microphone_mode: i32,
    /// Native `vecMicrophoneOffset` at byte 100.
    #[d4(value)]
    pub vec_microphone_offset: [f32; 3],
    /// Native `arKeys` at byte 112.
    #[d4(polymorphic = 2863656093)]
    pub keys: Vec<Polymorphic<'data>>,
    /// Native `flLastKeyTime` at byte 136.
    #[d4(value)]
    pub last_key_time: f32,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
}

/// Native `TriggerEventCameraFollowSpline` (312 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventCameraFollowSpline {
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
    /// Native `flBlendTime` at byte 144.
    #[d4(value)]
    pub blend_time: f32,
    /// Native `szSplineName` at byte 148.
    #[d4(value)]
    pub spline_name: u32,
    /// Native `eSplineRangeMode` at byte 152.
    #[d4(value)]
    pub spline_range_mode: i32,
    /// Native `szSplineFirstNode` at byte 156.
    #[d4(value)]
    pub spline_first_node: u32,
    /// Native `szSplineLastNode` at byte 160.
    #[d4(value)]
    pub spline_last_node: u32,
    /// Native `flFirstPercent` at byte 164.
    #[d4(value)]
    pub first_percent: f32,
    /// Native `flLastPercent` at byte 168.
    #[d4(value)]
    pub last_percent: f32,
    /// Native `flFirstDistance` at byte 172.
    #[d4(value)]
    pub first_distance: f32,
    /// Native `flLastDistance` at byte 176.
    #[d4(value)]
    pub last_distance: f32,
    /// Native `nSplineFirstNodeIndex` at byte 180.
    #[d4(value)]
    pub spline_first_node_index: i32,
    /// Native `nSplineLastNodeIndex` at byte 184.
    #[d4(value)]
    pub spline_last_node_index: i32,
    /// Native `eEasing` at byte 188.
    #[d4(value)]
    pub easing: i32,
    /// Native `tEasingWeight` at byte 192.
    pub easing_weight: SplineEasingWeight,
    /// Native `unk_802b7b` at byte 204.
    #[d4(value)]
    pub unk_802b7b: i32,
    /// Native `snoObserverForOrientation` at byte 208.
    #[d4(value)]
    pub observer_for_orientation: i32,
    /// Native `eOrientationMode` at byte 212.
    #[d4(value)]
    pub orientation_mode: i32,
    /// Native `transformOffset` at byte 216.
    pub transform_offset: PRTransform,
    /// Native `eLookAtTargetType` at byte 244.
    #[d4(value)]
    pub look_at_target_type: i32,
    /// Native `szLookAtTargetLocatorName` at byte 248.
    #[d4(value)]
    pub look_at_target_locator_name: u32,
    /// Native `tHardpointLinkLookAtTarget` at byte 252.
    pub hardpoint_link_look_at_target: HardpointLink,
    /// Native `dwLookAtTargetID` at byte 260.
    #[d4(value)]
    pub look_at_target_id: u32,
    /// Native `vecTargetOffset` at byte 264.
    #[d4(value)]
    pub vec_target_offset: [f32; 3],
    /// Native `eMicrophoneMode` at byte 276.
    #[d4(value)]
    pub microphone_mode: i32,
    /// Native `vecMicrophoneOffset` at byte 280.
    #[d4(value)]
    pub vec_microphone_offset: [f32; 3],
    /// Native `eEasingType` at byte 292.
    #[d4(value)]
    pub easing_type: i32,
    /// Native `eBlendEasing` at byte 296.
    #[d4(value)]
    pub blend_easing: i32,
    /// Native `eFollowSplineInterpolation` at byte 300.
    #[d4(value)]
    pub follow_spline_interpolation: i32,
    /// Native `flInTangentWeight` at byte 304.
    #[d4(value)]
    pub in_tangent_weight: f32,
    /// Native `flOutTangentWeight` at byte 308.
    #[d4(value)]
    pub out_tangent_weight: f32,
}

/// Native `TriggerEventHideWarpDestinationUI` (96 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventHideWarpDestinationUI {
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

/// Native `TriggerEventShowWarpDestinationUI` (112 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventShowWarpDestinationUI {
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
}
