//! Explicit playback selection and typed evaluation of semantic animation.
use super::curves::{CurveValue, SampleValue};
use crate::model::error::{AnimationError, Location};
use crate::model::profiles::Profile;
use crate::model::{
    AnimationId, ClipId, ClockId, DynamicTrack, Error, Model, Property, PropertyValue,
    PropertyValues, Quat, Texture, Time, TimeSource, Track, ValueSlot,
};
use glam::{Vec2, Vec3, Vec4};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Range;

/// TimeSource values and explicitly selected programmes for one evaluation.
///
/// The selected clip maps local time into its authored source interval. Without
/// a clip, ordinary tracks use `clip_time` directly without interval bounds.
/// Independent shared clocks wrap `shared_time` by their authored periods; frozen
/// clocks retain time zero.
/// This operation does not dispatch events.
#[derive(Clone, Copy, Debug, Default)]
pub struct AnimationSample {
    /// Selected playback clip.
    pub clip: Option<ClipId>,
    /// Seconds from the beginning of the selected clip; wrapped or clamped.
    pub clip_time: Time,
    /// Time driving all independent repeating clocks.
    pub shared_time: Time,
}

/// A validated playback selection that can evaluate many typed properties.
///
/// Model-level animations precede selected clip animations. Later matching tracks
/// override earlier tracks.
pub struct SampledAnimation<'view, 'data> {
    values: &'view PropertyValues,
    clocks: Cow<'view, BTreeMap<ClockId, Time>>,
    clip_policy: Option<(Time, bool)>,
    animation_time: Time,
    source_interval: Option<(Time, Time)>,
    shared_time: Time,
    tracks: TrackSelection<'view, 'data>,
}

enum TrackSelection<'view, 'data> {
    Flat(Vec<&'view DynamicTrack<'data>>),
    Indexed(IndexedSelection<'view>),
}
struct IndexedSelection<'view> {
    tracks: &'view [DynamicTrack<'static>],
    global: &'view TrackIndex,
    clip: Option<&'view TrackIndex>,
}
impl<'view> IndexedSelection<'view> {
    fn matching(
        &self,
        slot: ValueSlot,
    ) -> impl DoubleEndedIterator<Item = &'view DynamicTrack<'static>> + '_ {
        self.global
            .get(&slot)
            .into_iter()
            .chain(self.clip.and_then(|index| index.get(&slot)))
            .flatten()
            .map(|&index| &self.tracks[index])
    }
}
type TrackIndex = HashMap<ValueSlot, Vec<usize>>;

/// A value represented by a typed semantic animation track.
pub trait TrackValue: SampleValue + PropertyValue {
    /// Borrows the matching typed track when this track carries this value type.
    fn typed_track<'view, 'data>(
        dynamic_track: &'view DynamicTrack<'data>,
    ) -> Option<&'view Track<'data, Self>>;
}
macro_rules! track_value {
    ($($ty:ty => $variant:ident),* $(,)?) => {$ (
        impl TrackValue for $ty {
            fn typed_track<'view, 'data>(dynamic_track: &'view DynamicTrack<'data>) -> Option<&'view Track<'data, Self>> {
                if let DynamicTrack::$variant(track) = dynamic_track { Some(track) } else { None }
            }
        }
    )* };
}
track_value! {
    [u32; 4] => Unsigned4, [u8; 4] => Bytes4, Quat => Rotation,
    f32 => Scalar, Vec2 => Vector2, Vec3 => Vector3,
    Vec4 => Vector4, u32 => Unsigned, i32 => Signed,
    u16 => Unsigned16, u8 => Byte, bool => Boolean,
    Texture => Texture, Option<Texture> => OptionalTexture,
}

