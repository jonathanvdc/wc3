use super::*;
use wc3::model::ModelMetadata;
use wc3::model::{Clip, Controller, ModelBuilder, Time};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};

#[test]
fn controller_activation_and_reordering_do_not_replay_existing_programmes() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let a = builder.add_controller(Controller::default()).unwrap();
    let b = builder.add_controller(Controller::default()).unwrap();
    let mut controller = definition(TimeSource::Animation, 0.5);
    controller.program = EventProgram::Controller(a);
    let events = RuntimeEvents {
        definitions: vec![
            definition(TimeSource::Animation, 0.5),
            definition(TimeSource::Animation, 0.9),
            controller,
        ],
        ..SlotConstruct::construct(&mut slot_values)
    }
    .indexed();
    let mut state = EventState::default();
    collect(&mut state, &events, 0.75, 32);
    assert_eq!(
        state
            .occurrences
            .iter()
            .map(|value| value.definition)
            .collect::<Vec<_>>(),
        [0]
    );
    let controllers = [a, b];
    state.collect(
        &events,
        AnimationSample {
            controllers: &controllers,
            ..SlotConstruct::construct(&mut slot_values)
        },
        clocks(1.),
        EventTraversal::default(),
        false,
        32,
    );
    assert_eq!(
        state
            .occurrences
            .iter()
            .map(|value| value.definition)
            .collect::<Vec<_>>(),
        [1]
    );
    let reversed = [b, a];
    state.collect(
        &events,
        AnimationSample {
            controllers: &reversed,
            ..SlotConstruct::construct(&mut slot_values)
        },
        clocks(1.2),
        EventTraversal::default(),
        false,
        32,
    );
    assert!(state.occurrences.is_empty());
    // Removing controllers similarly leaves global traversal where it was.
    collect(&mut state, &events, 1.3, 32);
    assert!(state.occurrences.is_empty());
}

fn definition(clock: TimeSource, time: f64) -> EventDefinition {
    EventDefinition {
        feature: None,
        node: None,
        node_index: None,
        name: Arc::from("event"),
        source: None,
        payload: Arc::from([]),
        program: EventProgram::Global,
        clock,
        time,
    }
}

#[test]
fn many_looping_definitions_generate_only_seeds_plus_global_dispatch_budget() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let clock = builder
        .add_clock(wc3::model::SharedClock::Repeating {
            period: Time::seconds(1.).unwrap(),
        })
        .unwrap();
    let definitions = 1000;
    let budget = 64;
    let events = RuntimeEvents {
        definitions: (0..definitions)
            .map(|_| definition(TimeSource::Shared(clock), 0.))
            .collect(),
        clocks: HashMap::from([(clock, 1.)]),
        ..SlotConstruct::construct(&mut slot_values)
    }
    .indexed();
    let mut state = EventState::default();
    collect(&mut state, &events, 1e12, budget);
    assert_eq!(state.occurrences.len(), budget);
    assert_eq!(state.generated, definitions + budget - 1);
    assert_eq!(
        state.dropped,
        definitions as u64 * 1_000_000_000_001 - budget as u64
    );
    assert_eq!(
        state
            .occurrences
            .iter()
            .map(|value| value.definition)
            .collect::<Vec<_>>(),
        (0..budget).collect::<Vec<_>>()
    );
    assert!(state
        .occurrences
        .iter()
        .all(|value| value.occurrence_time == 0.));
    let capacity = state.heap.capacity();
    collect(&mut state, &events, 2e12, budget);
    assert!(state.generated < definitions + budget);
    assert_eq!(state.heap.capacity(), capacity);
}

