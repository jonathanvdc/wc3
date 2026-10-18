//! Explicit playback selection and typed evaluation of semantic animation.
use super::curves::{CurveValue, SampleValue};
use crate::model::profiles::Profile;
use crate::model::{
    AnimationId, Channel, ClipId, ClockId, ControllerId, Error, Model, Property, PropertyValue,
    PropertyValues, Quaternion, Texture, Time, TimeSource, Track, ValueSlot,
};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};

/// TimeSource values and explicitly selected programmes for one evaluation.
///
/// The selected clip maps local time into its authored source interval. Without
/// a clip, ordinary tracks use `clip_time` directly without interval bounds. The
/// caller chooses controllers according to its gameplay policy. Shared clocks wrap
/// `shared_time` by their authored periods; frozen clocks retain time zero.
/// This operation does not dispatch events.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationSample<'a> {
    /// Selected playback clip.
    pub clip: Option<ClipId>,
    /// Seconds from the beginning of the selected clip; wrapped or clamped.
    pub clip_time: Time,
    /// Time driving all independent repeating clocks.
    pub shared_time: Time,
    /// Explicitly enabled controllers, in tie-breaking order.
    pub controllers: &'a [ControllerId],
}

/// A validated playback selection that can evaluate many typed properties.
///
/// Model-level animations precede selected clip animations, then selected
/// controllers in ascending priority order. Selection order is authoritative. Later
/// matching channels override earlier channels. Equal-priority controllers keep
/// caller order. Controller concurrency is playback policy, not implicit selection.
pub struct SampledAnimation<'view, 'data> {
    values: &'view PropertyValues,
    clocks: Cow<'view, BTreeMap<ClockId, Time>>,
    clip_policy: Option<(Time, bool)>,
    animation_time: Time,
    source_interval: Option<(Time, Time)>,
    shared_time: Time,
    channels: ChannelSelection<'view, 'data>,
}

enum ChannelSelection<'view, 'data> {
    Flat(Vec<&'view Channel<'data>>),
    Indexed(IndexedSelection<'view>),
}
// Ordinary clips need only inline borrowed programme references. Controller
// storage allocates only when the caller explicitly selects controllers.
struct IndexedSelection<'view> {
    animations: &'view BTreeMap<AnimationId, IndexedChannels>,
    global: &'view [AnimationId],
    clip: &'view [AnimationId],
    controllers: Vec<&'view ControllerProgram>,
}
impl<'view> IndexedSelection<'view> {
    fn iter(&self) -> impl DoubleEndedIterator<Item = &'view IndexedChannels> + '_ {
        self.global
            .iter()
            .chain(self.clip)
            .chain(
                self.controllers
                    .iter()
                    .flat_map(|controller| &controller.animations),
            )
            .map(|id| &self.animations[id])
    }
}
#[derive(Clone, Debug)]
struct IndexedChannels {
    channels: Vec<Channel<'static>>,
    index: HashMap<ValueSlot, Vec<usize>>,
}
impl IndexedChannels {
    fn new(channels: Vec<Channel<'static>>) -> Self {
        let mut index: HashMap<ValueSlot, Vec<usize>> = HashMap::new();
        for (ordinal, channel) in channels.iter().enumerate() {
            index.entry(channel.target()).or_default().push(ordinal);
        }
        Self { channels, index }
    }
}

/// A value represented by a typed semantic animation channel.
pub trait ChannelValue: SampleValue + PropertyValue {
    /// Borrows the matching typed track when this channel carries this value type.
    fn channel_track<'view, 'data>(
        channel: &'view Channel<'data>,
    ) -> Option<&'view Track<'data, Self>>;
}
macro_rules! channel_value {
    ($($ty:ty => $variant:ident),* $(,)?) => {$ (
        impl ChannelValue for $ty {
            fn channel_track<'view, 'data>(channel: &'view Channel<'data>) -> Option<&'view Track<'data, Self>> {
                if let Channel::$variant(track) = channel { Some(track) } else { None }
            }
        }
    )* };
}
channel_value! {
    [u32; 4] => Unsigned4, [u8; 4] => Bytes4, Quaternion => Rotation,
    f32 => Scalar, [f32; 2] => Vector2, [f32; 3] => Vector3,
    [f32; 4] => Vector4, u32 => Unsigned, i32 => Signed,
    u16 => Unsigned16, u8 => Byte, bool => Boolean,
    Texture => Texture, Option<Texture> => OptionalTexture,
}

