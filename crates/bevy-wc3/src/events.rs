//! Owned semantic event programmes and bounded forward-clock traversal.
use bevy::prelude::*;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::iter::once;
use std::sync::Arc;
use wc3::model::{
    AnimationSample, ClipId, ClockId, EventOccurrence as SemanticEventOccurrence,
    EventSource as SemanticEventSource, EventSourceId, EventSourceSchema, Model, NodeId,
    TimeSource,
};

/// Authored event semantics; applications interpret identities and commands.
#[derive(Clone, Debug)]
pub enum EventSource {
    /// MDLX named action, including names whose prefixes have game-specific meaning.
    Mdlx(String),
    /// M2 four-byte action code with its independent game-data identity.
    M2 {
        /// Action code bytes.
        identifier: [u8; 4],
        /// External game-data identity.
        data: u32,
    },
}
/// Playback selection activating an animation event, separate from its time source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventProgram {
    /// Model-level animation selection.
    Global,
    /// Selected clip activating a reusable animation.
    Clip(ClipId),
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
    /// Model-owned event-source identity.
    pub source_id: EventSourceId,
    /// Complete authored action semantics.
    pub source: Arc<EventSource>,
    /// Playback selection activating this event occurrence.
    pub program: EventProgram,
    /// Time source on which this key is authored.
    pub clock: TimeSource,
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
    pub source_id: EventSourceId,
    pub node: Option<NodeId>,
    pub node_index: Option<usize>,
    pub source: Arc<EventSource>,
    pub program: EventProgram,
    pub clock: TimeSource,
    pub time: f64,
}
#[derive(Clone, Copy)]
struct ClipPolicy {
    duration: f64,
    looping: bool,
    origin: f64,
}
#[derive(Clone, Default)]
struct ProgramEvents {
    animation: Vec<usize>,
    shared: HashMap<ClockId, Vec<usize>>,
}
impl ProgramEvents {
    fn groups(&self) -> impl Iterator<Item = (TimeSource, &[usize])> {
        [(TimeSource::Animation, self.animation.as_slice())]
            .into_iter()
            .chain(
                self.shared
                    .iter()
                    .map(|(id, indices)| (TimeSource::Shared(*id), indices.as_slice())),
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
                TimeSource::Animation => program.animation.push(index),
                TimeSource::Shared(id) => program.shared.entry(id).or_default().push(index),
            }
        }
        for program in self.programs.values_mut() {
            for indices in [&mut program.animation]
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
            .iter()
            .enumerate()
            .map(|(index, (id, _))| (id, index))
            .collect();
        let sources: HashMap<_, _> = model
            .event_sources()
            .iter()
            .map(|(id, source)| {
                let (node, action) = match source {
                    SemanticEventSource::Mdlx(value) => {
                        (value.node, EventSource::Mdlx(value.name.to_string()))
                    }
                    SemanticEventSource::M2(value) => (
                        value.node,
                        EventSource::M2 {
                            identifier: value.identifier,
                            data: value.data,
                        },
                    ),
                };
                (id, (node, source.clock(), Arc::new(action)))
            })
            .collect();
        let mut result = Self {
            definitions: Vec::new(),
            programs: HashMap::new(),
            clocks: model
                .clocks()
                .iter()
                .map(|(id, value)| (id, value.period().as_seconds()))
                .collect(),
            clips: model
                .clips()
                .iter()
                .map(|(id, value)| {
                    let origin = value.source_start.as_seconds();
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
        let mut add = |event: &SemanticEventOccurrence, program| {
            let (node, clock, source) = &sources[&event.source];
            result.definitions.push(EventDefinition {
                source_id: event.source,
                node: Some(*node),
                node_index: node_indices.get(node).copied(),
                source: source.clone(),
                program,
                clock: *clock,
                time: event.time.as_seconds(),
            });
        };
        for event in model.global_events() {
            add(event, EventProgram::Global);
        }
        for (id, _) in model.clips() {
            for event in model.clip_events(id) {
                add(event, EventProgram::Clip(id));
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
        selection: AnimationSample,
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
        if ![current.clip, current.shared]
            .iter()
            .all(|value| value.is_finite())
        {
            return;
        }
        if self.previous.is_none()
            || self.revision != traversal.revision
            || self.clip != selection.clip
        {
            self.previous = Some(traversal.origin);
            self.revision = traversal.revision;
            self.include_origin = traversal.include_origin;
            self.clip = selection.clip;
        }
        let previous = self.previous.unwrap();
        if paused {
            if current.clip != previous.clip || current.shared != previous.shared {
                self.include_origin = false;
            }
            self.previous = Some(current);
            return;
        }
        if !self.include_origin && previous == current {
            return;
        }
        let include = self.include_origin;
        self.include_origin = false;
        self.previous = Some(current);
        let policy = selection.clip.and_then(|id| events.clips.get(&id));
        let programs = once(EventProgram::Global).chain(selection.clip.map(EventProgram::Clip));
        for program in programs.filter_map(|program| events.programs.get(&program)) {
            for (clock, indices) in program.groups() {
                let (from, to, minimum, maximum) = match clock {
                    TimeSource::Animation => {
                        if let Some(policy) = policy {
                            let origin = policy.origin;
                            (
                                previous.clip,
                                current.clip,
                                origin,
                                origin + policy.duration,
                            )
                        } else {
                            (previous.clip, current.clip, previous.clip, current.clip)
                        }
                    }
                    TimeSource::Shared(id) => {
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
                        TimeSource::Animation => {
                            if let Some(policy) = policy {
                                let origin = policy.origin;
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
                                (previous.clip, current.clip, definition.time, None)
                            }
                        }
                        TimeSource::Shared(id) => {
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
    }
    /// Converts raw occurrence clocks to sample times. The event's own clock is
    /// overridden separately with RetimedAnimation::at_occurrence.
    pub fn sample_times(occurrence: &EventOccurrence) -> (f64, f64) {
        (occurrence.clocks.clip, occurrence.clocks.shared)
    }
}

#[cfg(test)]
#[path = "events_tests.rs"]
mod tests;
