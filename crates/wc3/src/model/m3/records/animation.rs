//! Animation sequences connect controller tables to typed keyframe blocks and bones.
//!
//! A property animation ID is looked up in controller ID arrays. The corresponding
//! packed reference selects one of thirteen keyframe-block families and an element
//! within that family. Sequence and bone dialects preserve their own field layouts.
use super::dialect::*;
use super::{targets, AnimationValue, Extent};
use crate::model::m3::{self, Record, Ref};
use core::fmt::Debug;
/// Native field selections for independent `Sequence` section revisions.
pub trait SequenceDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `deprecated_unknown`; `()` omits the field.
    type DeprecatedUnknown: NativeField;
}

impl SequenceDialect for V0 {
    const VERSION: u32 = 0;
    type DeprecatedUnknown = u32;
}

impl SequenceDialect for V1 {
    const VERSION: u32 = 1;
    type DeprecatedUnknown = u32;
}

impl SequenceDialect for V2 {
    const VERSION: u32 = 2;
    type DeprecatedUnknown = ();
}

/// Sequence interval, selection metadata, and bounds under one native section dialect.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Sequence<D: SequenceDialect> {
    /// Signed identifier assigned to this sequence.
    pub id: i32,
    /// Signed sequence-table identity recorded by the file.
    pub index: i32,
    /// CHAR link naming this sequence.
    pub name: Ref<targets::Character>,
    /// Inclusive beginning of the sequence frame interval.
    pub start_frame: u32,
    /// Inclusive ending of the sequence frame interval.
    pub end_frame: u32,
    /// Multiplier describing movement speed for the sequence.
    pub move_speed: f32,
    /// Playback behavior bits, including looping and global-sequence behavior.
    pub flags: u32,
    /// Relative weight used when choosing this sequence.
    pub frequency: u32,
    /// Beginning frame of the sequence replay subinterval.
    pub replay_start: u32,
    /// Ending frame of the sequence replay subinterval.
    pub replay_end: u32,
    /// Duration of sequence blending, expressed in milliseconds.
    pub blend_time: u32,
    /// Extra legacy word between blend metadata and bounds; the selected dialect can omit it.
    pub deprecated_unknown: D::DeprecatedUnknown,
    /// Box and radius covering geometry during this sequence.
    pub bounds: Extent,
    /// U8__ array identifying animation sets associated with the sequence.
    pub animation_sets: Ref<targets::Byte>,
}

impl<D: SequenceDialect> Record for Sequence<D> {
    const TAG: [u8; 4] = *b"SEQS";
    const VERSION: u32 = D::VERSION;
}

/// Native field selections for independent `SubTrackContainer` section revisions.
pub trait SubTrackContainerDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `name`; `()` omits the field.
    type Name: NativeField;
}

impl SubTrackContainerDialect for V0 {
    const VERSION: u32 = 0;
    type Name = Ref<targets::Character>;
}

impl SubTrackContainerDialect for V1 {
    const VERSION: u32 = 1;
    type Name = Ref<targets::Character>;
}

impl SubTrackContainerDialect for V2 {
    const VERSION: u32 = 2;
    type Name = Ref<targets::Character>;
}

impl SubTrackContainerDialect for V3 {
    const VERSION: u32 = 3;
    type Name = Ref<targets::Character>;
}

impl SubTrackContainerDialect for V4 {
    const VERSION: u32 = 4;
    type Name = Ref<targets::Character>;
}

/// Controller lookup tables and the thirteen typed keyframe-block arrays.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct SubTrackContainer<D: SubTrackContainerDialect> {
    /// CHAR link naming this controller.
    pub name: D::Name,
    /// Nonzero permits this controller to run concurrently with another controller.
    pub runs_concurrent: u16,
    /// Priority assigned to this animation controller.
    pub anim_priority: u16,
    /// STS_ record selected as this controller state.
    pub animation_state_index: u16,
    /// Second stored state index; its distinction from `animation_state_index` is unresolved.
    pub animation_state_index_copy: u16,
    /// U32_ property identifiers whose entries correspond to `anim_refs`.
    pub anim_ids: Ref<targets::U32>,
    /// U32_ packed keyframe selectors: upper 16 bits choose the block family, lower 16 bits choose its element.
    pub anim_refs: Ref<targets::U32>,
    /// Uninterpreted word between controller lookup arrays and keyframe-family links.
    pub unknown: u32,
    /// Keyframe family slot 0, pointing to blocks of event keys.
    pub sdev: Ref<targets::AnimationBlockEvent>,
    /// Keyframe family slot 1, pointing to blocks of two-component vector keys.
    pub sd2v: Ref<targets::AnimationBlockVector2>,
    /// Keyframe family slot 2, pointing to blocks of three-component vector keys.
    pub sd3v: Ref<targets::AnimationBlockVector3>,
    /// Keyframe family slot 3, pointing to blocks of quaternion keys.
    pub sd4q: Ref<targets::AnimationBlockQuaternion>,
    /// Keyframe family slot 4, pointing to blocks of color keys.
    pub sdcc: Ref<targets::AnimationBlockColor>,
    /// Keyframe family slot 5, pointing to blocks of floating-point scalar keys.
    pub sdr3: Ref<targets::AnimationBlockReal>,
    /// Keyframe family slot 6, pointing to blocks of unsigned byte keys.
    pub sdu8: Ref<targets::AnimationBlockByte>,
    /// Keyframe family slot 7, pointing to blocks of signed 16-bit keys.
    pub sds6: Ref<targets::AnimationBlockI16>,
    /// Keyframe family slot 8, pointing to blocks of unsigned 16-bit keys.
    pub sdu6: Ref<targets::AnimationBlockU16>,
    /// Keyframe family slot 9, pointing to blocks of signed 32-bit keys.
    pub sds3: Ref<targets::AnimationBlockI32>,
    /// Keyframe family slot 10, pointing to blocks of unsigned 32-bit keys.
    pub sdu3: Ref<targets::AnimationBlockU32>,
    /// Keyframe family slot 11, pointing to blocks of bitmask keys.
    pub sdfg: Ref<targets::AnimationBlockFlag>,
    /// Keyframe family slot 12, pointing to blocks of bounding-extent keys.
    pub sdmb: Ref<targets::AnimationBlockExtent>,
}

