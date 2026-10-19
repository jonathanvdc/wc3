//! Typed curves and property destinations, independent of file clocks and track numbers.

use crate::model::common::packed_identity;
use crate::model::profiles::{Profile, Universal};
use crate::model::{AnimationId, ClockId, Error, EventId, EventSourceId, Quat, Texture};
use glam::{Vec2, Vec3, Vec4};
use std::borrow::Cow;
use std::collections::BTreeMap;
use zerovec::ule::AsULE;
pub mod curves;
pub use curves::*;
mod sampling;
pub use sampling::*;

/// Finite timeline time in seconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Time(f64);
impl Time {
    /// Constructs a finite timeline time, including keys before playback begins.
    pub fn seconds(value: f64) -> Result<Self, Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(Error::new("time", "seconds must be finite"))
        }
    }
    /// Time in seconds.
    pub const fn as_seconds(self) -> f64 {
        self.0
    }
}
/// Interpolation behavior of a value curve.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Interpolation {
    /// Holds each key until the next key.
    Step,
    /// Interpolates between neighboring values.
    #[default]
    Linear,
    /// Explicit spherical interpolation between quaternion rotations.
    Slerp,
    /// Normalize quaternion keys, choose their shortest arc, linearly blend
    /// components, and normalize the result.
    Nlerp,
    /// Uses incoming and outgoing derivatives.
    Hermite,
    /// Uses incoming and outgoing control points.
    Bezier,
    /// Kochanek–Bartels spline with authored tension, continuity, and bias per key.
    Tcb,
}

mod slots;
pub use slots::*;