#[test]
fn compilation_resolves_event_node_index_once() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let node = builder
        .add_node(wc3::model::Node::named("anchor", &mut slot_values))
        .unwrap();
    builder
        .add_global_event(Event {
            feature: None,
            clock: TimeSource::Animation,
            time: Time::seconds(0.5).unwrap(),
            name: "key".into(),
            node: Some(node),
            payload: SlotConstruct::construct(&mut slot_values),
        })
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let events = RuntimeEvents::compile(&model);
    assert_eq!(events.definitions[0].node, Some(node));
    assert_eq!(events.definitions[0].node_index, Some(0));
}
fn clocks(value: f64) -> EventClocks {
    EventClocks {
        clip: value,
        shared: value,
    }
}
fn collect(state: &mut EventState, events: &RuntimeEvents, time: f64, budget: usize) {
    state.collect(
        events,
        AnimationSample::default(),
        clocks(time),
        EventTraversal::default(),
        false,
        budget,
    );
}
#[test]
fn forward_intervals_keep_duplicate_keys_and_seek_skips_destination() {
    let mut slot_values = SlotValues::default();
    let events = RuntimeEvents {
        definitions: vec![
            definition(TimeSource::Animation, 0.),
            definition(TimeSource::Animation, 0.5),
            definition(TimeSource::Animation, 0.5),
        ],
        ..SlotConstruct::construct(&mut slot_values)
    }
    .indexed();
    let mut state = EventState::default();
    collect(&mut state, &events, 0.75, 32);
    assert_eq!(
        state
            .occurrences
            .iter()
            .map(|value| value.definition)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    collect(&mut state, &events, 0.75, 32);
    assert!(state.occurrences.is_empty());
    state.reset(clocks(0.5), 1, false);
    state.collect(
        &events,
        AnimationSample::default(),
        clocks(0.75),
        EventTraversal {
            revision: 1,
            origin: clocks(0.5),
            include_origin: false,
        },
        false,
        32,
    );
    assert!(state.occurrences.is_empty());
}
#[test]
fn paused_start_keys_wait_and_reverse_traversal_does_not_dispatch() {
    let mut slot_values = SlotValues::default();
    let events = RuntimeEvents {
        definitions: vec![
            definition(TimeSource::Animation, 0.),
            definition(TimeSource::Animation, 0.5),
        ],
        ..SlotConstruct::construct(&mut slot_values)
    }
    .indexed();
    let mut state = EventState::default();
    state.collect(
        &events,
        AnimationSample::default(),
        clocks(0.),
        EventTraversal::default(),
        true,
        32,
    );
    assert!(state.occurrences.is_empty());
    collect(&mut state, &events, 0., 32);
    assert_eq!(state.occurrences.len(), 1);
    collect(&mut state, &events, 0.75, 32);
    assert_eq!(state.occurrences.len(), 1);
    collect(&mut state, &events, 0.25, 32);
    assert!(state.occurrences.is_empty());
    collect(&mut state, &events, 0.6, 32);
    assert_eq!(state.occurrences.len(), 1);
}
#[test]
fn inclusive_loop_end_and_next_start_are_distinct_and_large_jumps_are_bounded() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let id = builder
        .add_clock(wc3::model::SharedClock::Repeating {
            period: Time::seconds(1.).unwrap(),
        })
        .unwrap();
    let events = RuntimeEvents {
        definitions: vec![
            definition(TimeSource::Shared(id), 0.),
            definition(TimeSource::Shared(id), 1.),
        ],
        clocks: HashMap::from([(id, 1.)]),
        ..SlotConstruct::construct(&mut slot_values)
    }
    .indexed();
    let mut state = EventState::default();
    collect(&mut state, &events, 1., 32);
    assert_eq!(
        state
            .occurrences
            .iter()
            .map(|value| (value.definition, value.occurrence_time))
            .collect::<Vec<_>>(),
        [(0, 0.), (0, 1.), (1, 1.)]
    );
    collect(&mut state, &events, 1e12, 7);
    assert_eq!(state.occurrences.len(), 7);
    assert!(state.dropped > 1_000_000);
    assert!(state.occurrences.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(state.occurrences[0].occurrence_time, 2.);
}
#[test]
fn timeline_keys_follow_selected_clip_interval_and_alias_identity() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let id = builder
        .add_clip(Clip::named("clip").with_duration(Time::seconds(1.).unwrap()))
        .unwrap();
    let mut source = definition(TimeSource::Animation, 10.5);
    source.program = EventProgram::Clip(id);
    let events = RuntimeEvents {
        definitions: vec![source, definition(TimeSource::Animation, 9.5)],
        clips: HashMap::from([(
            id,
            ClipPolicy {
                duration: 1.,
                looping: true,
                origin: 10.,
            },
        )]),
        ..SlotConstruct::construct(&mut slot_values)
    }
    .indexed();
    let selection = AnimationSample {
        clip: Some(id),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut state = EventState::default();
    state.collect(
        &events,
        selection,
        clocks(2.6),
        EventTraversal::default(),
        false,
        32,
    );
    assert_eq!(
        state
            .occurrences
            .iter()
            .map(|value| value.occurrence_time)
            .collect::<Vec<_>>(),
        [0.5, 1.5, 2.5]
    );
    let times = EventState::sample_times(&state.occurrences[1]);
    assert_eq!(times, (1.5, 1.5));
}

#[test]
fn seek_origin_preserves_forward_keys_crossed_before_first_dispatch() {
    let mut slot_values = SlotValues::default();
    let events = RuntimeEvents {
        definitions: vec![
            definition(TimeSource::Animation, 0.5),
            definition(TimeSource::Animation, 0.55),
            definition(TimeSource::Animation, 0.6),
        ],
        ..SlotConstruct::construct(&mut slot_values)
    }
    .indexed();
    let mut state = EventState::default();
    let mut playback = super::super::runtime::Playback::default();
    playback.seek(0.5).unwrap();
    state.collect(
        &events,
        AnimationSample::default(),
        clocks(0.6),
        playback.event_traversal,
        false,
        32,
    );
    assert_eq!(
        state
            .occurrences
            .iter()
            .map(|value| value.definition)
            .collect::<Vec<_>>(),
        [1, 2]
    );
}

#[test]
fn inactive_programs_and_unchanged_or_paused_updates_do_not_scan_event_definitions() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let mut events = RuntimeEvents::default();
    events
        .definitions
        .push(definition(TimeSource::Animation, 0.25));
    let mut selected = None;
    for index in 0..4096 {
        let id = builder
            .add_clip(Clip::named(format!("clip{index}")))
            .unwrap();
        events.clips.insert(
            id,
            ClipPolicy {
                duration: 1.,
                looping: true,
                origin: 0.,
            },
        );
        let mut event = definition(TimeSource::Animation, 0.5);
        event.program = EventProgram::Clip(id);
        events.definitions.push(event);
        selected = Some(id);
    }
    let events = events.indexed();
    let selection = AnimationSample {
        clip: selected,
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut state = EventState::default();
    state.collect(
        &events,
        selection,
        clocks(0.75),
        EventTraversal::default(),
        false,
        32,
    );
    assert_eq!(state.scanned, 2);
    assert_eq!(state.occurrences.len(), 2);
    assert_eq!(state.generated, 2);
    state.collect(
        &events,
        selection,
        clocks(0.75),
        EventTraversal::default(),
        false,
        32,
    );
    assert_eq!(state.scanned, 0);
    assert!(state.occurrences.is_empty());
    state.collect(
        &events,
        selection,
        clocks(1.),
        EventTraversal::default(),
        true,
        32,
    );
    assert_eq!(state.scanned, 0);
    assert!(state.occurrences.is_empty());
}

#[test]
fn interval_clip_index_limits_global_timeline_scans_to_its_authored_window() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let id = builder.add_clip(Clip::named("interval")).unwrap();
    let events = RuntimeEvents {
        definitions: (0..10000)
            .map(|index| definition(TimeSource::Animation, f64::from(index) + 0.5))
            .collect(),
        clips: HashMap::from([(
            id,
            ClipPolicy {
                duration: 1.,
                looping: true,
                origin: 5000.,
            },
        )]),
        ..SlotConstruct::construct(&mut slot_values)
    }
    .indexed();
    let selection = AnimationSample {
        clip: Some(id),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let mut state = EventState::default();
    state.collect(
        &events,
        selection,
        clocks(0.75),
        EventTraversal::default(),
        false,
        32,
    );
    assert_eq!(state.scanned, 1);
    assert_eq!(state.occurrences.len(), 1);
    assert_eq!(state.occurrences[0].definition, 5000);
}

#[test]
fn frozen_shared_clock_does_not_traverse_event_windows() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let clock = builder.add_clock(wc3::model::SharedClock::Frozen).unwrap();
    let events = RuntimeEvents {
        definitions: vec![definition(TimeSource::Shared(clock), 0.)],
        clocks: [(clock, 0.)].into(),
        ..SlotConstruct::construct(&mut slot_values)
    }
    .indexed();
    let mut state = EventState::default();
    for time in [0., 0.5, 100.] {
        collect(&mut state, &events, time, 32);
        assert!(state.occurrences.is_empty());
    }
}
