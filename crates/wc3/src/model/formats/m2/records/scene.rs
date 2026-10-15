//! Attachments, event markers, and lights.
use super::LightKind;
use super::{Track, TrackBase};
use crate::model::{common::Ownership, formats::m2};
use m2::ModelLayout;

/// Named-role anchor for attached models, effects, and UI elements.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct Attachment<'data, V: ModelLayout> {
    /// Attachment role ID, such as a hand, weapon socket, or nameplate anchor.
    pub id: u32,
    /// Index of the bone to which this attachment is anchored.
    pub bone_id: u16,
    /// Unresolved attachment word following the bone index.
    pub unknown: u16,
    /// Attachment position transformed by its anchor bone.
    pub position: [f32; 3],
    /// Animated attachment enable flag, with zero disabling the attachment.
    pub animate: Track<'data, u8, V>,
}
/// Timestamp-triggered action associated with a bone and location.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct Event<'data, V: ModelLayout> {
    /// Four-byte event code, commonly `$` followed by three letters, such as `$DTH`.
    pub identifier: [u8; 4],
    /// Event-specific payload, such as a sound-entry or camera-shake ID.
    pub data: u32,
    /// Index of the bone anchoring the event location.
    pub bone_id: u32,
    /// Event position transformed by its anchor bone.
    pub position: [f32; 3],
    /// Timestamp-only track; each timestamp triggers the event without a corresponding value.
    pub enabled: TrackBase<'data, V>,
}
/// Animated directional or point light attached to the model.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct Light<'data, V: ModelLayout> {
    /// Light type: zero is directional and one is a point light.
    pub kind: LightKind,
    /// Anchor bone index, or `-1` for a light without a bone attachment.
    pub bone_id: i16,
    /// Light position transformed by its anchor bone, or model coordinates when unattached.
    pub position: [f32; 3],
    /// Animated RGB color of the ambient-light contribution.
    pub ambient_color: Track<'data, [f32; 3], V>,
    /// Animated multiplier for the ambient-light color.
    pub ambient_intensity: Track<'data, f32, V>,
    /// Animated RGB color of the diffuse-light contribution.
    pub diffuse_color: Track<'data, [f32; 3], V>,
    /// Animated multiplier for the diffuse-light color.
    pub diffuse_intensity: Track<'data, f32, V>,
    /// Animated distance where light falloff begins, in model distance units.
    pub attenuation_start: Track<'data, f32, V>,
    /// Animated distance where light falloff ends, in model distance units.
    pub attenuation_end: Track<'data, f32, V>,
    /// Animated light enable flag, with zero disabling the light.
    pub visibility: Track<'data, u8, V>,
}