impl<D: SubTrackContainerDialect> Record for SubTrackContainer<D> {
    const TAG: [u8; 4] = *b"STC_";
    const VERSION: u32 = D::VERSION;
}

/// A named collection of animation controller indices.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct AnimationGroup {
    /// CHAR link naming this group.
    pub name: Ref<targets::Character>,
    /// U32_ array of STC_ indices belonging to this animation group.
    pub subtrack_indices: Ref<targets::U32>,
}

impl Record for AnimationGroup {
    const TAG: [u8; 4] = *b"STG_";
    const VERSION: u32 = 0;
}

/// An animation-ID list accompanied by an opaque state block.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct AnimationState {
    /// U32_ property identifiers belonging to this animation state.
    pub anim_ids: Ref<targets::U32>,
    /// Sixteen opaque bytes following the animation-ID link.
    pub unknown: [u8; 16],
}

impl Record for AnimationState {
    const TAG: [u8; 4] = *b"STS_";
    const VERSION: u32 = 0;
}

/// Primary and fallback sequences for a named bone animation set.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct BoneAnimationSet {
    /// Behavior bits for this bone animation set.
    pub flags: u32,
    /// SEQS index selected as the primary sequence for this bone animation set.
    pub animation_sequence_index: u16,
    /// SEQS index used as the fallback for this bone animation set.
    pub fallback_sequence_index: u16,
    /// CHAR link naming this bone animation set.
    pub name: Ref<targets::Character>,
    /// U16_ array of split-item identifiers for this bone animation set.
    pub split_items: Ref<targets::U16>,
}

impl Record for BoneAnimationSet {
    const TAG: [u8; 4] = *b"BSET";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `Bone` section revisions.
pub trait BoneDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `unknown`; `()` omits the field.
    type Unknown: NativeField;
}

impl BoneDialect for V0 {
    const VERSION: u32 = 0;
    type Unknown = u32;
}

impl BoneDialect for V1 {
    const VERSION: u32 = 1;
    type Unknown = u32;
}

/// Hierarchy metadata and animated local properties under one native section dialect.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct Bone<D: BoneDialect> {
    /// Legacy metadata word before the bone name; its purpose is unresolved.
    pub unknown: D::Unknown,
    /// CHAR link naming this bone.
    pub name: Ref<targets::Character>,
    /// Bone behavior bits recorded with its hierarchy metadata.
    pub flags: u32,
    /// BONE index identifying the parent; `0xffff` marks a hierarchy root.
    pub parent_index: u16,
    /// Two uninterpreted bytes separating hierarchy metadata from animated properties.
    pub padding: u16,
    /// Animated local translation with its identifier and fallback values.
    pub position: AnimationValue<[f32; 3]>,
    /// Animated quaternion with its identifier and fallback values.
    pub rotation: AnimationValue<[f32; 4]>,
    /// Animated local scale with its identifier and fallback values.
    pub scale: AnimationValue<[f32; 3]>,
    /// Animated visibility value with its identifier and fallback values.
    pub visibility: AnimationValue<u32>,
}

impl<D: BoneDialect> Record for Bone<D> {
    const TAG: [u8; 4] = *b"BONE";
    const VERSION: u32 = D::VERSION;
}

/// One inverse bind transform in the initial-reference section.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct InitialReference {
    /// Inverse bind matrix encoded as four rows of four floating-point components.
    pub matrix: [[f32; 4]; 4],
}

impl Record for InitialReference {
    const TAG: [u8; 4] = *b"IREF";
    const VERSION: u32 = 0;
}

/// Native field selections for independent `AttachmentPoint` section revisions.
pub trait AttachmentPointDialect: Revision + Copy + Clone + Debug + Default + PartialEq {
    /// Native section revision.
    const VERSION: u32;
    /// Field representation for `unknown`; `()` omits the field.
    type Unknown: NativeField;
}

impl AttachmentPointDialect for V0 {
    const VERSION: u32 = 0;
    type Unknown = u32;
}

impl AttachmentPointDialect for V1 {
    const VERSION: u32 = 1;
    type Unknown = u32;
}

/// A named attachment linked to a bone under its own section dialect.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct AttachmentPoint<D: AttachmentPointDialect> {
    /// Uninterpreted attachment metadata before its name.
    pub unknown: D::Unknown,
    /// CHAR link naming this attachment.
    pub name: Ref<targets::Character>,
    /// BONE index identifying the bone that owns this attachment point.
    pub bone_index: u32,
}

impl<D: AttachmentPointDialect> Record for AttachmentPoint<D> {
    const TAG: [u8; 4] = *b"ATT_";
    const VERSION: u32 = D::VERSION;
}

/// Matrix defining an oriented shadow-box volume.
#[derive(
    Default,
    Clone,
    Copy,
    Debug,
    PartialEq,
    m3::Read,
    m3::Write,
    m3::RecordLayout,
    m3::Validate,
    m3::VisitReferences,
)]
pub struct ShadowBox {
    /// Matrix specifying the orientation and placement of the box volume.
    pub matrix: [[f32; 4]; 4],
}

impl Record for ShadowBox {
    const TAG: [u8; 4] = *b"SHBX";
    const VERSION: u32 = 0;
}
