use super::super::Graph;
use super::*;
use crate::model::{
    conversion::{Context, ExportOptions},
    m3::{self as native, records as nr},
    semantic::{
        profiles::M3, Channel, Clip, Clock, Controller, Curve, Interpolation, Node, OwnerId,
        Property, Quaternion, Time, Track, Values,
    },
};

#[test]
fn exact_unsigned_keys_rebuild_typed_native_controller_tables() {
    let mut builder = Graph::builder("integer controller");
    let node = builder.reserve_node();
    let batching = Property::named(OwnerId::Node(node), "batching");
    let controls = surface::BoneControls {
        source_id: 0,
        extra_flags: Default::default(),
        reserved: 0,
        position: surface::PropertyControl {
            reset: [0.; 3],
            control: Default::default(),
            binding: Some(node.translation()),
        },
        rotation: surface::PropertyControl {
            reset: Quaternion::default(),
            control: Default::default(),
            binding: Some(node.rotation()),
        },
        scale: surface::PropertyControl {
            reset: [1.; 3],
            control: Default::default(),
            binding: Some(node.scale()),
        },
        batching: surface::Property {
            initial: 1,
            reset: 0,
            control: surface::AnimationControl {
                source_id: 42,
                interpolation: surface::values::Interpolation::Linear,
                ..Default::default()
            },
            binding: Some(batching.clone()),
        },
    };
    builder
        .set_node(node, Node::<M3>::named("bone").with_controls(controls))
        .unwrap();
    let clip = builder
        .add_clip(Clip::named("exact").with_duration(Time::seconds(1.).unwrap()))
        .unwrap();
    let controller = builder
        .add_controller(Controller {
            name: "program".into(),
            ..Default::default()
        })
        .unwrap();
    let selection = surface::SequenceControls {
        selection: Some(surface::ControllerSelection {
            name: "exact".into(),
            controllers: vec![controller],
        }),
        ..Default::default()
    };
    builder.set_clip_controls(clip, selection).unwrap();
    builder
        .add_controller_channel(
            controller,
            Channel::Unsigned(Track {
                target: batching.clone(),
                curve: Curve::new(
                    Values::owned(vec![Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()]),
                    Values::owned(vec![16_777_217, u32::MAX]),
                    Interpolation::Linear,
                )
                .unwrap(),
                clock: Clock::Timeline,
            }),
        )
        .unwrap();
    let graph = builder.finish().unwrap();
    let options = ExportOptions::default();
    let mut context = Context::export(&options);
    let ids = prepare(&graph, &mut context).unwrap();
    assert_eq!(ids[batching.target()].id, 42);
    let mut document = native::Document::<native::Md34>::new();
    let program = write(&graph, &mut document, &ids, &[node], &mut context).unwrap();
    let controller = document
        .resolve::<nr::SubTrackContainer<nr::V4>>(program.controllers.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let block = document
        .resolve::<nr::AnimationBlockU32>(controller.sdu3.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let values = document.resolve::<nr::U32>(block.keys.cast()).unwrap();
    assert_eq!(values.get(0).unwrap().unwrap().value, 16_777_217);
    assert_eq!(values.get(1).unwrap().unwrap().value, u32::MAX);
    assert_eq!(block.end_frame, 1000);
    assert_eq!(
        document
            .resolve::<nr::U32>(controller.anim_ids.cast())
            .unwrap()
            .get(0)
            .unwrap()
            .unwrap()
            .value,
        42
    );
    let sequence = document
        .resolve::<nr::Sequence<nr::V2>>(program.sequences.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(sequence.end_frame, 1000);
}

#[test]
fn unselected_program_and_orphan_keys_remain_editable() {
    use super::super::bindings::Bindings;
    use crate::model::conversion::{Context, ImportOptions};
    use crate::model::semantic::profiles::m3::{
        ControllerControls, OrderedKeys, UnboundBlock, UnboundKeys,
    };
    let mut builder = Graph::builder("program pools");
    let keys = OrderedKeys {
        times: Values::owned(vec![
            Time::seconds(-0.5).unwrap(),
            Time::seconds(2.).unwrap(),
        ]),
        values: Values::owned(vec![16_777_217u32, u32::MAX]),
    };
    builder
        .add_controller(Controller {
            name: "unselected".into(),
            controls: ControllerControls {
                unbound: vec![
                    UnboundBlock {
                        identity: Some(77),
                        flags: Default::default(),
                        end: Time::seconds(2.).unwrap(),
                        keys: UnboundKeys::Unsigned32(keys.clone()),
                    },
                    UnboundBlock {
                        identity: None,
                        flags: Default::default(),
                        end: Time::seconds(2.).unwrap(),
                        keys: UnboundKeys::Unsigned32(keys),
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();
    let graph = builder.finish().unwrap();
    let options = ExportOptions::default();
    let mut context = Context::export(&options);
    let ids = prepare(&graph, &mut context).unwrap();
    let mut document = native::Document::<native::Md34>::new();
    let program = write(&graph, &mut document, &ids, &[], &mut context).unwrap();
    let root = nr::ModelRecord::<nr::V29> {
        sequences: program.sequences,
        sub_track_collections: program.controllers,
        animation_groups: program.groups,
        animation_states: program.states,
        ..Default::default()
    };
    let root = export::append(&mut document, &[root]).unwrap();
    document.set_root(root);
    let source = native::Model::from_document(document).unwrap();
    let mut rebuilt = Graph::builder("decoded programs");
    let options = ImportOptions::default();
    read(
        &source,
        &[],
        &Bindings::default(),
        &mut rebuilt,
        &mut Context::import(&options),
    )
    .unwrap();
    let rebuilt = rebuilt.finish().unwrap();
    assert_eq!(rebuilt.clips().count(), 0);
    let (_, controller) = rebuilt.controllers().next().unwrap();
    assert_eq!(controller.name, "unselected");
    assert_eq!(controller.controls.unbound.len(), 2);
    for block in &controller.controls.unbound {
        let UnboundKeys::Unsigned32(keys) = &block.keys else {
            panic!("wrong numeric family")
        };
        assert_eq!(keys.times.get(0).unwrap().as_seconds(), -0.5);
        assert_eq!(keys.values.get(0), Some(16_777_217));
        assert_eq!(keys.values.get(1), Some(u32::MAX));
        assert!(keys.values.is_borrowed());
    }
    assert_eq!(controller.controls.unbound[0].identity, Some(77));
    assert_eq!(controller.controls.unbound[1].identity, None);
}

#[test]
fn event_blocks_select_occurrences_instead_of_command_features() {
    use crate::model::semantic::{Event, Node};
    let mut builder = Graph::builder("event groups");
    let frame = builder.add_node(Node::named("")).unwrap();
    let command = builder
        .add_feature(
            frame,
            surface::Feature::Event(
                (surface::EventCommand {
                    event_type: 3,
                    transform: [[0.; 4]; 4],
                    options: "command".into(),
                    render_target_channel: Some(4),
                    parameter: Some(5),
                    control: 6,
                })
                .into(),
            ),
        )
        .unwrap();
    let controller = builder
        .add_controller(Controller {
            name: "events".into(),
            ..Default::default()
        })
        .unwrap();
    let mut events = Vec::new();
    for time in [0.25, 0.75] {
        events.push(
            builder
                .add_controller_event(
                    controller,
                    Event {
                        feature: Some(command),
                        clock: Clock::Timeline,
                        time: Time::seconds(time).unwrap(),
                        name: "shared command".into(),
                        node: None,
                        payload: Values::default(),
                    },
                )
                .unwrap(),
        );
    }
    builder
        .set_controller_controls(
            controller,
            surface::ControllerControls {
                blocks: events
                    .into_iter()
                    .enumerate()
                    .map(|(index, event)| surface::BlockControls {
                        source_id: index as u32 + 90,
                        targets: Vec::new(),
                        events: vec![event],
                        extents: Vec::new(),
                        encoding: surface::KeyEncoding::Event,
                        flags: Default::default(),
                        end: Time::seconds(1.).unwrap(),
                    })
                    .collect(),
                ..Default::default()
            },
        )
        .unwrap();
    let graph = builder.finish().unwrap();
    let mut document = native::Document::<native::Md34>::new();
    let options = ExportOptions::default();
    let mut context = Context::export(&options);
    let ids = prepare(&graph, &mut context).unwrap();
    let program = write(&graph, &mut document, &ids, &[], &mut context).unwrap();
    let controller = document
        .resolve::<nr::SubTrackContainer<nr::V4>>(program.controllers.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let blocks = document
        .resolve::<nr::AnimationBlockEvent>(controller.sdev.cast())
        .unwrap();
    assert_eq!(blocks.len(), 2);
    for (index, block) in blocks.iter().enumerate() {
        let block = block.unwrap();
        assert_eq!(block.keys.entries, 1);
        assert_eq!(
            document
                .resolve::<nr::I32>(block.timestamps.cast())
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap()
                .value,
            [250, 750][index]
        );
    }
}

#[test]
fn named_selection_libraries_and_legacy_sequence_controls_roundtrip_independently() {
    use super::super::bindings::Bindings;
    use crate::model::conversion::ImportOptions;
    let mut builder = Graph::builder("selection libraries");
    let controller = builder
        .add_controller(Controller {
            name: "pool".into(),
            ..Default::default()
        })
        .unwrap();
    let controls = surface::SequenceControls {
        sequence_number: -19,
        legacy_control: Some(0x8000_1234),
        selection: Some(surface::ControllerSelection {
            name: "group name".into(),
            controllers: vec![controller],
        }),
        ..Default::default()
    };
    let mut clip = Clip::named("clip name").with_duration(Time::seconds(1.).unwrap());
    clip.controls = controls;
    builder.add_clip(clip).unwrap();
    builder
        .add_system(surface::System::ControllerSelections(vec![
            surface::ControllerSelection {
                name: "unselected library".into(),
                controllers: vec![controller],
            },
        ]))
        .unwrap();
    let graph = builder.finish().unwrap();
    let options = ExportOptions::default();
    let mut context = Context::export(&options);
    let ids = prepare(&graph, &mut context).unwrap();
    let mut document = native::Document::<native::Md34>::new();
    let program = write(&graph, &mut document, &ids, &[], &mut context).unwrap();
    let sequence = document
        .resolve::<nr::Sequence<nr::V1>>(program.sequences.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(sequence.index, -19);
    assert_eq!(sequence.deprecated_unknown, 0x8000_1234);
    assert_eq!(program.groups.entries, 2);
    let root = nr::ModelRecord::<nr::V29> {
        sequences: program.sequences,
        sub_track_collections: program.controllers,
        animation_groups: program.groups,
        animation_states: program.states,
        ..Default::default()
    };
    let reference = export::append(&mut document, &[root]).unwrap();
    document.set_root(reference);
    let source = native::Model::from_document(document).unwrap();
    let mut rebuilt = Graph::builder("decoded selections");
    read(
        &source,
        &[],
        &Bindings::default(),
        &mut rebuilt,
        &mut Context::import(&ImportOptions::default()),
    )
    .unwrap();
    let graph = rebuilt.finish().unwrap();
    let (_, clip) = graph.clips().next().unwrap();
    assert_eq!(clip.name, "clip name");
    assert_eq!(clip.controls.selection.as_ref().unwrap().name, "group name");
    assert_eq!(clip.controls.sequence_number, -19);
    assert_eq!(clip.controls.legacy_control, Some(0x8000_1234));
    let libraries = graph
        .systems()
        .find_map(|(_, system)| match system {
            surface::System::ControllerSelections(value) => Some(value),
            _ => None,
        })
        .unwrap();
    assert_eq!(libraries[0].name, "unselected library");
    assert_eq!(
        libraries[0].controllers,
        clip.controls.selection.as_ref().unwrap().controllers
    );
}
