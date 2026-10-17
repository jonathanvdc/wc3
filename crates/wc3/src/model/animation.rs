//! Typed curves and property destinations, independent of file clocks and channel numbers.
use std::any::TypeId;
/// Resettable values with authored sampling controls.
pub mod programmed;
use crate::model::profiles::{Extends, Narrows, Profile, Universal};
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
mod bindings;
pub use bindings::*;

/// Rest value and optional binding to a typed animated property.
#[derive(Clone, Debug)]
pub struct Animated<T: AnimatedValue, P: Profile = Universal> {
    /// Value when no animation overrides this property.
    pub default: T,
    /// Optional typed read binding.
    pub binding: Option<Binding<T, P>>,
}
impl<T: AnimatedValue, P: Profile> Animated<T, P> {
    /// An unanimated value.
    pub fn constant(default: T) -> Self {
        Self {
            default,
            binding: None,
        }
    }
    /// Binds a rest value to its animation destination.
    pub fn new<Q: Profile>(default: T, property: Property<T, Q>) -> Self
    where
        P: Extends<Q>,
    {
        Self {
            default,
            binding: Some(property.widen::<P>().into()),
        }
    }
}
impl<T: AnimatedValue + Default, P: Profile> Default for Animated<T, P> {
    fn default() -> Self {
        Self::constant(T::default())
    }
}
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
    pub channels: Vec<Channel<'a, P>>,
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
pub struct Track<'a, T: CurveValue, P: Profile = Universal> {
    /// Typed animated destination.
    pub target: Property<T, P>,
    /// Checked key data.
    pub curve: Curve<'a, T>,
    /// Evaluation clock.
    pub clock: Clock,
}
/// Heterogeneous typed channels in one profile's playback programme.
///
/// Profile property vocabularies cannot be mixed implicitly:
/// ```compile_fail
/// use wc3::model::{Channel, Clock, Curve, FeatureId, Time, Track};
/// use wc3::model::profiles::{M2, mdlx::properties::light};
/// fn invalid(feature: FeatureId) {
///     let _: Channel<'static, M2> = Channel::Scalar(Track {
///         target: light::INTENSITY.bind(feature),
///         curve: Curve::step([Time::default()], [1f32]).unwrap(),
///         clock: Clock::Timeline,
///     });
/// }
/// ```
#[derive(Clone, Debug)]
pub enum Channel<'a, P: Profile = Universal> {
    /// Four unsigned integer components.
    Unsigned4(Track<'a, [u32; 4], P>),
    /// Four byte components.
    Bytes4(Track<'a, [u8; 4], P>),
    /// Quaternion rotation, including angular interpolation for linear curves.
    Rotation(Track<'a, Quaternion, P>),
    /// Float scalar.
    Scalar(Track<'a, f32, P>),
    /// Two float components.
    Vector2(Track<'a, [f32; 2], P>),
    /// Three float components.
    Vector3(Track<'a, [f32; 3], P>),
    /// Four float components.
    Vector4(Track<'a, [f32; 4], P>),
    /// Unsigned scalar.
    Unsigned(Track<'a, u32, P>),
    /// Signed scalar.
    Signed(Track<'a, i32, P>),
    /// Unsigned short.
    Unsigned16(Track<'a, u16, P>),
    /// Byte scalar.
    Byte(Track<'a, u8, P>),
    /// Boolean.
    Boolean(Track<'a, bool, P>),
    /// Texture resource.
    Texture(Track<'a, Texture, P>),
    /// Optional texture selection.
    OptionalTexture(Track<'a, Option<Texture>, P>),
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
    pub(crate) channels: Vec<Channel<'a, P>>,
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
    pub fn channels(&self) -> &[Channel<'a, P>] {
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

impl<P: Profile> Channel<'_, P> {
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
    pub fn target(&self) -> &PropertyTarget<P> {
        match self {
            Self::Unsigned4(track) => track.target.target(),
            Self::Bytes4(track) => track.target.target(),
            Self::Rotation(track) => track.target.target(),
            Self::Scalar(track) => track.target.target(),
            Self::Vector2(track) => track.target.target(),
            Self::Vector3(track) => track.target.target(),
            Self::Vector4(track) => track.target.target(),
            Self::Unsigned(track) => track.target.target(),
            Self::Signed(track) => track.target.target(),
            Self::Unsigned16(track) => track.target.target(),
            Self::Byte(track) => track.target.target(),
            Self::Boolean(track) => track.target.target(),
            Self::Texture(track) => track.target.target(),
            Self::OptionalTexture(track) => track.target.target(),
        }
    }
}

impl<'a, T: CurveValue, P: Profile> Track<'a, T, P> {
    fn map_profile<Q: Profile>(
        self,
        map: impl Fn(P::Property) -> Result<Q::Property, Error>,
    ) -> Result<Track<'a, T, Q>, Error> {
        Ok(Track {
            target: Property::new(self.target.target().clone().map_profile(map)?),
            curve: self.curve,
            clock: self.clock,
        })
    }
}
impl<'a, P: Profile> Channel<'a, P> {
    pub(crate) fn map_profile<Q: Profile>(
        self,
        map: impl Fn(P::Property) -> Result<Q::Property, Error> + Copy,
    ) -> Result<Channel<'a, Q>, Error> {
        Ok(match self {
            Self::Unsigned4(track) => Channel::Unsigned4(track.map_profile(map)?),
            Self::Bytes4(track) => Channel::Bytes4(track.map_profile(map)?),
            Self::Rotation(track) => Channel::Rotation(track.map_profile(map)?),
            Self::Scalar(track) => {
                let track = track.map_profile(map)?;
                if track.target.target().value_type() == Some(TypeId::of::<bool>()) {
                    if track.curve.interpolation() != Interpolation::Step
                        || track
                            .curve
                            .values()
                            .iter()
                            .any(|value| value != 0. && value != 1.)
                    {
                        return Err(Error::new(
                            "socket.visibility",
                            "boolean socket visibility requires binary step keys",
                        ));
                    }
                    Channel::Boolean(Track {
                        target: Property::new(track.target.target().clone()),
                        curve: Curve::step(
                            track.curve.times().clone(),
                            track
                                .curve
                                .values()
                                .iter()
                                .map(|value| value == 1.)
                                .collect::<Vec<_>>(),
                        )?,
                        clock: track.clock,
                    })
                } else {
                    Channel::Scalar(track)
                }
            }
            Self::Vector2(track) => Channel::Vector2(track.map_profile(map)?),
            Self::Vector3(track) => Channel::Vector3(track.map_profile(map)?),
            Self::Vector4(track) => Channel::Vector4(track.map_profile(map)?),
            Self::Unsigned(track) => Channel::Unsigned(track.map_profile(map)?),
            Self::Signed(track) => Channel::Signed(track.map_profile(map)?),
            Self::Unsigned16(track) => Channel::Unsigned16(track.map_profile(map)?),
            Self::Byte(track) => Channel::Byte(track.map_profile(map)?),
            Self::Boolean(track) => Channel::Boolean(track.map_profile(map)?),
            Self::Texture(track) => Channel::Texture(track.map_profile(map)?),
            Self::OptionalTexture(track) => Channel::OptionalTexture(track.map_profile(map)?),
        })
    }
    /// Embeds a typed programme in a containing vocabulary while sharing its curve arrays.
    pub fn widen<Q: Extends<P>>(self) -> Channel<'a, Q> {
        self.map_profile(|key| Ok(Q::property(key)))
            .expect("infallible property embedding")
    }
    /// Projects a typed programme exactly, converting binary scalar step keys when required.
    pub fn try_narrow<Q: Narrows<P>>(self) -> Result<Channel<'a, Q>, Error> {
        self.map_profile(Q::property)
    }
}

impl<T: AnimatedValue + PartialEq, P: Profile, Q: Profile> PartialEq<Animated<T, Q>>
    for Animated<T, P>
{
    fn eq(&self, other: &Animated<T, Q>) -> bool {
        self.default == other.default
            && match (&self.binding, &other.binding) {
                (None, None) => true,
                (Some(a), Some(b)) => a.equivalent::<Q>(b),
                _ => false,
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
