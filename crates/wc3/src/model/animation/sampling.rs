//! Explicit playback selection and typed evaluation of semantic animation.
use super::curves::{CurveValue, SampleValue};
use crate::model::profiles::{Profile, Universal};
use crate::model::{
    Animated, Channel, ClipId, Clock, ClockId, ControllerId, Error, Model, Property,
    PropertyTarget, Quaternion, TextureId, Time, Track,
};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

/// Clock values and explicitly selected programmes for one evaluation.
///
/// The caller supplies authoring timeline time independently of local clip time,
/// and chooses controllers according to its gameplay policy. Shared clocks wrap
/// `shared_time` by their authored periods. This operation does not dispatch events.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationSample<'a> {
    /// Selected playback clip, including aliases.
    pub clip: Option<ClipId>,
    /// Seconds from the beginning of the selected clip; wrapped or clamped.
    pub clip_time: Time,
    /// Absolute authoring timeline time.
    pub timeline_time: Time,
    /// Optional inclusive key-selection interval for authored timeline programmes.
    /// Keys in neighboring sequences cannot influence interpolation within it.
    pub timeline_interval: Option<(Time, Time)>,
    /// Time driving all independent repeating clocks.
    pub shared_time: Time,
    /// Explicitly enabled controllers, in tie-breaking order.
    pub controllers: &'a [ControllerId],
}

/// A validated playback selection that can evaluate many typed properties.
///
/// Global timeline channels precede global shared channels, then the selected
/// clip programme, then selected controllers in ascending priority order. Later
/// matching channels override earlier channels. Equal-priority controllers keep
/// caller order. Controller concurrency is playback policy, not implicit selection.
pub struct SampledAnimation<'view, 'data, P: Profile = Universal> {
    clocks: Cow<'view, BTreeMap<ClockId, Time>>,
    clip_time: Time,
    clip_policy: Option<(Time, bool)>,
    timeline_time: Time,
    timeline_interval: Option<(Time, Time)>,
    shared_time: Time,
    channels: ChannelSelection<'view, 'data, P>,
}

