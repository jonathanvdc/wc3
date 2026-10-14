//! Typed curves and property destinations, independent of file clocks and channel numbers.
use crate::model::semantic::profiles::{Profile, Universal};
use crate::model::semantic::{
    ClipId, ClockId, Error, EventId, FeatureId, MaterialId, MeshId, NodeId, Quaternion, SystemId,
    TextureId, TextureTransformId, Values,
};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::marker::PhantomData;
mod curves;
pub use curves::*;

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
/// An object whose properties may be animated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OwnerId {
    /// Shared texture-coordinate transformation.
    TextureTransform(TextureTransformId),
    /// Independently authored scene system.
    System(SystemId),
    /// One attached logical feature.
    Feature(FeatureId),
    /// Model-level property.
    Model,
    /// Node or attached feature.
    Node(NodeId),
    /// Material property.
    Material(MaterialId),
    /// Geometry property.
    Mesh(MeshId),
}
/// Semantic destination of an animation channel.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PropertyTarget {
    /// Local translation.
    Translation(NodeId),
    /// Local rotation quaternion.
    Rotation(NodeId),
    /// Local scale.
    Scale(NodeId),
    /// Node visibility.
    Visibility(NodeId),
    /// Material opacity.
    MaterialOpacity(MaterialId),
    /// Material texture selection for a logical input slot.
    MaterialTexture(MaterialId, u32),
    /// Mesh tint.
    MeshColor(MeshId),
    /// Mesh opacity.
    MeshOpacity(MeshId),
    /// A profile property named independently of its physical channel encoding.
    Named {
        /// Semantic owner.
        owner: OwnerId,
        /// Logical profile property path.
        name: String,
    },
}
/// A property handle whose value type determines the permitted curve shape.
pub struct Property<T> {
    target: PropertyTarget,
    marker: PhantomData<fn() -> T>,
}
impl<T> Clone for Property<T> {
    fn clone(&self) -> Self {
        Self {
            target: self.target.clone(),
            marker: PhantomData,
        }
    }
}
impl<T> std::fmt::Debug for Property<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.target.fmt(f)
    }
}
impl<T> PartialEq for Property<T> {
    fn eq(&self, other: &Self) -> bool {
        self.target == other.target
    }
}
impl<T> Eq for Property<T> {}
impl<T> Property<T> {
    /// Binds a profile-defined destination whose descriptor fixes its value and owner types.
    ///
    /// Custom profiles implement [`PropertyDescriptor`] once per property definition;
    /// callers choose that definition rather than supplying a name and value type independently.
    /// ```compile_fail
    /// use wc3::model::semantic::{FeatureId, Property};
    /// use wc3::model::semantic::profiles::mdlx::properties::light;
    /// fn invalid(owner: FeatureId) -> Property<bool> {
    ///     Property::profile(owner, light::INTENSITY)
    /// }
    /// ```
    pub fn profile<D: PropertyDescriptor<Value = T>>(owner: D::Owner, descriptor: D) -> Self {
        Self::named(owner.into(), descriptor.name())
    }
    /// Decoded profile property path. Public authoring uses typed descriptors.
    pub(crate) fn named(owner: OwnerId, name: impl Into<String>) -> Self {
        Self {
            target: PropertyTarget::Named {
                owner,
                name: name.into(),
            },
            marker: PhantomData,
        }
    }
    /// Logical destination.
    pub fn target(&self) -> &PropertyTarget {
        &self.target
    }
    pub(crate) fn new(target: PropertyTarget) -> Self {
        Self {
            target,
            marker: PhantomData,
        }
    }
}
impl NodeId {
    /// Local translation property.
    pub fn translation(self) -> Property<[f32; 3]> {
        Property::new(PropertyTarget::Translation(self))
    }
    /// Local quaternion property, in x/y/z/w component order.
    pub fn rotation(self) -> Property<Quaternion> {
        Property::new(PropertyTarget::Rotation(self))
    }
    /// Local scale property.
    pub fn scale(self) -> Property<[f32; 3]> {
        Property::new(PropertyTarget::Scale(self))
    }
    /// Visibility property.
    pub fn visibility(self) -> Property<bool> {
        Property::new(PropertyTarget::Visibility(self))
    }
}
impl MaterialId {
    /// Opacity property.
    pub fn opacity(self) -> Property<f32> {
        Property::new(PropertyTarget::MaterialOpacity(self))
    }
    /// Resource choice for a logical texture input.
    pub fn texture(self, slot: u32) -> Property<TextureId> {
        Property::new(PropertyTarget::MaterialTexture(self, slot))
    }
}
/// A profile property definition fixing both its owner and animated value domains.
///
/// Applications can implement this trait for their own vocabulary. A descriptor's
/// name is its logical address, independent of native channel numbers. Graph
/// publication still checks owner identity and conflicting property definitions.
pub trait PropertyDescriptor {
    /// Owner identity accepted by this definition.
    type Owner: Into<OwnerId>;
    /// Animated value domain accepted by this definition.
    type Value;
    /// Logical property path.
    fn name(self) -> Cow<'static, str>;
}
/// A built-in typed property definition. Profile modules publish keys for their vocabulary.
#[derive(Debug)]
pub struct PropertyKey<O, T> {
    name: Cow<'static, str>,
    marker: PhantomData<fn(O) -> T>,
}
impl<O, T> Clone for PropertyKey<O, T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            marker: PhantomData,
        }
    }
}
impl<O: Into<OwnerId>, T> PropertyDescriptor for PropertyKey<O, T> {
    type Owner = O;
    type Value = T;
    fn name(self) -> Cow<'static, str> {
        self.name
    }
}
impl<O: Into<OwnerId>, T> PropertyKey<O, T> {
    pub(crate) const fn new(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            marker: PhantomData,
        }
    }
    pub(crate) fn indexed(name: String) -> Self {
        Self {
            name: Cow::Owned(name),
            marker: PhantomData,
        }
    }
    /// Binds this definition to an owner in a semantic graph.
    pub fn bind(self, owner: O) -> Property<T> {
        Property::profile(owner, self)
    }
}
macro_rules! owner_identity {
    ($($id:ty => $kind:ident),* $(,)?) => {$ (
        impl From<$id> for OwnerId { fn from(id: $id) -> Self { Self::$kind(id) } }
    )*};
}
owner_identity!(NodeId => Node, MaterialId => Material, MeshId => Mesh,
    TextureTransformId => TextureTransform, FeatureId => Feature, SystemId => System);

