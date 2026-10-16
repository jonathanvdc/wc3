//! Owned semantic event programmes and bounded forward-clock traversal.
use bevy::prelude::*;
use std::borrow::Cow;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::iter::once;
use std::sync::Arc;
use wc3::model::controls::playback::EventCommand;
use wc3::model::{
    AnimationSample, ClipControls, ClipId, Clock, ClockId, ControllerId, Event,
    EventSource as SemanticEventSource, FeatureId, Model, NodeFeature, NodeId,
};

/// Authored event semantics; applications interpret identities and commands.
#[derive(Clone, Debug)]
pub enum EventSource {
    /// Named action, including names whose prefixes have game-specific meaning.
    Named(String),
    /// Four-byte action code with its independent game-data identity.
    Identifier {
        /// Action code bytes.
        identifier: [u8; 4],
        /// External game-data identity.
        data: u32,
    },
    /// Authored command and typed optional parameters.
    Command(EventCommand<'static>),
}
/// Programme that owns an event, separate from the clock scheduling its key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventProgram {
    /// Global authoring or shared-clock programme.
    Global,
    /// Selected clip, including the selected alias identity.
    Clip(ClipId),
    /// Explicitly enabled controller programme.
    Controller(ControllerId),
}
/// One crossed semantic event with an occurrence-time world pose.
#[derive(Message, Clone, Debug)]
pub struct ModelEvent {
    /// Independently animated model root.
    pub root: Entity,
    /// Rig node entity, when the occurrence has a hierarchy anchor.
    pub node: Option<Entity>,
    /// Source node identity, when authored.
    pub node_id: Option<NodeId>,
    /// Source event-feature identity, when authored.
    pub feature: Option<FeatureId>,
    /// Authored event name.
    pub name: Arc<str>,
    /// Typed feature semantics; an occurrence without a feature has no source.
    pub source: Option<Arc<EventSource>>,
    /// Logical event payload bytes.
    pub payload: Arc<[u8]>,
    /// Programme owning this event occurrence.
    pub program: EventProgram,
    /// Clock on which this key is authored.
    pub clock: Clock,
    /// Exact authored key in seconds, including inclusive period endpoints.
    pub key_time: f64,
    /// Unwrapped occurrence time on its driving clock, in seconds.
    pub occurrence_time: f64,
    /// Pose at the occurrence; external transforms use current update values.
    pub transform: GlobalTransform,
}

/// Per-instance event dispatch accounting for the most recent update.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct EventDispatchStats {
    /// Number of emitted notifications, at most 4096 per update.
    pub emitted: usize,
    /// Notifications omitted after the bounded earliest-occurrence budget.
    pub dropped: u64,
}

#[derive(Clone)]
pub(super) struct EventDefinition {
    pub feature: Option<FeatureId>,
    pub node: Option<NodeId>,
    pub node_index: Option<usize>,
    pub name: Arc<str>,
    pub source: Option<Arc<EventSource>>,
    pub payload: Arc<[u8]>,
    pub program: EventProgram,
    pub clock: Clock,
    pub time: f64,
}
#[derive(Clone, Copy)]
struct ClipPolicy {
    duration: f64,
    looping: bool,
    origin: Option<f64>,
}
#[derive(Clone, Default)]
struct ProgramEvents {
    clip: Vec<usize>,
    timeline: Vec<usize>,
    shared: HashMap<ClockId, Vec<usize>>,
}
impl ProgramEvents {
    fn groups(&self) -> impl Iterator<Item = (Clock, &[usize])> {
        [
            (Clock::Clip, self.clip.as_slice()),
            (Clock::Timeline, self.timeline.as_slice()),
        ]
        .into_iter()
        .chain(
            self.shared
                .iter()
                .map(|(id, indices)| (Clock::Shared(*id), indices.as_slice())),
        )
    }
}