enum ChannelSelection<'view, 'data, P: Profile> {
    Flat(Vec<&'view Channel<'data, P>>),
    Indexed(Vec<&'view IndexedChannels<P>>),
}
#[derive(Clone, Debug)]
struct IndexedChannels<P: Profile> {
    channels: Vec<Channel<'static, P>>,
    index: HashMap<PropertyTarget<P>, Vec<usize>>,
}
impl<P: Profile> IndexedChannels<P> {
    fn new(channels: Vec<Channel<'static, P>>) -> Self {
        let mut index: HashMap<PropertyTarget<P>, Vec<usize>> = HashMap::new();
        for (ordinal, channel) in channels.iter().enumerate() {
            index
                .entry(channel.target().clone())
                .or_default()
                .push(ordinal);
        }
        Self { channels, index }
    }
}

/// A value represented by a typed semantic animation channel.
pub trait ChannelValue: SampleValue {
    /// Borrows the matching typed track when this channel carries this value type.
    fn channel_track<'view, 'data, P: Profile>(
        channel: &'view Channel<'data, P>,
    ) -> Option<&'view Track<'data, Self, P>>;
}
macro_rules! channel_value {
    ($($ty:ty => $variant:ident),* $(,)?) => {$ (
        impl ChannelValue for $ty {
            fn channel_track<'view, 'data, P: Profile>(channel: &'view Channel<'data, P>) -> Option<&'view Track<'data, Self, P>> {
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
    TextureId => Texture, Option<TextureId> => OptionalTexture,
}

impl<'data, P: Profile> Model<'data, P> {
    /// Resolves playback identities once before evaluating animated properties.
    pub fn animation_sample<'view>(
        &'view self,
        sample: AnimationSample<'_>,
    ) -> Result<SampledAnimation<'view, 'data, P>, Error> {
        let mut channels: Vec<_> = self
            .timeline_channels()
            .iter()
            .chain(self.shared_channels())
            .collect();
        validate_interval(sample.timeline_interval)?;
        let mut clip_time = sample.clip_time;
        let mut clip_policy = None;
        if let Some(id) = sample.clip {
            let clip = self
                .clip(id)
                .ok_or_else(|| Error::new("animation.sample.clip", "unknown clip identity"))?;
            clip_policy = Some((clip.duration(), clip.is_looping()));
            let duration = clip.duration().as_seconds();
            let seconds = clip_time.as_seconds();
            clip_time = Time::seconds(if duration > 0. {
                if clip.is_looping() {
                    seconds.rem_euclid(duration)
                } else {
                    seconds.clamp(0., duration)
                }
            } else {
                0.
            })?;
            let program = self
                .clip_program(id)
                .ok_or_else(|| Error::new("animation.sample.clip", "missing alias programme"))?;
            channels.extend(program.channels());
        }
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
            channels.extend(&controller.channels);
        }
        Ok(SampledAnimation {
            clocks: Cow::Owned(
                self.clocks()
                    .map(|(id, clock)| (id, clock.period))
                    .collect(),
            ),
            clip_time,
            clip_policy,
            timeline_time: sample.timeline_time,
            timeline_interval: sample.timeline_interval,
            shared_time: sample.shared_time,
            channels: ChannelSelection::Flat(channels),
        })
    }
}

impl<'view, 'data, P: Profile> SampledAnimation<'view, 'data, P> {
    /// Evaluates the selected channels for a typed property, or returns no override.
    pub fn property<T: ChannelValue>(&self, property: &Property<T, P>) -> Option<T> {
        self.property_at(property, self.times())
    }
    fn property_at<T: ChannelValue>(
        &self,
        property: &Property<T, P>,
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
                        .get(property.target())?
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
        channel: &Channel<'_, P>,
        property: &Property<T, P>,
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
                    Clock::Clip => times.clip,
                    Clock::Timeline => times.timeline,
                    Clock::Shared(id) => Time::seconds(
                        times
                            .shared
                            .as_seconds()
                            .rem_euclid(self.clocks.get(&id)?.as_seconds()),
                    )
                    .ok()?,
                }
            };
        if let (Clock::Timeline, Some((start, end))) = (track.clock, self.timeline_interval) {
            track.curve.sample_in(time, start..=end)
        } else {
            track.curve.sample(time)
        }
    }

    fn times(&self) -> SampleTimes {
        SampleTimes {
            clip: self.clip_time,
            timeline: self.timeline_time,
            shared: self.shared_time,
            occurrence: None,
        }
    }
    /// Borrows this programme selection at different clock values, without allocating.
    ///
    /// Local clip time wraps or clamps using the selected clip's own policy,
    /// including an alias's duration. Timeline time remains an explicitly supplied
    /// absolute authoring time; the existing inclusive key interval is retained.
    /// Shared clocks independently wrap the supplied shared time. No programmes
    /// are selected again and no events are dispatched.
    pub fn at_times(
        &self,
        clip_time: Time,
        timeline_time: Time,
        shared_time: Time,
    ) -> RetimedAnimation<'_, 'view, 'data, P> {
        let clip = normalize_clip(clip_time, self.clip_policy);
        RetimedAnimation {
            selection: self,
            times: SampleTimes {
                clip,
                timeline: timeline_time,
                shared: shared_time,
                occurrence: None,
            },
        }
    }
    /// Borrows the effective typed source track and its clock metadata.
    /// Empty curves and curves without keys in the selected timeline interval
    /// fall through to lower-priority tracks, exactly as value sampling does.
    pub fn track<T: ChannelValue>(
        &self,
        property: &Property<T, P>,
    ) -> Option<SampledTrack<'_, 'data, T, P>> {
        self.track_at(property, self.times())
    }
    fn track_at<T: ChannelValue>(
        &self,
        property: &Property<T, P>,
        times: SampleTimes,
    ) -> Option<SampledTrack<'_, 'data, T, P>> {
        let resolve = |channel: &'view Channel<'data, P>| {
            self.sample_channel(channel, property, times)?;
            let track = T::channel_track(channel)?;
            let (time, period, origin, interval) = match track.clock {
                Clock::Clip => (
                    times.clip,
                    self.clip_policy.and_then(|(duration, looping)| {
                        (looping && duration.as_seconds() > 0.).then_some(duration)
                    }),
                    Time::default(),
                    None,
                ),
                Clock::Timeline => (
                    times.timeline,
                    self.clip_policy.and_then(|(duration, looping)| {
                        (looping && duration.as_seconds() > 0. && self.timeline_interval.is_some())
                            .then_some(duration)
                    }),
                    self.timeline_interval
                        .map_or(Time::default(), |(start, _)| start),
                    self.timeline_interval,
                ),
                Clock::Shared(id) => {
                    let period = *self.clocks.get(&id)?;
                    (
                        Time::seconds(times.shared.as_seconds().rem_euclid(period.as_seconds()))
                            .ok()?,
                        Some(period),
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
                        .get(property.target())?
                        .iter()
                        .rev()
                        .find_map(|index| resolve(&programme.channels[*index]))
                })
            }
        }
    }
    /// Evaluates an animated value, preserving its authored default without a channel.
    pub fn animated<T: ChannelValue>(&self, value: &Animated<T, P>) -> T {
        value
            .property
            .as_ref()
            .and_then(|property| self.property(property))
            .unwrap_or(value.default)
    }
}

