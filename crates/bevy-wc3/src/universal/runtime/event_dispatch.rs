use super::super::events::{
    EventClocks, EventState, UniversalEventDispatchStats, UniversalModelEvent,
};
use super::super::pose::PoseContext;
use super::{UniversalInstance, UniversalPlayback, UniversalPoseOverrides};
use bevy::prelude::*;
use wc3::model::Time as ModelTime;

pub(crate) fn dispatch_universal_events(
    mut instances: Query<(
        Entity,
        &mut UniversalInstance,
        &UniversalPlayback,
        &UniversalPoseOverrides,
        &mut UniversalEventDispatchStats,
    )>,
    globals: Query<&GlobalTransform>,
    mut messages: MessageWriter<UniversalModelEvent>,
) {
    for (root, mut instance, playback, overrides, mut stats) in &mut instances {
        if instance.model.events.is_empty() {
            continue;
        }
        let timing = instance.model.timing.clone();
        let Ok(selection) = playback.selection_from_timing(&timing) else {
            continue;
        };
        let events = instance.model.events.clone();
        instance.events.collect(
            &events,
            selection,
            EventClocks {
                clip: playback.clip_time,
                timeline: playback.timeline_time,
                shared: playback.shared_time,
            },
            playback.event_traversal,
            playback.paused || playback.speed < 0.,
            4096,
        );
        let next = UniversalEventDispatchStats {
            emitted: instance.events.occurrences.len(),
            dropped: instance.events.dropped,
        };
        if stats.emitted != next.emitted || stats.dropped != next.dropped {
            *stats = next;
        }
        if instance.events.occurrences.is_empty() {
            continue;
        }
        let animation = instance.model.animation.clone();
        let Ok(sample) = animation.sample(selection) else {
            continue;
        };
        let Ok(basis) = globals.get(instance.basis_entity) else {
            continue;
        };
        let camera = playback
            .driving_camera
            .and_then(|id| globals.get(id).ok())
            .and_then(|camera| {
                (basis.affine().matrix3.determinant().abs() > 1e-12)
                    .then(|| basis.affine().inverse() * camera.affine())
            });
        let UniversalInstance {
            model,
            nodes,
            events: state,
            event_scratch,
            transition,
            ..
        } = &mut *instance;
        let context = PoseContext {
            nodes: &model.nodes,
            transition: transition.view_at(playback.transition_clock),
            overrides: &overrides.0,
            camera,
        };
        for occurrence in &state.occurrences {
            let definition = &events.definitions[occurrence.definition];
            let (clip, timeline, shared) = EventState::sample_times(&events, selection, occurrence);
            let (Ok(clip), Ok(timeline), Ok(shared), Ok(key)) = (
                ModelTime::seconds(clip),
                ModelTime::seconds(timeline),
                ModelTime::seconds(shared),
                ModelTime::seconds(definition.time),
            ) else {
                continue;
            };
            let retimed = sample
                .at_times(clip, timeline, shared)
                .at_occurrence(definition.clock, key);
            let node_index = definition.node_index;
            let transform = node_index.map_or(basis.to_matrix(), |index| {
                basis.to_matrix()
                    * context
                        .at_offset(occurrence.clocks.shared - playback.shared_time)
                        .sample_feature_node(index, &retimed, event_scratch)
                        .0
            });
            messages.write(UniversalModelEvent {
                root,
                node: node_index.map(|index| nodes[index]),
                node_id: definition.node,
                feature: definition.feature,
                name: definition.name.clone(),
                source: definition.source.clone(),
                payload: definition.payload.clone(),
                program: definition.program,
                clock: definition.clock,
                key_time: definition.time,
                occurrence_time: occurrence.occurrence_time,
                transform: GlobalTransform::from(transform),
            });
        }
    }
}