/// Rest value and optional binding to a typed animated property.
#[derive(Clone, Debug, PartialEq)]
pub struct Animated<T> {
    /// Value when no animation overrides this property.
    pub default: T,
    /// Typed animated destination.
    pub property: Option<Property<T>>,
}
impl<T> Animated<T> {
    /// An unanimated value.
    pub fn constant(default: T) -> Self {
        Self {
            default,
            property: None,
        }
    }
    /// Binds a rest value to its animation destination.
    pub fn new(default: T, property: Property<T>) -> Self {
        Self {
            default,
            property: Some(property),
        }
    }
}
impl<T: Default> Default for Animated<T> {
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
pub struct Track<'a, T: CurveValue> {
    /// Typed animated destination.
    pub target: Property<T>,
    /// Checked key data.
    pub curve: Curve<'a, T>,
    /// Evaluation clock.
    pub clock: Clock,
}
/// Heterogeneous typed channels in one clip.
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
    Texture(Track<'a, TextureId>),
    /// Optional texture selection.
    OptionalTexture(Track<'a, Option<TextureId>>),
}
/// A named playback interval with typed channels and event occurrences.
#[derive(Clone, Debug)]
pub struct Clip<'a, P: Profile = Universal> {
    /// Vocabulary-specific playback controls.
    pub controls: P::Clip<'a>,
    pub(crate) alias: Option<ClipId>,
    pub(crate) name: std::borrow::Cow<'a, str>,
    pub(crate) duration: Time,
    pub(crate) looping: bool,
    pub(crate) channels: Vec<Channel<'a>>,
    pub(crate) events: Vec<Event<'a>>,
}
impl<'a, P: Profile> Clip<'a, P> {
    /// Creates an empty clip.
    pub fn named(name: impl Into<std::borrow::Cow<'a, str>>) -> Self {
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
    pub payload: Values<'a, u8>,
}
/// A clip reference used by format-specific alias and transition metadata.
pub type ClipReference = ClipId;

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
    pub fn target(&self) -> &PropertyTarget {
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
