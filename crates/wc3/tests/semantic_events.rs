use wc3::model::profiles::{mdlx, Mdlx, Universal};
use wc3::model::{Animation, EventOccurrence, EventSource, Model, SharedClock, Time, TimeSource};

#[test]
fn sources_preserve_identity_borrowing_and_empty_shared_definitions_across_profiles() {
    let name = String::from("Footstep");
    let mut draft = Model::<Mdlx>::builder("");
    let node = draft.add_named_node("foot").unwrap();
    let clock = draft.add_clock(SharedClock::Frozen).unwrap();
    let source = draft.add_event_source(mdlx::EventSource {
        node,
        clock: TimeSource::Shared(clock),
        name: name.as_str().into(),
    });
    let model = draft.finish().unwrap().widen::<Universal>();
    let EventSource::Mdlx(definition) = model.event_source(source).unwrap() else {
        panic!("named event source changed kind");
    };
    assert_eq!(definition.name.as_ptr(), name.as_ptr());
    assert_eq!(definition.clock, TimeSource::Shared(clock));
    let restored = model.try_narrow::<Mdlx>().unwrap();
    assert_eq!(restored.event_sources().len(), 1);
    assert_eq!(restored.event_source(source).unwrap().node, node);
    assert!(restored.animations().is_empty());
}

#[test]
fn occurrences_require_owned_sources_and_source_edits_are_transactional() {
    let mut draft = Model::<Mdlx>::builder("");
    let node = draft.add_named_node("foot").unwrap();
    let definition = mdlx::EventSource {
        node,
        clock: TimeSource::Animation,
        name: "Footstep".into(),
    };
    let source = draft.add_event_source(definition.clone());
    let independent = draft.add_event_source(definition);
    assert_ne!(source, independent);
    let animation = draft.add_animation(Animation::default());
    let occurrence = draft
        .add_animation_event(
            animation,
            EventOccurrence {
                source,
                time: Time::seconds(0.5).unwrap(),
            },
        )
        .unwrap();
    let mut model = draft.finish().unwrap();
    assert!(model
        .edit(|draft| {
            draft.remove_event_source(source);
            Ok(())
        })
        .is_err());
    assert_eq!(model.event(occurrence).unwrap().source, source);
    assert!(model.event_source(source).is_some());

    let mut foreign = Model::<Mdlx>::builder("");
    let animation = foreign.add_animation(Animation::default());
    foreign
        .add_animation_event(
            animation,
            EventOccurrence {
                source,
                time: Time::default(),
            },
        )
        .unwrap();
    assert!(foreign.finish().is_err());
}

#[test]
fn even_unused_sources_validate_their_anchor_and_clock() {
    let mut owner = Model::<Mdlx>::builder("");
    let node = owner.add_named_node("foreign").unwrap();
    let clock = owner.add_clock(SharedClock::Frozen).unwrap();
    let mut draft = Model::<Mdlx>::builder("");
    draft.add_event_source(mdlx::EventSource {
        node,
        clock: TimeSource::Animation,
        name: "action".into(),
    });
    assert!(draft.finish().is_err());
    let mut draft = Model::<Mdlx>::builder("");
    let node = draft.add_named_node("local").unwrap();
    draft.add_event_source(mdlx::EventSource {
        node,
        clock: TimeSource::Shared(clock),
        name: "action".into(),
    });
    assert!(draft.finish().is_err());
}

#[test]
fn exact_narrowing_rejects_incompatible_sources_even_without_occurrences() {
    use wc3::model::profiles::{m2, Narrows, Wmo, M2};
    let mut draft = Model::<Mdlx>::builder("");
    let node = draft.add_named_node("anchor").unwrap();
    let named = EventSource::Mdlx(mdlx::EventSource {
        node,
        clock: TimeSource::Animation,
        name: "Footstep".into(),
    });
    assert!(<M2 as Narrows<Universal>>::event_source(named.clone()).is_err());
    assert!(<Wmo as Narrows<Universal>>::event_source(named).is_err());
    let mut model = draft.finish().unwrap().widen::<Universal>();
    model
        .edit(|draft| {
            draft.add_event_source(EventSource::M2(m2::EventSource {
                node,
                clock: TimeSource::Animation,
                identifier: *b"$SND",
                data: 7,
            }));
            Ok(())
        })
        .unwrap();
    assert!(model.try_narrow::<Mdlx>().is_err());
}