impl<'data, P: Profile> Model<'data, P> {
    /// Resolves playback identities once before evaluating animated properties.
    pub fn animation_sample<'view>(
        &'view self,
        sample: AnimationSample<'_>,
    ) -> Result<SampledAnimation<'view, 'data>, Error> {
        let mut channels: Vec<_> = self.global_channels().collect();
        let (clip_policy, animation_time, source_interval) = if let Some(id) = sample.clip {
            let clip = self
                .clip(id)
                .ok_or_else(|| Error::new("animation.sample.clip", "unknown clip identity"))?;
            channels.extend(self.clip_channels(id));
            resolve_times(
                sample,
                clip.duration(),
                clip.is_looping(),
                clip.source_start,
            )?
        } else {
            (None, sample.clip_time, None)
        };
        let mut controllers = Vec::with_capacity(sample.controllers.len());
        for (index, id) in sample.controllers.iter().enumerate() {
            if sample.controllers[..index].contains(id) {
                return Err(Error::new(
                    "animation.sample.controllers",
                    "controller selected more than once",
                ));
            }
            controllers.push(self.controller(*id).ok_or_else(|| {
                Error::new(
                    "animation.sample.controllers",
                    "unknown controller identity",
                )
            })?);
        }
        controllers.sort_by_key(|controller| controller.priority);
        for controller in controllers {
            channels.extend(self.animation_channels(&controller.animations));
        }
        Ok(SampledAnimation {
            values: self.values(),
            clocks: Cow::Owned(
                self.clocks()
                    .iter()
                    .map(|(id, clock)| (id, clock.period()))
                    .collect(),
            ),
            clip_policy,
            animation_time,
            source_interval,
            shared_time: sample.shared_time,
            channels: ChannelSelection::Flat(channels),
        })
    }
}

impl<'view, 'data> SampledAnimation<'view, 'data> {
    /// Evaluates every slot into instance-owned storage, resetting from authored constants.
    pub fn evaluate_into(&self, output: &mut PropertyValues) {
        self.values.evaluate_into(self, output);
    }
    /// Evaluates every slot into a new table with the same identities.
    pub fn evaluate(&self) -> PropertyValues {
        let mut values = PropertyValues::default();
        self.evaluate_into(&mut values);
        values
    }
    /// Authored constants and explicit value conversions.
    pub fn values(&self) -> &PropertyValues {
        self.values
    }
    /// Resolved inclusive authoring interval of the selected clip.
    pub fn source_interval(&self) -> Option<(Time, Time)> {
        self.source_interval
    }
    /// Resolved authoring time after applying the selected clip's mapping.
    pub fn animation_time(&self) -> Time {
        self.animation_time
    }
    /// Absolute shared-clock input before individual periods wrap it.
    /// Useful for continuous procedural controls independent of clip looping.
    pub fn shared_time(&self) -> Time {
        self.shared_time
    }

    /// Evaluates the selected channels for a typed property, or returns no override.
    pub fn override_value<T: ChannelValue>(&self, property: &Property<T>) -> Option<T> {
        self.override_at(property, self.times())
    }
    fn override_at<T: ChannelValue>(
        &self,
        property: &Property<T>,
        times: SampleTimes,
    ) -> Option<T> {
        match &self.channels {
            ChannelSelection::Flat(channels) => channels
                .iter()
                .rev()
                .find_map(|channel| self.sample_channel(channel, property, times)),
            ChannelSelection::Indexed(programmes) => {
                programmes.iter().rev().find_map(|programme| {
                    programme
                        .index
                        .get(&property.slot())?
                        .iter()
                        .rev()
                        .find_map(|index| {
                            self.sample_channel(&programme.channels[*index], property, times)
                        })
                })
            }
        }
    }
    fn sample_channel<T: ChannelValue>(
        &self,
        channel: &Channel<'_>,
        property: &Property<T>,
        times: SampleTimes,
    ) -> Option<T> {
        let track = T::channel_track(channel)?;
        if track.target != *property {
            return None;
        }
        let time =
            if let Some((_, time)) = times.occurrence.filter(|(clock, _)| *clock == track.clock) {
                time
            } else {
                match track.clock {
                    TimeSource::Animation => times.animation,
                    TimeSource::Shared(id) => {
                        let period = self.clocks.get(&id)?.as_seconds();
                        if period > 0. {
                            Time::seconds(times.shared.as_seconds().rem_euclid(period)).ok()?
                        } else {
                            Time::default()
                        }
                    }
                }
            };
        if let (TimeSource::Animation, Some((start, end))) = (track.clock, self.source_interval) {
            track.curve.sample_in(time, start..=end)
        } else {
            track.curve.sample(time)
        }
    }