impl<'data, P: Profile> Model<'data, P> {
    /// Resolves playback identities once before evaluating animated properties.
    pub fn animation_sample<'view>(
        &'view self,
        sample: AnimationSample,
    ) -> Result<SampledAnimation<'view, 'data>, Error> {
        let mut tracks: Vec<_> = self.global_tracks().collect();
        let (clip_policy, animation_time, source_interval) = if let Some(id) = sample.clip {
            let clip = self.clip(id).ok_or_else(|| {
                Error::at(
                    Location::new().field("clip"),
                    AnimationError::UnknownClip { clip: id },
                )
            })?;
            tracks.extend(self.clip_tracks(id));
            resolve_times(
                sample,
                clip.duration(),
                clip.is_looping(),
                clip.source_start,
            )?
        } else {
            (None, sample.clip_time, None)
        };
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
            tracks: TrackSelection::Flat(tracks),
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

    /// Evaluates the selected tracks for a typed property, or returns no override.
    pub fn override_value<T: TrackValue>(&self, property: &Property<T>) -> Option<T> {
        self.override_at(property, self.times())
    }
    fn override_at<T: TrackValue>(&self, property: &Property<T>, times: SampleTimes) -> Option<T> {
        match &self.tracks {
            TrackSelection::Flat(tracks) => tracks
                .iter()
                .rev()
                .find_map(|dynamic_track| self.sample_track(dynamic_track, property, times)),
            TrackSelection::Indexed(selection) => selection
                .matching(property.slot())
                .rev()
                .find_map(|track| self.sample_track(track, property, times)),
        }
    }
    fn sample_track<T: TrackValue>(
        &self,
        dynamic_track: &DynamicTrack<'_>,
        property: &Property<T>,
        times: SampleTimes,
    ) -> Option<T> {
        let track = T::typed_track(dynamic_track)?;
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
    pub fn track<T: TrackValue>(
        &self,
        property: &Property<T>,
    ) -> Option<SampledTrack<'_, 'data, T>> {
        self.track_at(property, self.times())
    }
    fn track_at<T: TrackValue>(
        &self,
        property: &Property<T>,
        times: SampleTimes,
    ) -> Option<SampledTrack<'_, 'data, T>> {
        let resolve = |dynamic_track: &'view DynamicTrack<'data>| {
            self.sample_track(dynamic_track, property, times)?;
            let track = T::typed_track(dynamic_track)?;
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
        match &self.tracks {
            TrackSelection::Flat(tracks) => tracks
                .iter()
                .rev()
                .find_map(|dynamic_track| resolve(dynamic_track)),
            TrackSelection::Indexed(selection) => selection
                .matching(property.slot())
                .rev()
                .find_map(|track| resolve(track)),
        }
    }
    /// Evaluates an animated value, preserving its authored default without a track.
    pub fn animated<T: TrackValue>(&self, value: &Property<T>) -> T {
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
    pub fn override_value<T: TrackValue>(&self, property: &Property<T>) -> Option<T> {
        self.selection.override_at(property, self.times)
    }
    /// Preserves an authored default when no selected track supplies a value.
    pub fn animated<T: TrackValue>(&self, value: &Property<T>) -> T {
        self.override_value(value)
            .unwrap_or_else(|| T::resolve(self.selection.values, *value, self))
    }
    /// Borrows the effective source track at the overridden clock values.
    pub fn track<T: TrackValue>(&self, property: &Property<T>) -> Option<SampledTrack<'_, '_, T>> {
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
impl DynamicTrack<'_> {
    /// Detaches borrowed curve storage without translating property identities.
    pub fn into_owned(self) -> DynamicTrack<'static> {
        macro_rules! owned { ($($variant:ident),*) => { match self { $(Self::$variant(track) => DynamicTrack::$variant(track.into_owned()),)* } }; }
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
    index: TrackIndex,
}
/// Owned semantic animation data, independent of geometry and source asset lifetime.
///
/// Compilation retains property identities and interpolation controls; it does not
/// bake frames or retain the source model graph.
#[derive(Clone, Debug)]
pub struct AnimationProgram {
    values: PropertyValues,
    clocks: BTreeMap<ClockId, Time>,
    tracks: Vec<DynamicTrack<'static>>,
    animations: BTreeMap<AnimationId, Range<usize>>,
    global: Vec<AnimationId>,
    global_index: TrackIndex,
    clips: BTreeMap<ClipId, ClipProgram>,
}
impl AnimationProgram {
    /// Authored constants retained by this compiled program.
    pub fn values(&self) -> &PropertyValues {
        &self.values
    }
    /// Copies each reusable animation once and indexes candidates by playback selection.
    /// Clip aliases share curve storage; each property read visits only its selected writers.
    pub fn compile<P: Profile>(model: &Model<'_, P>) -> Self {
        let mut tracks = Vec::new();
        let animations = model
            .animations()
            .iter()
            .map(|(id, animation)| {
                let start = tracks.len();
                tracks.extend(
                    animation
                        .tracks
                        .iter()
                        .cloned()
                        .map(DynamicTrack::into_owned),
                );
                (id, start..tracks.len())
            })
            .collect();
        let mut program = Self {
            values: model.values().clone(),
            clocks: model
                .clocks()
                .iter()
                .map(|(id, clock)| (id, clock.period()))
                .collect(),
            tracks,
            animations,
            global: model.global_animations().to_vec(),
            global_index: TrackIndex::new(),
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
                            index: TrackIndex::new(),
                        },
                    )
                })
                .collect(),
        };
        program.index_selections();
        program
    }
    fn index_selections(&mut self) {
        let index = |selection: &[AnimationId]| {
            let mut index = TrackIndex::new();
            for id in selection {
                for ordinal in self.animations[id].clone() {
                    index
                        .entry(self.tracks[ordinal].target())
                        .or_default()
                        .push(ordinal);
                }
            }
            index
        };
        self.global_index = index(&self.global);
        for clip in self.clips.values_mut() {
            clip.index = index(&clip.animations);
        }
    }
    /// Combines disjoint property owners on the receiver's playback clocks.
    ///
    /// `clips` pairs destination clip identities with source clips. Both use the
    /// destination's time in seconds; curves are not stretched. Source global and
    /// shared-clock tracks remain active. `include` can exclude properties whose
    /// ownership is supplied by another rig. No source clip becomes independently
    /// selectable. Mapped clips must have matching source starts; a failed
    /// combination leaves both inputs unchanged.
    pub fn compose(
        &self,
        source: &Self,
        clips: &[(ClipId, ClipId)],
        mut include: impl FnMut(&DynamicTrack<'static>) -> bool,
    ) -> Result<Self, Error> {
        let mut result = self.clone();
        result.values.extend(&source.values)?;
        let mut destinations = HashSet::new();
        for (mapping, &(destination, source_clip)) in clips.iter().enumerate() {
            if !destinations.insert(destination) {
                return Err(Error::at(
                    Location::new().field("clips").index(mapping),
                    AnimationError::DuplicateDestinationClip { clip: destination },
                ));
            }
            if !self.clips.contains_key(&destination) {
                return Err(Error::at(
                    Location::new()
                        .field("clips")
                        .index(mapping)
                        .field("destination"),
                    AnimationError::UnknownClip { clip: destination },
                ));
            }
            if !source.clips.contains_key(&source_clip) {
                return Err(Error::at(
                    Location::new()
                        .field("clips")
                        .index(mapping)
                        .field("source"),
                    AnimationError::UnknownClip { clip: source_clip },
                ));
            }
            if self.clips[&destination].source_start != source.clips[&source_clip].source_start {
                return Err(Error::at(
                    Location::new().field("clips").index(mapping),
                    AnimationError::ClipStartMismatch {
                        destination,
                        source: source_clip,
                        destination_start: self.clips[&destination].source_start,
                        source_start: source.clips[&source_clip].source_start,
                    },
                ));
            }
        }
        for (&id, &period) in &source.clocks {
            if result
                .clocks
                .get(&id)
                .is_some_and(|existing| *existing != period)
            {
                return Err(Error::at(
                    Location::for_owner(id).field("period"),
                    AnimationError::ClockPeriodConflict {
                        clock: id,
                        existing: result.clocks[&id],
                        incoming: period,
                    },
                ));
            }
            result.clocks.insert(id, period);
        }
        let owned: HashSet<_> = self.tracks.iter().map(DynamicTrack::target).collect();
        for (&id, range) in &source.animations {
            if result.animations.contains_key(&id) {
                return Err(Error::at(
                    Location::for_owner(id),
                    AnimationError::AnimationIdentityConflict { animation: id },
                ));
            }
            let start = result.tracks.len();
            for dynamic_track in &source.tracks[range.clone()] {
                if !include(dynamic_track) {
                    continue;
                }
                if owned.contains(&dynamic_track.target()) {
                    return Err(Error::at(
                        Location::for_owner(dynamic_track.target()),
                        AnimationError::PropertyOwnershipConflict {
                            slot: dynamic_track.target(),
                        },
                    ));
                }
                result.tracks.push(dynamic_track.clone());
            }
            result.animations.insert(id, start..result.tracks.len());
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
        result.index_selections();
        Ok(result)
    }
    /// Selects clocks and programmes using the same precedence as `Model::animation_sample`.
    pub fn sample(&self, sample: AnimationSample) -> Result<SampledAnimation<'_, 'static>, Error> {
        let mut selected_clip = None;
        let (clip_policy, animation_time, source_interval) = if let Some(id) = sample.clip {
            let clip = self.clips.get(&id).ok_or_else(|| {
                Error::at(
                    Location::new().field("clip"),
                    AnimationError::UnknownClip { clip: id },
                )
            })?;
            selected_clip = Some(&clip.index);
            resolve_times(sample, clip.duration, clip.looping, clip.source_start)?
        } else {
            (None, sample.clip_time, None)
        };
        Ok(SampledAnimation {
            values: &self.values,
            clocks: Cow::Borrowed(&self.clocks),
            clip_policy,
            animation_time,
            source_interval,
            shared_time: sample.shared_time,
            tracks: TrackSelection::Indexed(IndexedSelection {
                tracks: &self.tracks,
                global: &self.global_index,
                clip: selected_clip,
            }),
        })
    }
}

type ResolvedTimes = (Option<(Time, bool)>, Time, Option<(Time, Time)>);
fn resolve_times(
    sample: AnimationSample,
    duration: Time,
    looping: bool,
    origin: Time,
) -> Result<ResolvedTimes, Error> {
    let policy = Some((duration, looping));
    let clip = normalize_clip(sample.clip_time, policy);
    let animation = Time::seconds(origin.as_seconds() + clip.as_seconds())
        .map_err(|error| error.with_context(Location::new().field("clip_time")))?;
    let interval = Some((
        origin,
        Time::seconds(origin.as_seconds() + duration.as_seconds())
            .map_err(|error| error.with_context(Location::new().field("duration")))?,
    ));
    Ok((policy, animation, interval))
}

/// Typed slot reads shared by ordinary and occurrence-time animation samples.
/// Consumers can evaluate poses and surfaces without changing the selected programmes.
pub trait ValueSample {
    /// Authored constants for the selected model.
    fn values(&self) -> &PropertyValues;
    /// Samples an override, returning `None` when no selected track writes it.
    fn override_value<T: TrackValue>(&self, property: &Property<T>) -> Option<T>;
    /// Reads a slot, using its authored constant when no selected writer supplies it.
    ///
    /// Panics if the slot is absent from this model. Use `values().get(slot)` to
    /// check an identity before reading an externally supplied reference.
    fn animated<T: TrackValue>(&self, value: &Property<T>) -> T {
        self.override_value(value)
            .unwrap_or_else(|| T::resolve(self.values(), *value, self))
    }
}
impl ValueSample for SampledAnimation<'_, '_> {
    fn values(&self) -> &PropertyValues {
        self.values
    }
    fn override_value<T: TrackValue>(&self, property: &Property<T>) -> Option<T> {
        self.override_value(property)
    }
}
impl ValueSample for RetimedAnimation<'_, '_, '_> {
    fn values(&self) -> &PropertyValues {
        self.selection.values
    }
    fn override_value<T: TrackValue>(&self, property: &Property<T>) -> Option<T> {
        self.override_value(property)
    }
}

impl ValueSample for PropertyValues {
    fn values(&self) -> &PropertyValues {
        self
    }
    fn override_value<T: TrackValue>(&self, slot: &Property<T>) -> Option<T> {
        self.get(*slot).copied()
    }
}

#[cfg(test)]
#[path = "sampling_tests.rs"]
mod tests;