#[derive(Clone, Copy)]
struct SampleTimes {
    clip: Time,
    timeline: Time,
    shared: Time,
    occurrence: Option<(Clock, Time)>,
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
pub struct RetimedAnimation<'sample, 'view, 'data, P: Profile = Universal> {
    selection: &'sample SampledAnimation<'view, 'data, P>,
    times: SampleTimes,
}
impl<P: Profile> RetimedAnimation<'_, '_, '_, P> {
    /// Samples an occurrence's own clock at its exact authored key, including
    /// inclusive loop-end keys. Other clocks keep their normal wrapped times.
    pub fn at_occurrence(mut self, clock: Clock, time: Time) -> Self {
        self.times.occurrence = Some((clock, time));
        self
    }
    /// Evaluates the same selected programmes at the overridden clocks.
    pub fn property<T: ChannelValue>(&self, property: &Property<T, P>) -> Option<T> {
        self.selection.property_at(property, self.times)
    }
    /// Preserves an authored default when no selected channel supplies a value.
    pub fn animated<T: ChannelValue>(&self, value: &Animated<T, P>) -> T {
        value
            .property
            .as_ref()
            .and_then(|property| self.property(property))
            .unwrap_or(value.default)
    }
    /// Borrows the effective source track at the overridden clock values.
    pub fn track<T: ChannelValue>(
        &self,
        property: &Property<T, P>,
    ) -> Option<SampledTrack<'_, '_, T, P>> {
        self.selection.track_at(property, self.times)
    }
}
/// A borrowed effective curve together with its selected clock interpretation.
///
/// `time` is resolved for value sampling. To enumerate crossed keys, callers keep
/// their own unwrapped clock times and use `origin`/`period` to enumerate cycles.
/// A bounded timeline's origin is its inclusive interval start; clip aliases use
/// the selected alias duration rather than the referenced programme's duration.
pub struct SampledTrack<'sample, 'data, T: CurveValue, P: Profile = Universal> {
    /// Effective typed source, without copying its packed key arrays.
    pub track: &'sample Track<'data, T, P>,
    /// Resolved local time used for the sampled value.
    pub time: Time,
    /// Repeating period, absent for nonlooping clocks.
    pub period: Option<Time>,
    /// Absolute origin of a bounded authoring timeline, otherwise zero.
    pub origin: Time,
    /// Inclusive timeline-key selection, absent for clip and shared tracks.
    pub interval: Option<(Time, Time)>,
}

