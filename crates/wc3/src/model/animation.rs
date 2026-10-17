//! Typed curves and property destinations, independent of file clocks and channel numbers.

/// Resettable values with authored sampling controls.
pub mod programmed;
use crate::model::profiles::{Profile, Universal};
use crate::model::{ClipId, ClockId, Error, EventId, FeatureId, NodeId, Quaternion, Texture};
use std::borrow::Cow;
use std::collections::BTreeMap;
use zerovec::{ule::AsULE, ZeroVec};
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
pub mod properties;
pub use properties::*;
mod slots;
pub use slots::*;

/// Clock used to evaluate a track.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Clock {
    /// Active clip time.
    #[default]
    Clip,
    /// Absolute authoring time shared by interval-based playback selections.
    Timeline,
    /// Independent shared looping clock.
    Shared(ClockId),
}
/// Layer identity and ordering for a set of channels.
#[derive(Clone, Debug)]
pub struct Controller<'a, P: Profile = Universal> {
    /// Vocabulary-specific controller behavior.
    pub controls: P::Controller<'a>,
    /// Controller display name.
    pub name: Cow<'a, str>,
    /// Curves owned by this programme, rather than duplicated by playback clips.
    pub channels: Vec<Channel<'a>>,
    /// Event occurrences owned by this programme.
    pub events: BTreeMap<EventId, Event<'a>>,
    /// Evaluation priority.
    pub priority: i32,
    /// Whether this controller runs concurrently.
    pub concurrent: bool,
}
impl<P: Profile> Default for Controller<'_, P> {
    fn default() -> Self {
        Self {
            name: Cow::Borrowed(""),
            priority: 0,
            concurrent: false,
            channels: Vec::new(),
            events: BTreeMap::new(),
            controls: Default::default(),
        }
    }
}
/// One typed animation channel.
#[derive(Clone, Debug)]
pub struct Track<'a, T: CurveValue + Animatable> {
    /// Typed animated destination.
    pub target: Animated<T>,
    /// Checked key data.
    pub curve: Curve<'a, T>,
    /// Evaluation clock.
    pub clock: Clock,
}
/// Heterogeneous channels targeting typed value slots.
///
/// A channel cannot write a slot of another value type:
/// ```compile_fail
/// use wc3::model::{Animated, Clock, Curve, Track};
/// fn invalid(target: Animated<bool>, curve: Curve<'static, f32>) -> Track<'static, f32> {
///     Track { target, curve, clock: Clock::Clip }
/// }
/// ```
#[derive(Clone, Debug)]
pub enum Channel<'a> {
    /// Four unsigned integer components.
    Unsigned4(Track<'a, [u32; 4]>),
    /// Four byte components.
    Bytes4(Track<'a, [u8; 4]>),
    /// Quaternion rotation, including angular interpolation for linear curves.
    Rotation(Track<'a, Quaternion>),
    /// Float scalar.
    Scalar(Track<'a, f32>),
    /// Two float components.
    Vector2(Track<'a, [f32; 2]>),
    /// Three float components.
    Vector3(Track<'a, [f32; 3]>),
    /// Four float components.
    Vector4(Track<'a, [f32; 4]>),
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
/// A named playback interval with typed channels and event occurrences.
#[derive(Clone, Debug)]
pub struct Clip<'a, P: Profile = Universal> {
    /// Vocabulary-specific playback controls.
    pub controls: P::Clip<'a>,
    pub(crate) alias: Option<ClipId>,
    pub(crate) name: Cow<'a, str>,
    pub(crate) duration: Time,
    pub(crate) looping: bool,
    pub(crate) channels: Vec<Channel<'a>>,
    pub(crate) events: Vec<Event<'a>>,
}
impl<'a, P: Profile> Clip<'a, P> {
    /// Creates an empty clip.
    pub fn named(name: impl Into<Cow<'a, str>>) -> Self {
        Self {
            name: name.into(),
            controls: Default::default(),
            alias: None,
            duration: Time::default(),
            looping: true,
            channels: Vec::new(),
            events: Vec::new(),
        }
    }
    /// Sets a finite nonnegative duration.
    pub fn with_duration(mut self, duration: Time) -> Self {
        self.duration = duration;
        self
    }
    /// Uses another clip's channels and event program.
    pub fn with_alias(mut self, target: ClipId) -> Self {
        self.alias = Some(target);
        self
    }
    /// Referenced playback program, when this clip is an alias.
    pub fn alias(&self) -> Option<ClipId> {
        self.alias
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
    /// Typed channel definitions.
    pub fn channels(&self) -> &[Channel<'a>] {
        &self.channels
    }
    /// Event occurrences.
    pub fn events(&self) -> &[Event<'a>] {
        &self.events
    }
}
/// An event occurrence evaluated on a clip's timeline.
#[derive(Clone, Debug)]
pub struct Event<'a> {
    /// Specific event feature, when multiple features share an anchor.
    pub feature: Option<FeatureId>,
    /// Clock on which the occurrence is scheduled.
    pub clock: Clock,
    /// Occurrence time.
    pub time: Time,
    /// Event name.
    pub name: Cow<'a, str>,
    /// Optional node anchor.
    pub node: Option<NodeId>,
    /// Logical payload bytes, when a format defines an event message.
    pub payload: ZeroVec<'a, u8>,
}

impl Channel<'_> {
    /// Clock on which this channel is evaluated.
    pub fn clock(&self) -> Clock {
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
    /// Logical destination of this typed channel.
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
