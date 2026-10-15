//! Native audio event records.
use super::*;
/// Native `TriggerEventFoley` (152 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventFoley {
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
    /// Native `szFoleyTypeTag` at byte 104.
    #[d4(value)]
    pub foley_type_tag: u32,
    /// Native `snoFoleySoundTable` at byte 108.
    #[d4(value)]
    pub foley_sound_table: i32,
    /// Native `unk_e6137fe` at byte 112.
    #[d4(value)]
    pub unk_e6137fe: u32,
    /// Native `fMustAttach` at byte 116.
    #[d4(value)]
    pub must_attach: i32,
    /// Native `dwID` at byte 120.
    #[d4(value)]
    pub id: u32,
    /// Native `tCleanup` at byte 124.
    pub cleanup: TriggerCleanup,
    /// Unclassified native bytes at offset 148.
    #[d4(value)]
    pub reserved_148: [u8; 4],
}

/// Native `TriggerEventMusic` (104 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventMusic {
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
    /// Native `snoMusic` at byte 96.
    #[d4(value)]
    pub music: i32,
    /// Unclassified native bytes at offset 100.
    #[d4(value)]
    pub reserved_100: [u8; 4],
}

/// Native `TriggerEventVoice` (144 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventVoice {
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
    /// Native `szVoiceTypeTag` at byte 104.
    #[d4(value)]
    pub voice_type_tag: u32,
    /// Native `snoVoiceSoundTable` at byte 108.
    #[d4(value)]
    pub voice_sound_table: i32,
    /// Native `dwID` at byte 112.
    #[d4(value)]
    pub id: u32,
    /// Native `tCleanup` at byte 116.
    pub cleanup: TriggerCleanup,
    /// Unclassified native bytes at offset 140.
    #[d4(value)]
    pub reserved_140: [u8; 4],
}

/// Native `TriggerEventSetIGCMusic` (152 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventSetIGCMusic {
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
    /// Native `snoMusic` at byte 144.
    #[d4(value)]
    pub music: i32,
    /// Native `unk_e1829be` at byte 148.
    #[d4(value)]
    pub unk_e1829be: f32,
}

/// Native `TriggerEventPlayConversationLine` (160 fixed bytes).
#[derive(Clone, Debug, PartialEq, Default, Ownership, d4::Read, d4::Write, d4::RecordLayout)]
pub struct TriggerEventPlayConversationLine {
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
    /// Native `dwConversationNodeID` at byte 144.
    #[d4(value)]
    pub conversation_node_id: u32,
    /// Native `dwActorID` at byte 148.
    #[d4(value)]
    pub actor_id: u32,
    /// Native `fIsSkipPoint` at byte 152.
    #[d4(value)]
    pub is_skip_point: i32,
    /// Native `unk_e8a81a5` at byte 156.
    #[d4(value)]
    pub unk_e8a81a5: u32,
}
