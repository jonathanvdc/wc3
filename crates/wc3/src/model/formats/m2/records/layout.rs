//! Version families select the native field types of M2 records.
use super::super::codec::Value;
use super::super::{Read, RecordLayout, Write};
use super::{ScrollField, SplineStorage, TimelineStorage, Validate, ValuesStorage, VisitArrays};
use crate::model::common::{Borrowed, Detached};
use std::fmt::Debug;

/// A native record field with codec, validation and traversal contracts.
pub trait FamilyField<'data>:
    Read<'data>
    + Write
    + RecordLayout
    + Clone
    + Debug
    + PartialEq
    + Default
    + Validate
    + VisitArrays<'data>
{
}
impl<'data, T> FamilyField<'data> for T where
    T: Read<'data>
        + Write
        + RecordLayout
        + Clone
        + Debug
        + PartialEq
        + Default
        + Validate
        + VisitArrays<'data>
{
}
/// A field whose representation has no borrowed storage.
pub trait ScalarField: for<'data> FamilyField<'data> + 'static {}
impl<T> ScalarField for T where T: for<'data> FamilyField<'data> + 'static {}

/// The flat or per-sequence animation storage selected by a version.
pub trait TrackLayout {
    /// Timestamp storage, including legacy interpolation ranges where present.
    type Timeline<'data>: TimelineStorage<'data>
        + for<'view> Borrowed<'view, View = Self::Timeline<'view>>
        + for<'owned> Detached<'owned, Detached = Self::Timeline<'owned>>;
    /// Ordinary values or runtime-selected spline triplets.
    type TrackValues<'data, T>: ValuesStorage<'data, T>
        + for<'view> Borrowed<'view, View = Self::TrackValues<'view, T>>
        + for<'owned> Detached<'owned, Detached = Self::TrackValues<'owned, T>>
    where
        T: Value,
        [T; 3]: Value;
    /// Camera spline keys, whose triplet shape is independent of interpolation.
    type SplineValues<'data, T>: SplineStorage<'data, T>
        + for<'view> Borrowed<'view, View = Self::SplineValues<'view, T>>
        + for<'owned> Detached<'owned, Detached = Self::SplineValues<'owned, T>>
    where
        T: Value,
        [T; 3]: Value;
}
/// Bone scalar metadata and quaternion component storage.
pub trait BoneLayout: TrackLayout {
    /// Absent in classic files and a u32 CRC in compressed-bone layouts.
    type NameCrc: ScalarField;
    /// Float or compressed i16 quaternion components.
    type RotationValue: Value + Default;
}
/// Native sequence clock and blending fields.
pub trait SequenceLayout {
    /// A legacy start/end interval or a modern duration word.
    type Timing: ScalarField;
    /// A single blend duration or separate incoming/outgoing words.
    type BlendTime: ScalarField;
}
/// Camera field-of-view placement and storage.
pub trait CameraLayout: TrackLayout {
    /// A scalar before the clip fields, or an absent unit field.
    type FovPrefix: ScalarField;
    /// An appended spline track, or an absent unit field.
    type FovTrack<'data>: FamilyField<'data>
        + for<'view> Borrowed<'view, View = Self::FovTrack<'view>>
        + for<'owned> Detached<'owned, Detached = Self::FovTrack<'owned>>;
}
/// Optional native ribbon metadata selected by a version.
pub trait RibbonLayout: TrackLayout {
    /// Unit for legacy records or the priority/color/transform tail.
    type Tail: ScalarField;
}
/// The native particle record family selected by a version.
pub trait ParticleLayout: TrackLayout {
    /// Wide legacy emitter words or packed bytes and a color index.
    type Header: ScalarField;
    /// Legacy particle/head bytes or modern fixed-point texture scales.
    type TextureScale: ScalarField;
    /// Absent legacy variation or a modern f32 variation.
    type Variation: ScalarField;
    /// Legacy fixed lifetime ramps or modern particle-age tracks.
    type Lifetime<'data>: FamilyField<'data>
        + for<'view> Borrowed<'view, View = Self::Lifetime<'view>>
        + for<'owned> Detached<'owned, Detached = Self::Lifetime<'owned>>;
    /// Legacy spin or the four modern spin parameters.
    type Spin: ScalarField;
    /// Absent, flag-selected, or mandatory scrolling data.
    type Scroll: ScrollField;
}
/// Native skin-section sorting metadata.
pub trait SkinSectionLayout {
    /// Absent classic fields or sort center and radius.
    type Tail: ScalarField;
}