#[derive(Clone, Default)]
pub(super) struct RuntimeEvents {
    pub definitions: Vec<EventDefinition>,
    programs: HashMap<EventProgram, ProgramEvents>,
    clips: HashMap<ClipId, ClipPolicy>,
    clocks: HashMap<ClockId, f64>,
}
impl RuntimeEvents {
    fn indexed(mut self) -> Self {
        self.programs.clear();
        for (index, definition) in self.definitions.iter().enumerate() {
            let program = self.programs.entry(definition.program).or_default();
            match definition.clock {
                Clock::Clip => program.clip.push(index),
                Clock::Timeline => program.timeline.push(index),
                Clock::Shared(id) => program.shared.entry(id).or_default().push(index),
            }
        }
        for program in self.programs.values_mut() {
            for indices in [&mut program.clip, &mut program.timeline]
                .into_iter()
                .chain(program.shared.values_mut())
            {
                indices.sort_by(|a, b| {
                    self.definitions[*a]
                        .time
                        .total_cmp(&self.definitions[*b].time)
                        .then_with(|| a.cmp(b))
                });
            }
        }
        self
    }
    pub(crate) fn append_remapped(
        &mut self,
        source: &Self,
        indices: &[usize],
        nodes: &HashMap<NodeId, NodeId>,
        clips: &HashMap<ClipId, ClipId>,
    ) {
        for definition in &source.definitions {
            let destinations: Vec<_> = match definition.program {
                EventProgram::Clip(source) => clips
                    .iter()
                    .filter_map(|(&host, &part)| {
                        (source == part).then_some(EventProgram::Clip(host))
                    })
                    .collect(),
                program => vec![program],
            };
            for program in destinations {
                let mut definition = definition.clone();
                definition.program = program;
                definition.node = definition.node.map(|id| nodes[&id]);
                definition.node_index = definition.node_index.map(|i| indices[i]);
                self.definitions.push(definition);
            }
        }
        self.clocks
            .extend(source.clocks.iter().map(|(&id, &period)| (id, period)));
        *self = self.clone().indexed();
    }
    pub fn compile(model: &Model<'_>) -> Self {
        let node_indices: HashMap<_, _> = model
            .nodes()
            .enumerate()
            .map(|(index, (id, _))| (id, index))
            .collect();
        let sources: HashMap<_, _> = model
            .features()
            .filter_map(|(id, feature)| {
                let NodeFeature::Event(source) = &feature.payload else {
                    return None;
                };
                let source = match source {
                    SemanticEventSource::Named(value) => EventSource::Named(value.name.to_string()),
                    SemanticEventSource::Identifier(value) => EventSource::Identifier {
                        identifier: value.identifier,
                        data: value.data,
                    },
                    SemanticEventSource::Command(value) => EventSource::Command(EventCommand {
                        event_type: value.event_type,
                        transform: value.transform,
                        options: Cow::Owned(value.options.to_string()),
                        render_target_channel: value.render_target_channel,
                        parameter: value.parameter,
                        control: value.control,
                    }),
                };
                Some((id, (feature.node, Arc::new(source))))
            })
            .collect();
        let mut result = Self {
            definitions: Vec::new(),
            programs: HashMap::new(),
            clocks: model
                .clocks()
                .map(|(id, value)| (id, value.period().as_seconds()))
                .collect(),
            clips: model
                .clips()
                .map(|(id, value)| {
                    let origin = match &value.controls {
                        Some(ClipControls::Mdlx(value)) => Some(value.start.as_seconds()),
                        Some(ClipControls::M3(value)) => Some(value.timeline_origin.as_seconds()),
                        _ => None,
                    };
                    (
                        id,
                        ClipPolicy {
                            duration: value.duration().as_seconds(),
                            looping: value.is_looping(),
                            origin,
                        },
                    )
                })
                .collect(),
        };
        let mut add = |event: &Event<'_>, program| {
            let source = event.feature.and_then(|id| sources.get(&id));
            let node = event.node.or(source.map(|(node, _)| *node));
            result.definitions.push(EventDefinition {
                feature: event.feature,
                node,
                node_index: node.and_then(|id| node_indices.get(&id).copied()),
                name: Arc::from(event.name.as_ref()),
                source: source.map(|(_, value)| value.clone()),
                payload: Arc::from(event.payload.iter().collect::<Vec<_>>()),
                program,
                clock: event.clock,
                time: event.time.as_seconds(),
            });
        };
        for event in model.timeline_events().iter().chain(model.shared_events()) {
            add(event, EventProgram::Global);
        }
        for (id, _) in model.clips() {
            for event in model
                .clip_program(id)
                .expect("validated alias programme")
                .events()
            {
                add(event, EventProgram::Clip(id));
            }
        }
        for (id, controller) in model.controllers() {
            for event in controller.events.values() {
                add(event, EventProgram::Controller(id));
            }
        }
        result.indexed()
    }
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}

