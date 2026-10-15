//! Sequence metadata, timing, and skeletal animation records.
use super::{ArrayVisitor, Extent, Track, Validate, VisitArrays};
use super::{BoneFlags, SequenceFlags};
use crate::model::{common::Ownership, formats::m2};
use m2::ModelLayout;
use m2::ReadError;

/// Absolute start/end timestamps on a legacy shared sequence clock.
#[derive(
    Clone, Copy, Debug, PartialEq, Default, Ownership, m2::Read, m2::Write, m2::RecordLayout,
)]
pub struct LegacyTiming {
    /// Inclusive sequence start in milliseconds.
    pub start: u32,
    /// Sequence end in milliseconds.
    pub end: u32,
}
impl Validate for LegacyTiming {
    fn validate(&self) -> Result<(), ReadError> {
        if self.end < self.start {
            Err(ReadError::invalid(0, "legacy sequence window"))
        } else {
            Ok(())
        }
    }
}
impl<'data> VisitArrays<'data> for LegacyTiming {
    fn visit_arrays<V: ArrayVisitor<'data>>(&mut self, _: &mut V) -> Result<(), V::Error> {
        Ok(())
    }
}
/// Separate incoming and outgoing sequence transition durations.
#[derive(
    Clone,
    Copy,
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
pub struct SplitBlendTime {
    /// Duration of the transition into this sequence, in milliseconds.
    pub incoming: u16,
    /// Duration of the transition out of this sequence, in milliseconds.
    pub outgoing: u16,
}
/// Animation identity, variation selection, timing, and sequence bounds.
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
pub struct Sequence<V: ModelLayout> {
    /// AnimationData animation ID, such as stand, walk, or attack.
    pub id: u16,
    /// Variation number among sequences with the same animation ID.
    pub variation_index: u16,
    /// Legacy absolute millisecond window or modern millisecond duration, selected by the layout.
    pub timing: V::Timing,
    /// Movement speed associated with the animation, in model distance units per second.
    pub move_speed: f32,
    /// Sequence behavior and storage flags; `0x40` marks an alias to `alias_next`.
    pub flags: SequenceFlags,
    /// Relative selection weight among variations of the same animation ID.
    pub frequency: i16,
    /// Alignment word with no defined animation behavior.
    pub padding: u16,
    /// Minimum repetition count; zero requests no repeat.
    pub replay_min: u32,
    /// Maximum repetition count.
    pub replay_max: u32,
    /// Transition duration in milliseconds, stored as one value or separate incoming/outgoing values.
    pub blend_time: V::BlendTime,
    /// Bounds encompassing the model while this sequence plays.
    pub bounding: Extent,
    /// Index of the next variation in the sequence array, or `-1` to end the chain.
    pub variation_next: i16,
    /// Index of the sequence supplying animation data when the alias flag is set.
    pub alias_next: u16,
}
/// Skeletal hierarchy entry with pivot-relative animation tracks.
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
pub struct Bone<'data, V: ModelLayout> {
    /// Semantic key-bone ID, or `-1` for a bone with no key-bone role.
    pub key_bone_id: i32,
    /// Bone transform flags controlling billboarding and inherited transforms.
    pub flags: BoneFlags,
    /// Parent index in the bone array, or `-1` for a root bone.
    pub parent_bone_id: i16,
    /// Mesh-part identifier associated with the bone.
    pub submesh_id: u16,
    /// Bone-name CRC in layouts that store it; `()` in layouts without this field.
    pub name_crc: V::NameCrc,
    /// Animated translation relative to the bone's pivot.
    pub translation: Track<'data, [f32; 3], V>,
    /// Animated quaternion rotation; the layout selects full or compressed components.
    pub rotation: Track<'data, V::RotationValue, V>,
    /// Animated scale along the three bone axes.
    pub scale: Track<'data, [f32; 3], V>,
    /// Rotation and scaling pivot in model coordinates.
    pub pivot: [f32; 3],
}
impl<'data, V: ModelLayout> Bone<'data, V> {
    /// Creates an empty root bone in the selected native layout.
    pub fn new() -> Self {
        Self {
            key_bone_id: -1,
            parent_bone_id: -1,
            ..Self::default()
        }
    }
}