impl<T: CurveValue, P: Profile> Track<'_, T, P> {
    /// Detaches borrowed curve storage for a persistent runtime programme.
    pub fn into_owned(self) -> Track<'static, T, P> {
        Track {
            target: self.target,
            curve: self.curve.into_owned(),
            clock: self.clock,
        }
    }
}
impl<P: Profile> Channel<'_, P> {
    /// Detaches borrowed curve storage without translating property identities.
    pub fn into_owned(self) -> Channel<'static, P> {
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
struct ClipProgram<P: Profile> {
    duration: Time,
    looping: bool,
    channels: IndexedChannels<P>,
}
#[derive(Clone, Debug)]
struct ControllerProgram<P: Profile> {
    priority: i32,
    channels: IndexedChannels<P>,
}
/// Owned semantic animation data, independent of geometry and source asset lifetime.
///
/// Compilation retains property identities and interpolation controls; it does not
/// bake frames, select gameplay controllers, or retain the source model graph.
#[derive(Clone, Debug)]
pub struct AnimationProgram<P: Profile = Universal> {
    clocks: BTreeMap<ClockId, Time>,
    global: IndexedChannels<P>,
    clips: BTreeMap<ClipId, ClipProgram<P>>,
    controllers: BTreeMap<ControllerId, ControllerProgram<P>>,
}
impl<P: Profile> AnimationProgram<P> {
    /// Copies borrowed animation buffers and resolves clip alias programmes.
    pub fn compile(model: &Model<'_, P>) -> Self {
        let own = |channels: &[Channel<'_, P>]| {
            IndexedChannels::new(channels.iter().cloned().map(Channel::into_owned).collect())
        };
        Self {
            clocks: model
                .clocks()
                .map(|(id, clock)| (id, clock.period))
                .collect(),
            global: IndexedChannels::new(
                model
                    .timeline_channels()
                    .iter()
                    .chain(model.shared_channels())
                    .cloned()
                    .map(Channel::into_owned)
                    .collect(),
            ),
            clips: model
                .clips()
                .map(|(id, clip)| {
                    (
                        id,
                        ClipProgram {
                            duration: clip.duration(),
                            looping: clip.is_looping(),
                            channels: own(model
                                .clip_program(id)
                                .expect("validated clip alias")
                                .channels()),
                        },
                    )
                })
                .collect(),
            controllers: model
                .controllers()
                .map(|(id, controller)| {
                    (
                        id,
                        ControllerProgram {
                            priority: controller.priority,
                            channels: own(&controller.channels),
                        },
                    )
                })
                .collect(),
        }
    }
    /// Selects clocks and programmes using the same precedence as `Model::animation_sample`.
    pub fn sample(
        &self,
        sample: AnimationSample<'_>,
    ) -> Result<SampledAnimation<'_, 'static, P>, Error> {
        validate_interval(sample.timeline_interval)?;
        let mut channels = vec![&self.global];
        let mut clip_time = sample.clip_time;
        let mut clip_policy = None;
        if let Some(id) = sample.clip {
            let clip = self
                .clips
                .get(&id)
                .ok_or_else(|| Error::new("animation.sample.clip", "unknown clip identity"))?;
            clip_policy = Some((clip.duration, clip.looping));
            let duration = clip.duration.as_seconds();
            clip_time = Time::seconds(if duration > 0. {
                if clip.looping {
                    clip_time.as_seconds().rem_euclid(duration)
                } else {
                    clip_time.as_seconds().clamp(0., duration)
                }
            } else {
                0.
            })?;
            channels.push(&clip.channels);
        }
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
        for controller in controllers {
            channels.push(&controller.channels);
        }
        Ok(SampledAnimation {
            clocks: Cow::Borrowed(&self.clocks),
            clip_time,
            clip_policy,
            timeline_time: sample.timeline_time,
            timeline_interval: sample.timeline_interval,
            shared_time: sample.shared_time,
            channels: ChannelSelection::Indexed(channels),
        })
    }
}