/// Raw instance clocks retained independently of sample-time wrapping.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct EventClocks {
    pub clip: f64,
    pub timeline: f64,
    pub shared: f64,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct EventTraversal {
    pub revision: u64,
    pub origin: EventClocks,
    pub include_origin: bool,
}
impl Default for EventTraversal {
    fn default() -> Self {
        Self {
            revision: 0,
            origin: EventClocks::default(),
            include_origin: true,
        }
    }
}
impl EventClocks {
    fn interpolate(self, next: Self, fraction: f64) -> Self {
        Self {
            clip: self.clip + (next.clip - self.clip) * fraction,
            timeline: self.timeline + (next.timeline - self.timeline) * fraction,
            shared: self.shared + (next.shared - self.shared) * fraction,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct EventOccurrence {
    pub definition: usize,
    pub occurrence_time: f64,
    pub clocks: EventClocks,
    fraction: f64,
    cycle: f64,
}
impl PartialEq for EventOccurrence {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for EventOccurrence {}
impl PartialOrd for EventOccurrence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for EventOccurrence {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fraction
            .total_cmp(&other.fraction)
            .then(self.definition.cmp(&other.definition))
            .then(self.cycle.total_cmp(&other.cycle))
    }
}

#[derive(Clone, Copy)]
struct EventStream {
    occurrence: EventOccurrence,
    key: f64,
    period: f64,
    last: f64,
    from: f64,
    to: f64,
}
impl PartialEq for EventStream {
    fn eq(&self, other: &Self) -> bool {
        self.occurrence == other.occurrence
    }
}
impl Eq for EventStream {}
impl PartialOrd for EventStream {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for EventStream {
    fn cmp(&self, other: &Self) -> Ordering {
        self.occurrence.cmp(&other.occurrence)
    }
}
impl EventStream {
    fn advance(&mut self, previous: EventClocks, current: EventClocks) -> bool {
        if self.occurrence.cycle >= self.last {
            return false;
        }
        self.occurrence.cycle += 1.;
        self.occurrence.occurrence_time = self.key + self.occurrence.cycle * self.period;
        self.occurrence.fraction = if self.to > self.from {
            (self.occurrence.occurrence_time - self.from) / (self.to - self.from)
        } else {
            0.
        };
        self.occurrence.clocks = previous.interpolate(current, self.occurrence.fraction);
        true
    }
}
/// Per-instance traversal state. Revisions distinguish explicit seeks/restarts
/// from ordinary forward clock movement without inferring intent from delta size.
#[derive(Default)]
pub(super) struct EventState {
    previous: Option<EventClocks>,
    revision: u64,
    clip: Option<ClipId>,
    controllers: Vec<ControllerId>,
    include_origin: bool,
    pub occurrences: Vec<EventOccurrence>,
    heap: BinaryHeap<Reverse<EventStream>>,
    #[cfg(test)]
    generated: usize,
    #[cfg(test)]
    scanned: usize,
    /// Number of omitted notifications in the most recent bounded traversal.
    pub dropped: u64,
}
impl EventState {
    /// Reset traversal explicitly. Restart traverses from zero inclusively;
    /// seeking installs the destination as an exclusive next traversal origin.
    #[cfg(test)]
    fn reset(&mut self, clocks: EventClocks, revision: u64, include_origin: bool) {
        self.previous = Some(clocks);
        self.revision = revision;
        self.include_origin = include_origin;
    }
    pub fn collect(
        &mut self,
        events: &RuntimeEvents,
        selection: AnimationSample<'_>,
        current: EventClocks,
        traversal: EventTraversal,
        paused: bool,
        budget: usize,
    ) {
        self.occurrences.clear();
        self.heap.clear();
        self.dropped = 0;
        #[cfg(test)]
        {
            self.generated = 0;
            self.scanned = 0;
        }
        if ![current.clip, current.timeline, current.shared]
            .iter()
            .all(|value| value.is_finite())
        {
            return;
        }
        // Controller transitions do not restart existing clock traversals.
        // Newly enabled controllers begin at this update's observed position.
        if self.previous.is_none()
            || self.revision != traversal.revision
            || self.clip != selection.clip
        {
            self.previous = Some(traversal.origin);
            self.revision = traversal.revision;
            self.include_origin = traversal.include_origin;
            self.clip = selection.clip;
            self.controllers.clear();
            self.controllers.extend_from_slice(selection.controllers);
        }
        let previous = self.previous.unwrap();
        if paused {
            if current.clip != previous.clip
                || current.timeline != previous.timeline
                || current.shared != previous.shared
            {
                self.include_origin = false;
            }
            self.previous = Some(current);
            self.controllers.clear();
            self.controllers.extend_from_slice(selection.controllers);
            return;
        }
        if !self.include_origin && previous == current {
            self.controllers.clear();
            self.controllers.extend_from_slice(selection.controllers);
            return;
        }
        let include = self.include_origin;
        self.include_origin = false;
        self.previous = Some(current);
        let policy = selection.clip.and_then(|id| events.clips.get(&id));
        let old_controllers = &self.controllers;
        let programs = once(EventProgram::Global)
            .chain(selection.clip.map(EventProgram::Clip))
            .chain(
                selection
                    .controllers
                    .iter()
                    .copied()
                    .filter(|id| old_controllers.contains(id))
                    .map(EventProgram::Controller),
            );
        for program in programs.filter_map(|program| events.programs.get(&program)) {
            for (clock, indices) in program.groups() {
                let (from, to, minimum, maximum) = match clock {
                    Clock::Clip => {
                        let Some(policy) = policy else {
                            continue;
                        };
                        (previous.clip, current.clip, 0., policy.duration)
                    }
                    Clock::Timeline => {
                        if let Some(policy) = policy.filter(|value| value.origin.is_some()) {
                            let origin = policy.origin.unwrap();
                            (
                                previous.clip,
                                current.clip,
                                origin,
                                origin + policy.duration,
                            )
                        } else {
                            (
                                previous.timeline,
                                current.timeline,
                                previous.timeline,
                                current.timeline,
                            )
                        }
                    }
                    Clock::Shared(id) => {
                        let Some(period) = events.clocks.get(&id) else {
                            continue;
                        };
                        (previous.shared, current.shared, 0., *period)
                    }
                };
                if to < from || (!include && to == from) {
                    continue;
                }
                let first =
                    indices.partition_point(|index| events.definitions[*index].time < minimum);
                let last =
                    indices.partition_point(|index| events.definitions[*index].time <= maximum);
                for &index in &indices[first..last] {
                    #[cfg(test)]
                    {
                        self.scanned += 1;
                    }
                    let definition = &events.definitions[index];
                    let (from, to, key, period) = match definition.clock {
                        Clock::Clip => {
                            let Some(policy) = policy else { continue };
                            if definition.time > policy.duration {
                                continue;
                            }
                            (
                                previous.clip,
                                current.clip,
                                definition.time,
                                (policy.looping && policy.duration > 0.).then_some(policy.duration),
                            )
                        }
                        Clock::Timeline => {
                            if let Some(policy) = policy.filter(|value| value.origin.is_some()) {
                                let origin = policy.origin.unwrap();
                                if definition.time < origin
                                    || definition.time > origin + policy.duration
                                {
                                    continue;
                                }
                                (
                                    previous.clip,
                                    current.clip,
                                    definition.time - origin,
                                    (policy.looping && policy.duration > 0.)
                                        .then_some(policy.duration),
                                )
                            } else {
                                (previous.timeline, current.timeline, definition.time, None)
                            }
                        }
                        Clock::Shared(id) => {
                            let Some(period) = events.clocks.get(&id).copied() else {
                                continue;
                            };
                            if period <= 0. || definition.time > period {
                                continue;
                            }
                            (
                                previous.shared,
                                current.shared,
                                definition.time,
                                Some(period),
                            )
                        }
                    };
                    if to < from {
                        continue;
                    }
                    let (first, last) = if let Some(period) = period {
                        let first = ((from - key) / period).ceil().max(0.);
                        let first = if !include && key + first * period <= from {
                            first + 1.
                        } else {
                            first
                        };
                        (first, ((to - key) / period).floor())
                    } else if key <= to && (key > from || (include && key == from)) {
                        (0., 0.)
                    } else {
                        continue;
                    };
                    if first > last {
                        continue;
                    }
                    let count = (last - first + 1.).min(u64::MAX as f64) as u64;
                    self.dropped = self.dropped.saturating_add(count);
                    if budget > 0 {
                        let cycle = first;
                        let time = key + cycle * period.unwrap_or(0.);
                        let fraction = if to > from {
                            (time - from) / (to - from)
                        } else {
                            0.
                        };
                        let occurrence = EventOccurrence {
                            definition: index,
                            occurrence_time: time,
                            clocks: previous.interpolate(current, fraction),
                            fraction,
                            cycle,
                        };
                        self.heap.push(Reverse(EventStream {
                            occurrence,
                            key,
                            period: period.unwrap_or(0.),
                            last,
                            from,
                            to,
                        }));
                        #[cfg(test)]
                        {
                            self.generated += 1;
                        }
                    }
                }
            }
        }
        while self.occurrences.len() < budget {
            let Some(Reverse(mut stream)) = self.heap.pop() else {
                break;
            };
            self.occurrences.push(stream.occurrence);
            if self.occurrences.len() < budget && stream.advance(previous, current) {
                self.heap.push(Reverse(stream));
                #[cfg(test)]
                {
                    self.generated += 1;
                }
            }
        }
        self.heap.clear();
        self.dropped = self.dropped.saturating_sub(self.occurrences.len() as u64);
        self.controllers.clear();
        self.controllers.extend_from_slice(selection.controllers);
    }
    /// Converts raw occurrence clocks to sample times. The event's own clock is
    /// overridden separately with RetimedAnimation::at_occurrence.
    pub fn sample_times(
        events: &RuntimeEvents,
        selection: AnimationSample<'_>,
        occurrence: &EventOccurrence,
    ) -> (f64, f64, f64) {
        let clocks = occurrence.clocks;
        let timeline = selection
            .clip
            .and_then(|id| events.clips.get(&id))
            .and_then(|policy| {
                policy.origin.map(|origin| {
                    origin
                        + if policy.duration <= 0. {
                            0.
                        } else if policy.looping {
                            clocks.clip.rem_euclid(policy.duration)
                        } else {
                            clocks.clip.clamp(0., policy.duration)
                        }
                })
            })
            .unwrap_or(clocks.timeline);
        (clocks.clip, timeline, clocks.shared)
    }
}

#[cfg(test)]
#[path = "events_tests.rs"]
mod tests;