    fn times(&self) -> SampleTimes {
        SampleTimes {
            animation: self.animation_time,
            shared: self.shared_time,
            occurrence: None,
        }
    }
    /// Borrows this programme selection at different clock values, without allocating.
    ///
    /// Local clip time wraps or clamps using the selected clip's own policy,
    /// including an alias's duration. Animation time is derived from the source
    /// start, and the existing inclusive key interval is retained.
    /// Shared clocks independently wrap the supplied shared time. No programmes
    /// are selected again and no events are dispatched.
    pub fn at_times(
        &self,
        clip_time: Time,
        shared_time: Time,
    ) -> RetimedAnimation<'_, 'view, 'data> {
        let clip = normalize_clip(clip_time, self.clip_policy);
        RetimedAnimation {
            selection: self,
            times: SampleTimes {
                animation: self.source_interval.map_or(clip, |(origin, _)| {
                    Time::seconds(origin.as_seconds() + clip.as_seconds())
                        .expect("validated mapping")
                }),
                shared: shared_time,
                occurrence: None,
            },
        }
    }
    /// Borrows the effective typed source track and its clock metadata.
    /// Empty curves and curves without keys in the selected source interval
    /// fall through to lower-priority tracks, exactly as value sampling does.
    pub fn track<T: ChannelValue>(
        &self,
        property: &Property<T>,
    ) -> Option<SampledTrack<'_, 'data, T>> {
        self.track_at(property, self.times())
    }
    fn track_at<T: ChannelValue>(
        &self,
        property: &Property<T>,
        times: SampleTimes,
    ) -> Option<SampledTrack<'_, 'data, T>> {
        let resolve = |channel: &'view Channel<'data>| {
            self.sample_channel(channel, property, times)?;
            let track = T::channel_track(channel)?;
            let (time, period, origin, interval) = match track.clock {
                TimeSource::Animation => (
                    times.animation,
                    self.clip_policy.and_then(|(duration, looping)| {
                        (looping && duration.as_seconds() > 0. && self.source_interval.is_some())
                            .then_some(duration)
                    }),
                    self.source_interval
                        .map_or(Time::default(), |(start, _)| start),
                    self.source_interval,
                ),
                TimeSource::Shared(id) => {
                    let period = *self.clocks.get(&id)?;
                    (
                        if period.as_seconds() > 0. {
                            Time::seconds(times.shared.as_seconds().rem_euclid(period.as_seconds()))
                                .ok()?
                        } else {
                            Time::default()
                        },
                        (period.as_seconds() > 0.).then_some(period),
                        Time::default(),
                        None,
                    )
                }
            };
            Some(SampledTrack {
                track,
                time: times
                    .occurrence
                    .filter(|(clock, _)| *clock == track.clock)
                    .map_or(time, |(_, time)| time),
                period,
                origin,
                interval,
            })
        };
        match &self.channels {
            ChannelSelection::Flat(channels) => {
                channels.iter().rev().find_map(|channel| resolve(channel))
            }
            ChannelSelection::Indexed(programmes) => {
                programmes.iter().rev().find_map(|programme| {
                    programme
                        .index
                        .get(&property.slot())?
                        .iter()
                        .rev()
                        .find_map(|index| resolve(&programme.channels[*index]))
                })
            }
        }
    }
    /// Evaluates an animated value, preserving its authored default without a channel.
    pub fn animated<T: ChannelValue>(&self, value: &Property<T>) -> T {
        self.override_value(value)
            .unwrap_or_else(|| T::resolve(self.values, *value, self))
    }
}

