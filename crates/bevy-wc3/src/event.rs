//! Discrete model notifications; applications interpret their names.
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use wc3::model::scene::EventObject;

use crate::animation::pose::{sample_emitter_transform_at_frame, EmitterNodes, NodeFrame};
use crate::animation::Wc3Animation;
use crate::preparation::PreparedModel;

/// An authored event key crossed during forward model playback.
///
/// Read with `MessageReader<Wc3ModelEvent>` in `PostUpdate`, after
/// `Wc3Systems::DispatchEvents`, or in the next `Update`. Names are preserved
/// verbatim; sound, visual-effect, and gameplay behavior belongs to the consumer.
/// Events are ordered per instance by occurrence time, event index, then key
/// index. Ordering between different model instances is unspecified.
#[derive(Message, Clone, Debug)]
pub struct Wc3ModelEvent {
    /// The independently animated model instance that produced this event.
    pub root: Entity,
    /// Animated event node in that instance's rig.
    pub node: Entity,
    /// Authored object ID of the event node, rather than its collection index.
    pub object_id: u32,
    /// Authored event-node name for the application to interpret.
    pub name: String,
    /// Event record index in the source model's event-object collection.
    pub event_index: usize,
    /// Key index in the event object's sorted frames, retaining duplicates.
    pub key_index: usize,
    /// Sequence selected when dispatching; `None` when no sequence exists.
    pub sequence: Option<usize>,
    /// Global-sequence index for this event, if present.
    pub global_sequence_id: Option<u32>,
    /// Authored model or global-sequence timestamp in milliseconds.
    pub frame: i32,
    /// Unwrapped occurrence time, in elapsed playback milliseconds.
    pub elapsed_ms: f64,
    /// Node pose sampled at the occurrence time. External root/ancestor
    /// transforms and driving cameras use their current update values.
    pub transform: GlobalTransform,
}

#[derive(Component)]
pub(crate) struct EventState {
    definitions: Arc<[EventObject]>,
    nodes: Vec<Entity>,
    cursor: Option<EventCursor>,
}

struct EventCursor {
    revision: u64,
    elapsed_ms: f64,
    include_from: bool,
}

pub(crate) fn spawn_events(
    commands: &mut Commands,
    prepared: &PreparedModel,
    root: Entity,
    nodes: &HashMap<u32, Entity>,
) {
    if prepared.events.is_empty() {
        return;
    }
    commands.entity(root).insert(EventState {
        definitions: prepared.events.clone(),
        nodes: prepared
            .events
            .iter()
            .map(|event| nodes[&event.node.object_id])
            .collect(),
        cursor: None,
    });
}

pub(crate) fn dispatch_events(
    mut roots: Query<(Entity, &Wc3Animation, &mut EventState)>,
    nodes: EmitterNodes,
    mut messages: MessageWriter<Wc3ModelEvent>,
) {
    for (root, animation, mut state) in &mut roots {
        let playback = &animation.event_playback;
        if state
            .cursor
            .as_ref()
            .is_none_or(|cursor| cursor.revision != playback.revision)
        {
            state.cursor = Some(EventCursor {
                revision: playback.revision,
                elapsed_ms: playback.origin_ms,
                include_from: playback.include_origin,
            });
        }
        let cursor = state.cursor.as_mut().unwrap();
        let from_ms = cursor.elapsed_ms;
        let include_from = cursor.include_from;
        let to_ms = animation.elapsed_ms;
        if !to_ms.is_finite() {
            continue;
        }
        if !animation.playing || animation.speed < 0.0 || to_ms < from_ms {
            // Preserve a pending start event while paused at its origin.
            if to_ms != from_ms || animation.speed < 0.0 {
                cursor.include_from = false;
            }
            cursor.elapsed_ms = to_ms;
            continue;
        }
        cursor.elapsed_ms = to_ms;
        cursor.include_from = false;

        let mut crossed = Vec::new();
        for (event_index, event) in state.definitions.iter().enumerate() {
            crossed.extend(
                animation
                    .time()
                    .event_occurrences(event, from_ms, include_from)
                    .into_iter()
                    .map(|occurrence| (event_index, occurrence)),
            );
        }
        crossed.sort_by(|(ai, a), (bi, b)| {
            a.elapsed_ms
                .total_cmp(&b.elapsed_ms)
                .then(ai.cmp(bi))
                .then(a.key_index.cmp(&b.key_index))
        });
        if crossed.is_empty() {
            continue;
        }
        let mut birth = animation.clone();
        for (event_index, occurrence) in crossed {
            let event = &state.definitions[event_index];
            let node = state.nodes[event_index];
            birth.set_sample_time(occurrence.elapsed_ms);
            let frame = Some(NodeFrame {
                frame: occurrence.frame,
                global_sequence_id: (event.global_sequence_id != u32::MAX)
                    .then_some(event.global_sequence_id),
            });
            let Some(transform) =
                sample_emitter_transform_at_frame(node, root, &birth, &nodes, frame)
            else {
                continue;
            };
            messages.write(Wc3ModelEvent {
                root,
                node,
                object_id: event.node.object_id,
                name: event.node.name.text().into_owned(),
                event_index,
                key_index: occurrence.key_index,
                sequence: animation.time().sequence.map(|_| animation.sequence),
                global_sequence_id: (event.global_sequence_id != u32::MAX)
                    .then_some(event.global_sequence_id),
                frame: occurrence.frame,
                elapsed_ms: occurrence.elapsed_ms,
                transform,
            });
        }
    }
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