/// Time coordinate used to evaluate a track or event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeSource {
    /// Authored animation time, mapped from the selected clip source interval.
    #[default]
    Animation,
    /// Independent shared looping clock.
    Shared(ClockId),
}
/// Reusable curves and event occurrences, independent of playback selections.
#[derive(Clone, Debug, Default)]
pub struct Animation<'a> {
    /// Typed curves in evaluation order; later applicable curves win.
    pub tracks: Vec<DynamicTrack<'a>>,
    /// Identified event occurrences, traversed separately from value sampling.
    pub events: BTreeMap<EventId, EventOccurrence>,
}
/// One typed animation track.
#[derive(Clone, Debug)]
pub struct Track<'a, T: CurveValue + PropertyValue> {
    /// Typed animated destination.
    pub target: Property<T>,
    /// Checked key data.
    pub curve: Curve<'a, T>,
    /// Evaluation clock.
    pub clock: TimeSource,
}
/// A track whose value type is selected by its variant.
///
/// Each variant retains a typed `Track<T>`, allowing heterogeneous tracks to
/// share one collection while preserving the relationship between curve and target.
///
/// A track cannot write a slot of another value type:
/// ```compile_fail
/// use wc3::model::{Property, TimeSource, Curve, Track};
/// fn invalid(target: Property<bool>, curve: Curve<'static, f32>) -> Track<'static, f32> {
///     Track { target, curve, clock: TimeSource::Animation }
/// }
/// ```
#[derive(Clone, Debug)]
pub enum DynamicTrack<'a> {
    /// Four unsigned integer components.
    Unsigned4(Track<'a, [u32; 4]>),
    /// Four byte components.
    Bytes4(Track<'a, [u8; 4]>),
    /// Quat rotation, including angular interpolation for linear curves.
    Rotation(Track<'a, Quat>),
    /// Float scalar.
    Scalar(Track<'a, f32>),
    /// Two float components.
    Vector2(Track<'a, Vec2>),
    /// Three float components.
    Vector3(Track<'a, Vec3>),
    /// Four float components.
    Vector4(Track<'a, Vec4>),
    /// Unsigned scalar.
    Unsigned(Track<'a, u32>),
    /// Signed scalar.
    Signed(Track<'a, i32>),
    /// Unsigned short.
    Unsigned16(Track<'a, u16>),
    /// Byte scalar.
    Byte(Track<'a, u8>),
    /// Boolean.
    Boolean(Track<'a, bool>),
    /// Texture resource.
    Texture(Track<'a, Texture>),
    /// Optional texture selection.
    OptionalTexture(Track<'a, Option<Texture>>),
}
/// A named playback selection with timing and reusable animation identities.
#[derive(Clone, Debug)]
pub struct Clip<'a, P: Profile = Universal> {
    /// Vocabulary-specific playback properties.
    pub properties: P::ClipProperties<'a>,
    /// Reusable animations selected by this clip, in evaluation order.
    pub animations: Vec<AnimationId>,
    /// Start of the authored source interval; duration bounds key selection.
    pub source_start: Time,
    pub(crate) name: Cow<'a, str>,
    pub(crate) duration: Time,
    pub(crate) looping: bool,
}
impl<'a, P: Profile> Clip<'a, P> {
    /// Creates an empty clip.
    pub fn named(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            properties: Default::default(),
            animations: Vec::new(),
            source_start: Time::default(),
            duration: Time::default(),
            looping: true,
        }
    }
    /// Sets a finite nonnegative duration.
    pub fn with_duration(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    /// Selects a reusable animation without copying its curves or events.
    pub fn with_animation(mut self, animation: AnimationId) -> Self {
        self.animations.push(animation);
        self
    }
    /// Maps local playback time into an absolute authoring interval.
    pub fn with_source_start(mut self, origin: Time) -> Self {
        self.source_start = origin;
        self
    }
    /// Enables repeating playback.
    pub fn looping(mut self) -> Self {
        self.looping = true;
        self
    }
    /// Disables repeating playback.
    pub fn once(mut self) -> Self {
        self.looping = false;
        self
    }
    /// Clip name.
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Playback duration.
    pub fn duration(&self) -> Time {
        self.duration
    }
    /// Whether the clip repeats.
    pub fn is_looping(&self) -> bool {
        self.looping
    }
}
/// An event occurrence scheduled on an explicit time source.
#[derive(Clone, Debug)]
pub struct EventOccurrence {
    /// Model-owned definition of the action and its evaluation clock.
    pub source: EventSourceId,
    /// Occurrence time on the source's clock.
    pub time: Time,
}

impl DynamicTrack<'_> {
    /// TimeSource on which this track is evaluated.
    pub fn clock(&self) -> TimeSource {
        match self {
            Self::Unsigned4(track) => track.clock,
            Self::Bytes4(track) => track.clock,
            Self::Rotation(track) => track.clock,
            Self::Scalar(track) => track.clock,
            Self::Vector2(track) => track.clock,
            Self::Vector3(track) => track.clock,
            Self::Vector4(track) => track.clock,
            Self::Unsigned(track) => track.clock,
            Self::Signed(track) => track.clock,
            Self::Unsigned16(track) => track.clock,
            Self::Byte(track) => track.clock,
            Self::Boolean(track) => track.clock,
            Self::Texture(track) => track.clock,
            Self::OptionalTexture(track) => track.clock,
        }
    }
    /// Logical destination of this typed track.
    pub fn target(&self) -> ValueSlot {
        match self {
            Self::Unsigned4(track) => track.target.slot(),
            Self::Bytes4(track) => track.target.slot(),
            Self::Rotation(track) => track.target.slot(),
            Self::Scalar(track) => track.target.slot(),
            Self::Vector2(track) => track.target.slot(),
            Self::Vector3(track) => track.target.slot(),
            Self::Vector4(track) => track.target.slot(),
            Self::Unsigned(track) => track.target.slot(),
            Self::Signed(track) => track.target.slot(),
            Self::Unsigned16(track) => track.target.slot(),
            Self::Byte(track) => track.target.slot(),
            Self::Boolean(track) => track.target.slot(),
            Self::Texture(track) => track.target.slot(),
            Self::OptionalTexture(track) => track.target.slot(),
        }
    }
}

impl AsULE for Time {
    type ULE = <f64 as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        self.0.to_unaligned()
    }
    fn from_unaligned(value: Self::ULE) -> Self {
        Self(f64::from_unaligned(value))
    }
}

packed_identity!(Time);