#[derive(Clone, Copy)]
struct SampleTimes {
    animation: Time,
    shared: Time,
    occurrence: Option<(TimeSource, Time)>,
}
fn normalize_clip(time: Time, policy: Option<(Time, bool)>) -> Time {
    let Some((duration, looping)) = policy else {
        return time;
    };
    let duration = duration.as_seconds();
    let value = if duration <= 0. {
        0.
    } else if looping {
        time.as_seconds().rem_euclid(duration)
    } else {
        time.as_seconds().clamp(0., duration)
    };
    Time::seconds(value).expect("normalizing finite time by a validated clip duration")
}
/// Allocation-free clock override borrowing an existing programme selection.
/// Timeline offsets are explicit, as in [`AnimationSample`].
pub struct RetimedAnimation<'sample, 'view, 'data> {
    selection: &'sample SampledAnimation<'view, 'data>,
    times: SampleTimes,
}
impl RetimedAnimation<'_, '_, '_> {
    /// Samples an occurrence's own clock at its exact authored key, including
    /// inclusive loop-end keys. Other clocks keep their normal wrapped times.
    pub fn at_occurrence(mut self, clock: TimeSource, time: Time) -> Self {
        self.times.occurrence = Some((clock, time));
        self
    }
    /// Evaluates the same selected programmes at the overridden clocks.
    pub fn override_value<T: ChannelValue>(&self, property: &Property<T>) -> Option<T> {
        self.selection.override_at(property, self.times)
    }
    /// Preserves an authored default when no selected channel supplies a value.
    pub fn animated<T: ChannelValue>(&self, value: &Property<T>) -> T {
        self.override_value(value)
            .unwrap_or_else(|| T::resolve(self.selection.values, *value, self))
    }
    /// Borrows the effective source track at the overridden clock values.
    pub fn track<T: ChannelValue>(
        &self,
        property: &Property<T>,
    ) -> Option<SampledTrack<'_, '_, T>> {
        self.selection.track_at(property, self.times)
    }
}
/// A borrowed effective curve together with its selected clock interpretation.
///
/// `time` is resolved for value sampling. To enumerate crossed keys, callers keep
/// their own unwrapped clock times and use `origin`/`period` to enumerate cycles.
/// A bounded timeline's origin is its inclusive interval start; shared programs use
/// the selected alias duration rather than the referenced programme's duration.
pub struct SampledTrack<'sample, 'data, T: CurveValue + PropertyValue> {
    /// Effective typed source, without copying its packed key arrays.
    pub track: &'sample Track<'data, T>,
    /// Resolved local time used for the sampled value.
    pub time: Time,
    /// Repeating period, absent for nonlooping clocks.
    pub period: Option<Time>,
    /// Absolute origin of a bounded authoring timeline, otherwise zero.
    pub origin: Time,
    /// Inclusive source-key selection, absent without a clip and for shared tracks.
    pub interval: Option<(Time, Time)>,
}

impl<T: CurveValue + PropertyValue> Track<'_, T> {
    /// Detaches borrowed curve storage for a persistent runtime programme.
    pub fn into_owned(self) -> Track<'static, T> {
        Track {
            target: self.target,
            curve: self.curve.into_owned(),
            clock: self.clock,
        }
    }
}
impl Channel<'_> {
    /// Detaches borrowed curve storage without translating property identities.
    pub fn into_owned(self) -> Channel<'static> {
        macro_rules! owned { ($($variant:ident),*) => { match self { $(Self::$variant(track) => Channel::$variant(track.into_owned()),)* } }; }
        owned!(
            Unsigned4,
            Bytes4,
            Rotation,
            Scalar,
            Vector2,
            Vector3,
            Vector4,
            Unsigned,
            Signed,
            Unsigned16,
            Byte,
            Boolean,
            Texture,
            OptionalTexture
        )
    }
}

