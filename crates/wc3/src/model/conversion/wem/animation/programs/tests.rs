use super::super::{super::stages, routing, targets};
use super::{
    codec::append,
    inventory::prepare_clips,
    read::{decode_model, install},
    write::{write_clips, write_model},
};
use crate::model::conversion::{Context, ExportOptions, ImportOptions};
use crate::model::semantic::{profiles::Wem, Clip, Controller, ModelCollectionBuilder};
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::BTreeMap;

#[test]
fn historical_clip_fields_and_shared_prefix_layers_remain_independent() {
    let mut source = RawDocument::new();
    let containers = append(
        &mut source,
        &[
            records::SubTrackContainer::default(),
            records::SubTrackContainer {
                priority: 17,
                ..Default::default()
            },
        ],
    )
    .unwrap();
    let clips = append(
        &mut source,
        &[
            records::Clip::<records::V1> {
                containers: Ref::from_section(1, containers.index),
                duration: 1.,
                ..Default::default()
            },
            records::Clip::<records::V1> {
                containers: containers.cast(),
                duration: 2.,
                ..Default::default()
            },
        ],
    )
    .unwrap();
    let mut builder = ModelCollectionBuilder::<Wem>::new();
    builder.add_model("test");
    let mut context = Context::import(&ImportOptions::default());
    let prepared = prepare_clips(&source, clips.cast(), &mut builder, &mut context).unwrap();
    let stages = stages::Decoded {
        value: Default::default(),
        stages: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    let model = builder.model_mut(0).unwrap();
    let routing = routing::ReadContext {
        model: model.model(),
        simulation: targets::ReadContext {
            nodes: &[],
            physics: None,
            stages: &stages,
        },
        meshes: &[],
        sections: &[],
        slots: &[],
    };
    let decoded = decode_model(
        &source,
        &prepared,
        super::SourceTables {
            model: 0,
            channels: Ref::default(),
            selections: Ref::default(),
        },
        &routing,
        &[],
        &mut context,
    )
    .unwrap();
    install(decoded, model).unwrap();
    let clips: Vec<_> = model.model().clips().map(|(_, clip)| clip).collect();
    assert_eq!(clips[0].controls.programs[0], clips[1].controls.programs[0]);
    assert!(clips[0].controls.bounds.is_none() && clips[0].controls.read_rule.is_none());
    let mut target = RawDocument::new();
    let nodes = BTreeMap::new();
    let meshes = BTreeMap::new();
    let sections = BTreeMap::new();
    let slots = BTreeMap::new();
    let features = BTreeMap::new();
    let stages = stages::Encoded {
        value: Ref::default(),
        stages: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    let routing = routing::WriteContext {
        model: model.model(),
        simulation: targets::WriteContext {
            nodes: &nodes,
            physics: None,
            stages: &stages,
            pipeline: &stages_pipeline(),
        },
        meshes: &meshes,
        sections: &sections,
        slots: &slots,
        features: &features,
    };
    let written = write_model(
        &mut target,
        model.model(),
        &routing,
        &BTreeMap::new(),
        &mut Context::export(&ExportOptions::default()),
    )
    .unwrap();
    let (reference, _) = write_clips(
        &mut target,
        written.clips.into_iter().map(|clip| (0, clip)).collect(),
    )
    .unwrap();
    assert_eq!(
        target
            .referenced_section(reference)
            .unwrap()
            .unwrap()
            .revision(),
        1
    );
    let clips = target
        .resolve::<records::Clip<records::V1>>(reference.cast())
        .unwrap();
    let a = clips.get(0).unwrap().unwrap();
    let b = clips.get(1).unwrap().unwrap();
    assert_eq!(a.containers.index, b.containers.index);
    assert_eq!((a.containers.entries, b.containers.entries), (1, 2));
}
fn stages_pipeline() -> crate::model::semantic::profiles::wem::PosePipeline<'static> {
    Default::default()
}

#[test]
fn overlapping_nonprefix_layer_lists_cannot_duplicate_authority() {
    let mut model = crate::model::semantic::Model::<Wem>::builder("test");
    let a = model.add_controller(Controller::default()).unwrap();
    let b = model.add_controller(Controller::default()).unwrap();
    let c = model.add_controller(Controller::default()).unwrap();
    let mut first = Clip::<Wem>::named("first");
    first.controls.programs = vec![a, b];
    model.add_clip(first).unwrap();
    let mut second = Clip::<Wem>::named("second");
    second.controls.programs = vec![a, c];
    model.add_clip(second).unwrap();
    let mut document = RawDocument::new();
    let declarations = super::declarations::Written {
        channels: Ref::default(),
        selections: Ref::default(),
        ids: Default::default(),
        selection_ids: Default::default(),
    };
    let error = super::controllers::write(
        &mut document,
        model.model(),
        &declarations,
        &BTreeMap::new(),
        &mut Context::export(&ExportOptions::default()),
    )
    .err()
    .unwrap();
    assert!(error.to_string().contains("prefix"));
}

#[test]
fn recorded_bake_preserves_editable_report_and_invalidates_cache_fingerprints() {
    use crate::model::wem::InlineOption;
    let mut source = RawDocument::new();
    let recipe = records::ClipPhysics::<records::V7> {
        baked: InlineOption::from(Some(records::PhysicsBake {
            inputs: 17,
            written: 31,
            max_error: 0.25,
            seam_before: 3.,
            seam_after: 1.,
            ..Default::default()
        })),
        ..Default::default()
    };
    let clips = append(
        &mut source,
        &[records::Clip::<records::V7> {
            physics: InlineOption::from(Some(recipe)),
            duration: 2.,
            ..Default::default()
        }],
    )
    .unwrap();
    let mut builder = ModelCollectionBuilder::<Wem>::new();
    builder.add_model("test");
    let mut context = Context::import(&ImportOptions::default());
    let prepared = prepare_clips(&source, clips.cast(), &mut builder, &mut context).unwrap();
    let stages = stages::Decoded {
        value: Default::default(),
        stages: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    let model = builder.model_mut(0).unwrap();
    let routing = routing::ReadContext {
        model: model.model(),
        simulation: targets::ReadContext {
            nodes: &[],
            physics: None,
            stages: &stages,
        },
        meshes: &[],
        sections: &[],
        slots: &[],
    };
    let decoded = decode_model(
        &source,
        &prepared,
        super::SourceTables {
            model: 0,
            channels: Ref::default(),
            selections: Ref::default(),
        },
        &routing,
        &[],
        &mut context,
    )
    .unwrap();
    let hints = install(decoded, model).unwrap();
    assert!(matches!(
        hints.as_slice(),
        [
            crate::model::conversion::EncodingHint::WemBakeFingerprints {
                inputs: 17,
                written: 31,
                ..
            }
        ]
    ));
    let id = prepared.clip_ids[0];
    let mut clip = model.model().clip(id).unwrap().clone();
    let report = clip
        .controls
        .physics
        .as_mut()
        .unwrap()
        .baked
        .as_mut()
        .unwrap();
    assert_eq!(report.max_error, 0.25);
    assert!(report.cloth.is_some() && report.soft.is_some());
    report.max_error = 0.5;
    model.set_clip(id, clip).unwrap();
    let mut target = RawDocument::new();
    let nodes = BTreeMap::new();
    let meshes = BTreeMap::new();
    let sections = BTreeMap::new();
    let slots = BTreeMap::new();
    let features = BTreeMap::new();
    let stages = stages::Encoded {
        value: Ref::default(),
        stages: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    let routing = routing::WriteContext {
        model: model.model(),
        simulation: targets::WriteContext {
            nodes: &nodes,
            physics: None,
            stages: &stages,
            pipeline: &stages_pipeline(),
        },
        meshes: &meshes,
        sections: &sections,
        slots: &slots,
        features: &features,
    };
    let mut context = Context::export(&ExportOptions::default());
    let output = write_model(
        &mut target,
        model.model(),
        &routing,
        &BTreeMap::new(),
        &mut context,
    )
    .unwrap();
    let (reference, _) = write_clips(
        &mut target,
        output.clips.into_iter().map(|row| (0, row)).collect(),
    )
    .unwrap();
    let row = target
        .resolve::<records::Clip<records::V7>>(reference.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let bake = row.physics.value.unwrap().baked.value.unwrap();
    assert_eq!(bake.max_error, 0.5);
    assert_eq!((bake.inputs, bake.written), (0, 0));
    assert!(context
        .report
        .issues
        .iter()
        .any(|issue| issue.path == "wem.physics.bake.fingerprints"));
}

#[test]
fn aliased_and_unused_declarations_keep_independent_initial_values() {
    use crate::model::semantic::{
        profiles::wem::animation::{AnyDeclaration, NodeOperation, Target},
        Model, Node,
    };
    let mut model = Model::<Wem>::builder("aliases");
    let node = model.add_node(Node::named("target")).unwrap();
    let nodes = [node];
    let mut source = RawDocument::new();
    let first = append(
        &mut source,
        &[
            records::Byte { value: 0 },
            records::Byte { value: 0 },
            records::Byte { value: 128 },
            records::Byte { value: 63 },
            records::Byte { value: 0 },
            records::Byte { value: 0 },
            records::Byte { value: 0 },
            records::Byte { value: 64 },
            records::Byte { value: 0 },
            records::Byte { value: 0 },
            records::Byte { value: 64 },
            records::Byte { value: 64 },
        ],
    )
    .unwrap()
    .cast();
    let rows = append(
        &mut source,
        &[
            records::AnimChannel {
                id: 7,
                target: records::TrackTarget {
                    kind: records::TrackTargetKind::Node,
                    node: 0,
                    channel: records::Channel::Translation,
                    ..Default::default()
                },
                value_type: records::AttrType::F32x3,
                init_value: first,
            },
            records::AnimChannel {
                id: 9,
                target: records::TrackTarget {
                    kind: records::TrackTargetKind::Node,
                    node: 0,
                    channel: records::Channel::Translation,
                    ..Default::default()
                },
                value_type: records::AttrType::F32x3,
                init_value: Ref::default(),
            },
        ],
    )
    .unwrap();
    let stages = stages::Decoded {
        value: Default::default(),
        stages: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    let routing = routing::ReadContext {
        model: model.model(),
        simulation: targets::ReadContext {
            nodes: &nodes,
            physics: None,
            stages: &stages,
        },
        meshes: &[],
        sections: &[],
        slots: &[],
    };
    let (channels, ids) =
        super::declarations::channels(&source, rows.cast(), &routing, &[]).unwrap();
    assert_ne!(ids[&7], ids[&9]);
    assert_eq!(channels.declarations().len(), 2);
    let AnyDeclaration::Vector3(_, first) = channels.declaration(ids[&7]).unwrap() else {
        panic!()
    };
    let AnyDeclaration::Vector3(_, second) = channels.declaration(ids[&9]).unwrap() else {
        panic!()
    };
    assert!(
        matches!((&first.target,&second.target),(Target::Node{node:a,operation:NodeOperation::Translation},Target::Node{node:b,operation:NodeOperation::Translation}) if a==b)
    );
    assert_eq!(first.initial, Some([1., 2., 3.]));
    assert_eq!(second.initial, None);
    assert_eq!(
        model.model().node(node).unwrap().transform.translation,
        [0., 0., 0.]
    );
}

fn rebuild(
    source: &RawDocument<'_>,
    reference: Ref<records::targets::Clip>,
    selections: Ref<records::targets::TrackSet>,
) -> (RawDocument<'static>, Ref<records::targets::Clip>) {
    let mut builder = ModelCollectionBuilder::<Wem>::new();
    builder.add_model("dialect fixture");
    let mut context = Context::import(&ImportOptions::default());
    let prepared = prepare_clips(source, reference, &mut builder, &mut context).unwrap();
    let stages = stages::Decoded {
        value: Default::default(),
        stages: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    let model = builder.model_mut(0).unwrap();
    let routing = routing::ReadContext {
        model: model.model(),
        simulation: targets::ReadContext {
            nodes: &[],
            physics: None,
            stages: &stages,
        },
        meshes: &[],
        sections: &[],
        slots: &[],
    };
    let decoded = decode_model(
        source,
        &prepared,
        super::SourceTables {
            model: 0,
            channels: Ref::default(),
            selections,
        },
        &routing,
        &[],
        &mut context,
    )
    .unwrap();
    install(decoded, model).unwrap();
    let mut target = RawDocument::new();
    let nodes = BTreeMap::new();
    let meshes = BTreeMap::new();
    let sections = BTreeMap::new();
    let slots = BTreeMap::new();
    let features = BTreeMap::new();
    let pipeline = stages_pipeline();
    let stages = stages::Encoded {
        value: Ref::default(),
        stages: BTreeMap::new(),
        sources: BTreeMap::new(),
    };
    let routing = routing::WriteContext {
        model: model.model(),
        simulation: targets::WriteContext {
            nodes: &nodes,
            physics: None,
            stages: &stages,
            pipeline: &pipeline,
        },
        meshes: &meshes,
        sections: &sections,
        slots: &slots,
        features: &features,
    };
    let written = write_model(
        &mut target,
        model.model(),
        &routing,
        &BTreeMap::new(),
        &mut Context::export(&ExportOptions::default()),
    )
    .unwrap();
    let (reference, _) = write_clips(
        &mut target,
        written.clips.into_iter().map(|row| (0, row)).collect(),
    )
    .unwrap();
    (target, reference)
}

#[test]
fn every_historical_clip_vocabulary_rebuilds_declared_fields() {
    use crate::model::wem::InlineOption;
    let mut source = RawDocument::new();
    let reference = append(&mut source, &[records::Clip::<records::V1>::default()]).unwrap();
    let (target, output) = rebuild(&source, reference.cast(), Ref::default());
    assert_eq!(
        target
            .referenced_section(output)
            .unwrap()
            .unwrap()
            .revision(),
        1
    );
    let mut source = RawDocument::new();
    let reference = append(
        &mut source,
        &[records::Clip::<records::V2> {
            bounds: records::Extent {
                sphere_radius: 17.,
                ..Default::default()
            },
            ..Default::default()
        }],
    )
    .unwrap();
    let (target, output) = rebuild(&source, reference.cast(), Ref::default());
    let row = target
        .resolve::<records::Clip<records::V2>>(output.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(row.bounds.sphere_radius, 17.);
    let mut source = RawDocument::new();
    let selection = append(&mut source, &[records::TrackSet::default()]).unwrap();
    let selected = append(
        &mut source,
        &[records::ClipTrackSet {
            set: 0,
            priority: 23,
        }],
    )
    .unwrap();
    let reference = append(
        &mut source,
        &[records::Clip::<records::V3> {
            track_sets: selected.cast(),
            ..Default::default()
        }],
    )
    .unwrap();
    let (target, output) = rebuild(&source, reference.cast(), selection.cast());
    let row = target
        .resolve::<records::Clip<records::V3>>(output.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(
        target
            .resolve::<records::ClipTrackSet>(row.track_sets.cast())
            .unwrap()
            .get(0)
            .unwrap()
            .unwrap()
            .priority,
        23
    );
    let mut source = RawDocument::new();
    let reference = append(
        &mut source,
        &[records::Clip::<records::V4> {
            read_rule: records::ReadRule::Unknown(47),
            ..Default::default()
        }],
    )
    .unwrap();
    let (target, output) = rebuild(&source, reference.cast(), Ref::default());
    assert_eq!(
        target
            .resolve::<records::Clip<records::V4>>(output.cast())
            .unwrap()
            .get(0)
            .unwrap()
            .unwrap()
            .read_rule,
        records::ReadRule::Unknown(47)
    );
    macro_rules! bake {
        ($d:ident,$expected:expr) => {{
            let mut source = RawDocument::new();
            let reference = append(
                &mut source,
                &[records::Clip::<records::$d> {
                    physics: InlineOption::from(Some(records::ClipPhysics {
                        baked: InlineOption::from(Some(records::PhysicsBake {
                            nodes: 53,
                            ..Default::default()
                        })),
                        ..Default::default()
                    })),
                    ..Default::default()
                }],
            )
            .unwrap();
            let (target, output) = rebuild(&source, reference.cast(), Ref::default());
            assert_eq!(
                target
                    .referenced_section(output)
                    .unwrap()
                    .unwrap()
                    .revision(),
                $expected
            );
            let row = target
                .resolve::<records::Clip<records::$d>>(output.cast())
                .unwrap()
                .get(0)
                .unwrap()
                .unwrap();
            assert_eq!(row.physics.value.unwrap().baked.value.unwrap().nodes, 53);
        }};
    }
    bake!(V5, 5);
    bake!(V6, 6);
    bake!(V7, 7);
}