fn validate_interval(interval: Option<(Time, Time)>) -> Result<(), Error> {
    if interval.is_some_and(|(start, end)| start > end) {
        return Err(Error::new(
            "animation.sample.timeline_interval",
            "interval start exceeds end",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Clip, Controller, Curve, ModelBuilder, Node, SharedClock};
    use std::ptr::eq;
    fn time(seconds: f64) -> Time {
        Time::seconds(seconds).unwrap()
    }
    fn channel(property: Property<[f32; 3]>, clock: Clock, end: f32) -> Channel<'static> {
        Channel::Vector3(Track {
            target: property,
            curve: Curve::linear([time(0.), time(2.)], [[0.; 3], [end; 3]]).unwrap(),
            clock,
        })
    }
    #[test]
    fn event_boundary_override_keeps_own_clock_end_and_other_clock_start() {
        let mut builder = ModelBuilder::new("occurrence");
        let node = builder.add_node(Node::named("bone")).unwrap();
        let clock = builder.add_clock(SharedClock { period: time(2.) }).unwrap();
        let clip = builder
            .add_clip(Clip::named("loop").with_duration(time(2.)))
            .unwrap();
        builder
            .add_channel(clip, channel(node.translation(), Clock::Clip, 4.))
            .unwrap();
        builder
            .add_shared_channel(channel(node.scale(), Clock::Shared(clock), 8.))
            .unwrap();
        let model = builder.finish().unwrap();
        let sampled = model
            .animation_sample(AnimationSample {
                clip: Some(clip),
                ..Default::default()
            })
            .unwrap();
        let end = sampled
            .at_times(time(2.), time(2.), time(2.))
            .at_occurrence(Clock::Clip, time(2.));
        assert_eq!(end.property(&node.translation()), Some([4.; 3]));
        assert_eq!(end.property(&node.scale()), Some([0.; 3]));
        assert_eq!(end.track(&node.translation()).unwrap().time, time(2.));
        let shared_end = sampled
            .at_times(time(2.), time(2.), time(2.))
            .at_occurrence(Clock::Shared(clock), time(2.));
        assert_eq!(shared_end.property(&node.translation()), Some([0.; 3]));
        assert_eq!(shared_end.property(&node.scale()), Some([8.; 3]));
    }
    #[test]
    fn alias_local_clock_and_owned_program_survive_source_drop() {
        let mut builder = ModelBuilder::new("sample");
        let node = builder.add_node(Node::named("bone")).unwrap();
        let property = node.translation();
        let original = builder
            .add_clip(Clip::named("original").with_duration(time(2.)))
            .unwrap();
        builder
            .add_channel(original, channel(property.clone(), Clock::Clip, 4.))
            .unwrap();
        let alias = builder
            .add_clip(
                Clip::named("alias")
                    .with_alias(original)
                    .with_duration(time(1.))
                    .once(),
            )
            .unwrap();
        let model = builder.finish().unwrap();
        let sample = AnimationSample {
            clip: Some(alias),
            clip_time: time(8.),
            ..Default::default()
        };
        assert_eq!(
            model.animation_sample(sample).unwrap().property(&property),
            Some([2.; 3])
        );
        let program = AnimationProgram::compile(&model);
        drop(model);
        assert_eq!(
            program.sample(sample).unwrap().property(&property),
            Some([2.; 3])
        );
    }
    #[test]
    fn independent_clocks_and_controller_priority_are_explicit() {
        let mut builder = ModelBuilder::new("sample");
        let node = builder.add_node(Node::named("bone")).unwrap();
        let translation = node.translation();
        let scale = node.scale();
        let clock = builder.add_clock(SharedClock { period: time(2.) }).unwrap();
        builder
            .add_timeline_channel(channel(translation.clone(), Clock::Timeline, 4.))
            .unwrap();
        builder
            .add_shared_channel(channel(scale.clone(), Clock::Shared(clock), 8.))
            .unwrap();
        let low = builder
            .add_controller(Controller {
                priority: -1,
                channels: vec![channel(translation.clone(), Clock::Timeline, 10.)],
                ..Default::default()
            })
            .unwrap();
        let high = builder
            .add_controller(Controller {
                priority: 2,
                channels: vec![channel(translation.clone(), Clock::Timeline, 20.)],
                ..Default::default()
            })
            .unwrap();
        let model = builder.finish().unwrap();
        let sample = AnimationSample {
            timeline_time: time(1.),
            shared_time: time(2.5),
            controllers: &[high, low],
            ..Default::default()
        };
        let selected = model.animation_sample(sample).unwrap();
        assert_eq!(selected.property(&translation), Some([10.; 3]));
        assert_eq!(selected.property(&scale), Some([2.; 3]));
        assert_eq!(selected.animated(&Animated::constant(7f32)), 7.);
        assert!(model
            .animation_sample(AnimationSample {
                controllers: &[low, low],
                ..sample
            })
            .is_err());
        let program = AnimationProgram::compile(&model);
        assert_eq!(
            program.sample(sample).unwrap().property(&translation),
            selected.property(&translation)
        );
    }
    #[test]
    fn indexed_empty_override_falls_back_and_timeline_keys_are_bounded() {
        let mut builder = ModelBuilder::new("bounded");
        let node = builder.add_node(Node::named("bone")).unwrap();
        let property = node.translation();
        builder
            .add_timeline_channel(Channel::Vector3(Track {
                target: property.clone(),
                clock: Clock::Timeline,
                curve: Curve::linear(
                    [time(0.), time(1.), time(4.)],
                    [[0.; 3], [2.; 3], [100.; 3]],
                )
                .unwrap(),
            }))
            .unwrap();
        let controller = builder
            .add_controller(Controller {
                channels: vec![Channel::Vector3(Track {
                    target: property.clone(),
                    clock: Clock::Timeline,
                    curve: Curve::step(Vec::<Time>::new(), Vec::<[f32; 3]>::new()).unwrap(),
                })],
                ..Default::default()
            })
            .unwrap();
        let model = builder.finish().unwrap();
        let program = AnimationProgram::compile(&model);
        let sample = AnimationSample {
            timeline_time: time(2.),
            timeline_interval: Some((time(0.), time(2.))),
            controllers: &[controller],
            ..Default::default()
        };
        assert_eq!(
            program.sample(sample).unwrap().property(&property),
            Some([2.; 3])
        );
        assert_eq!(
            model.animation_sample(sample).unwrap().property(&property),
            Some([2.; 3])
        );
        assert!(program
            .sample(AnimationSample {
                timeline_interval: Some((time(2.), time(1.))),
                ..sample
            })
            .is_err());
    }
    #[test]
    fn retiming_borrows_indexed_programmes_and_preserves_alias_clock_policy() {
        let mut builder = ModelBuilder::new("retime");
        let node = builder.add_node(Node::named("node")).unwrap();
        let property = node.translation();
        let base = builder
            .add_clip(Clip::named("base").with_duration(time(2.)))
            .unwrap();
        builder
            .add_channel(base, channel(property.clone(), Clock::Clip, 4.))
            .unwrap();
        let alias = builder
            .add_clip(
                Clip::named("alias")
                    .with_alias(base)
                    .with_duration(time(1.)),
            )
            .unwrap();
        let model = builder.finish().unwrap();
        let program = AnimationProgram::compile(&model);
        let selected = program
            .sample(AnimationSample {
                clip: Some(alias),
                clip_time: time(0.1),
                ..Default::default()
            })
            .unwrap();
        let original = selected.track(&property).unwrap();
        let retimed = selected.at_times(time(2.25), time(100.), time(100.));
        assert!(eq(retimed.selection, &selected));
        assert_eq!(retimed.property(&property), Some([0.5; 3]));
        let effective = retimed.track(&property).unwrap();
        assert!(eq(original.track, effective.track));
        assert_eq!(effective.time, time(0.25));
        assert_eq!(effective.period, Some(time(1.)));
        assert_eq!(selected.property(&property), Some([0.2; 3]));
    }
    #[test]
    fn retimed_timeline_and_shared_clocks_keep_offsets_intervals_and_fallback() {
        let mut builder = ModelBuilder::new("offsets");
        let node = builder.add_node(Node::named("node")).unwrap();
        let translation = node.translation();
        let scale = node.scale();
        let period = builder.add_clock(SharedClock { period: time(2.) }).unwrap();
        builder
            .add_timeline_channel(Channel::Vector3(Track {
                target: translation.clone(),
                clock: Clock::Timeline,
                curve: Curve::linear(
                    [time(9.), time(10.), time(12.), time(13.)],
                    [[90.; 3], [2.; 3], [6.; 3], [130.; 3]],
                )
                .unwrap(),
            }))
            .unwrap();
        builder
            .add_shared_channel(channel(scale.clone(), Clock::Shared(period), 8.))
            .unwrap();
        let empty = builder
            .add_controller(Controller {
                channels: vec![Channel::Vector3(Track {
                    target: translation.clone(),
                    clock: Clock::Timeline,
                    curve: Curve::step(Vec::<Time>::new(), Vec::<[f32; 3]>::new()).unwrap(),
                })],
                ..Default::default()
            })
            .unwrap();
        let clip = builder
            .add_clip(Clip::named("interval").with_duration(time(2.)))
            .unwrap();
        let model = builder.finish().unwrap();
        let selected = model
            .animation_sample(AnimationSample {
                clip: Some(clip),
                timeline_time: time(10.5),
                timeline_interval: Some((time(10.), time(12.))),
                controllers: &[empty],
                ..Default::default()
            })
            .unwrap();
        let retimed = selected.at_times(time(3.), time(11.5), time(5.));
        assert_eq!(retimed.property(&translation), Some([5.; 3]));
        assert_eq!(retimed.property(&scale), Some([4.; 3]));
        let track = retimed.track(&translation).unwrap();
        assert_eq!(track.origin, time(10.));
        assert_eq!(track.period, Some(time(2.)));
        assert_eq!(track.interval, Some((time(10.), time(12.))));
        assert_eq!(track.track.curve.values().len(), 4);
        let shared = retimed.track(&scale).unwrap();
        assert_eq!(shared.time, time(1.));
        assert_eq!(shared.period, Some(time(2.)));
        assert_eq!(shared.origin, Time::default());
    }
    #[test]
    #[ignore = "explicit animation throughput measurement"]
    fn compiled_property_sampling_throughput() {
        use std::hint::black_box;
        use std::time::Instant;
        for node_count in [128, 1024, 4096] {
            let mut builder = ModelBuilder::new("throughput");
            let mut properties = Vec::with_capacity(node_count);
            for _ in 0..node_count {
                let node = builder.add_node(Node::named("bone")).unwrap();
                let property = node.translation();
                builder
                    .add_timeline_channel(channel(property.clone(), Clock::Timeline, 4.))
                    .unwrap();
                properties.push(property);
            }
            let model = builder.finish().unwrap();
            let program = AnimationProgram::compile(&model);
            let sampled = program
                .sample(AnimationSample {
                    timeline_time: time(1.),
                    ..Default::default()
                })
                .unwrap();
            let start = Instant::now();
            let mut checksum = 0.;
            let rounds = 100;
            for _ in 0..rounds {
                for property in &properties {
                    checksum += black_box(sampled.property(black_box(property)).unwrap()[0]);
                }
            }
            let count = rounds * node_count;
            eprintln!(
                "{node_count} animated nodes: {:.1} ns/property",
                start.elapsed().as_nanos() as f64 / count as f64
            );
            assert_eq!(checksum, (count * 2) as f32);
        }
    }
}