#[derive(Clone, Debug)]
struct ClipProgram {
    duration: Time,
    looping: bool,
    source_start: Time,
    animations: Vec<AnimationId>,
}
#[derive(Clone, Debug)]
struct ControllerProgram {
    priority: i32,
    animations: Vec<AnimationId>,
}
/// Owned semantic animation data, independent of geometry and source asset lifetime.
///
/// Compilation retains property identities and interpolation controls; it does not
/// bake frames, select gameplay controllers, or retain the source model graph.
#[derive(Clone, Debug)]
pub struct AnimationProgram {
    values: PropertyValues,
    clocks: BTreeMap<ClockId, Time>,
    animations: BTreeMap<AnimationId, IndexedChannels>,
    global: Vec<AnimationId>,
    clips: BTreeMap<ClipId, ClipProgram>,
    controllers: BTreeMap<ControllerId, ControllerProgram>,
}
impl AnimationProgram {
    /// Authored constants retained by this compiled program.
    pub fn values(&self) -> &PropertyValues {
        &self.values
    }
    /// Copies each reusable animation once and retains playback selections.
    pub fn compile<P: Profile>(model: &Model<'_, P>) -> Self {
        let own = |channels: &[Channel<'_>]| {
            IndexedChannels::new(channels.iter().cloned().map(Channel::into_owned).collect())
        };
        Self {
            values: model.values().clone(),
            clocks: model
                .clocks()
                .iter()
                .map(|(id, clock)| (id, clock.period()))
                .collect(),
            animations: model
                .animations()
                .iter()
                .map(|(id, animation)| (id, own(&animation.channels)))
                .collect(),
            global: model.global_animations().to_vec(),
            clips: model
                .clips()
                .iter()
                .map(|(id, clip)| {
                    (
                        id,
                        ClipProgram {
                            duration: clip.duration(),
                            looping: clip.is_looping(),
                            source_start: clip.source_start,
                            animations: clip.animations.clone(),
                        },
                    )
                })
                .collect(),
            controllers: model
                .controllers()
                .iter()
                .map(|(id, controller)| {
                    (
                        id,
                        ControllerProgram {
                            priority: controller.priority,
                            animations: controller.animations.clone(),
                        },
                    )
                })
                .collect(),
        }
    }
    /// Combines disjoint property owners on the receiver's playback clocks.
    ///
    /// `clips` pairs destination clip identities with source clips. Both use the
    /// destination's time in seconds; curves are not stretched. Source global and
    /// shared-clock channels remain active. `include` can exclude properties whose
    /// ownership is supplied by another rig. No source clip becomes independently
    /// selectable. Mapped clips must have matching source starts; a failed
    /// combination leaves both inputs unchanged.
    pub fn compose(
        &self,
        source: &Self,
        clips: &[(ClipId, ClipId)],
        mut include: impl FnMut(&Channel<'static>) -> bool,
    ) -> Result<Self, Error> {
        let mut result = self.clone();
        result.values.extend(&source.values)?;
        let mut destinations = HashSet::new();
        for &(destination, source_clip) in clips {
            if !destinations.insert(destination) {
                return Err(Error::new(
                    "animation.compose.clips",
                    "destination clip is mapped more than once",
                ));
            }
            if !self.clips.contains_key(&destination) || !source.clips.contains_key(&source_clip) {
                return Err(Error::new(
                    "animation.compose.clips",
                    "unknown destination or source clip",
                ));
            }
            if self.clips[&destination].source_start != source.clips[&source_clip].source_start {
                return Err(Error::new(
                    "animation.compose.clips",
                    "mapped clips must have matching source starts",
                ));
            }
        }
        for (&id, &period) in &source.clocks {
            if result
                .clocks
                .get(&id)
                .is_some_and(|existing| *existing != period)
            {
                return Err(Error::new(
                    "animation.compose.clocks",
                    "shared clock identity has conflicting periods",
                ));
            }
            result.clocks.insert(id, period);
        }
        let owned: HashSet<_> = self
            .animations
            .values()
            .flat_map(|animation| animation.index.keys())
            .collect();
        for (&id, animation) in &source.animations {
            if result.animations.contains_key(&id) {
                return Err(Error::new(
                    "animation.compose.animations",
                    "animation identities overlap",
                ));
            }
            let mut channels = Vec::new();
            for channel in &animation.channels {
                if !include(channel) {
                    continue;
                }
                if owned.contains(&channel.target()) {
                    return Err(Error::new(
                        "animation.compose.properties",
                        "property owners overlap",
                    ));
                }
                channels.push(channel.clone());
            }
            result.animations.insert(id, IndexedChannels::new(channels));
        }
        result.global.extend(&source.global);
        for &(destination, source_clip) in clips {
            result
                .clips
                .get_mut(&destination)
                .unwrap()
                .animations
                .extend(&source.clips[&source_clip].animations);
        }
        for (&id, controller) in &source.controllers {
            if result.controllers.insert(id, controller.clone()).is_some() {
                return Err(Error::new(
                    "animation.compose.controllers",
                    "controller identities overlap",
                ));
            }
        }
        Ok(result)
    }
    /// Selects clocks and programmes using the same precedence as `Model::animation_sample`.
    pub fn sample(
        &self,
        sample: AnimationSample<'_>,
    ) -> Result<SampledAnimation<'_, 'static>, Error> {
        let mut selected_clip: &[AnimationId] = &[];
        let (clip_policy, animation_time, source_interval) = if let Some(id) = sample.clip {
            let clip = self
                .clips
                .get(&id)
                .ok_or_else(|| Error::new("animation.sample.clip", "unknown clip identity"))?;
            selected_clip = &clip.animations;
            resolve_times(sample, clip.duration, clip.looping, clip.source_start)?
        } else {
            (None, sample.clip_time, None)
        };
        let mut controllers = Vec::with_capacity(sample.controllers.len());
        for (index, id) in sample.controllers.iter().enumerate() {
            if sample.controllers[..index].contains(id) {
                return Err(Error::new(
                    "animation.sample.controllers",
                    "controller selected more than once",
                ));
            }
            controllers.push(self.controllers.get(id).ok_or_else(|| {
                Error::new(
                    "animation.sample.controllers",
                    "unknown controller identity",
                )
            })?);
        }
        controllers.sort_by_key(|controller| controller.priority);
        Ok(SampledAnimation {
            values: &self.values,
            clocks: Cow::Borrowed(&self.clocks),
            clip_policy,
            animation_time,
            source_interval,
            shared_time: sample.shared_time,
            channels: ChannelSelection::Indexed(IndexedSelection {
                animations: &self.animations,
                global: &self.global,
                clip: selected_clip,
                controllers,
            }),
        })
    }
}

type ResolvedTimes = (Option<(Time, bool)>, Time, Option<(Time, Time)>);
fn resolve_times(
    sample: AnimationSample<'_>,
    duration: Time,
    looping: bool,
    origin: Time,
) -> Result<ResolvedTimes, Error> {
    let policy = Some((duration, looping));
    let clip = normalize_clip(sample.clip_time, policy);
    let animation = Time::seconds(origin.as_seconds() + clip.as_seconds())?;
    let interval = Some((
        origin,
        Time::seconds(origin.as_seconds() + duration.as_seconds())?,
    ));
    Ok((policy, animation, interval))
}

/// Typed slot reads shared by ordinary and occurrence-time animation samples.
/// Consumers can evaluate poses and surfaces without changing the selected programmes.
pub trait ValueSample {
    /// Authored constants for the selected model.
    fn values(&self) -> &PropertyValues;
    /// Samples an override, returning `None` when no selected channel writes it.
    fn override_value<T: ChannelValue>(&self, property: &Property<T>) -> Option<T>;
    /// Reads a slot, using its authored constant when no selected writer supplies it.
    ///
    /// Panics if the slot is absent from this model. Use `values().get(slot)` to
    /// check an identity before reading an externally supplied reference.
    fn animated<T: ChannelValue>(&self, value: &Property<T>) -> T {
        self.override_value(value)
            .unwrap_or_else(|| T::resolve(self.values(), *value, self))
    }
}
impl ValueSample for SampledAnimation<'_, '_> {
    fn values(&self) -> &PropertyValues {
        self.values
    }
    fn override_value<T: ChannelValue>(&self, property: &Property<T>) -> Option<T> {
        self.override_value(property)
    }
}
impl ValueSample for RetimedAnimation<'_, '_, '_> {
    fn values(&self) -> &PropertyValues {
        self.selection.values
    }
    fn override_value<T: ChannelValue>(&self, property: &Property<T>) -> Option<T> {
        self.override_value(property)
    }
}

impl ValueSample for PropertyValues {
    fn values(&self) -> &PropertyValues {
        self
    }
    fn override_value<T: ChannelValue>(&self, slot: &Property<T>) -> Option<T> {
        self.get(*slot).copied()
    }
}

#[cfg(test)]
#[path = "sampling_tests.rs"]
mod tests;
