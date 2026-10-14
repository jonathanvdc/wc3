use super::{append, read, write};
use crate::model::semantic::profiles::wem::{PosePipeline, PoseStage, StageKind, StageSource};
use crate::model::semantic::{Scope, Transform};
use crate::model::wem::{records, RawDocument, Ref};
use std::borrow::Cow;
use std::collections::BTreeMap;

#[test]
fn weighted_inputs_and_operations_relocate_without_changing_handles() {
    let node = Scope::default().allocate();
    let mut stage = PoseStage::new(StageKind::Link);
    stage.name = Cow::Borrowed("weighted link");
    stage.driven.push(node);
    let world = stage.add_source(StageSource::new(None, 0.25)).unwrap();
    let local = stage
        .add_source(StageSource::new(Some(node), 0.75))
        .unwrap();
    stage.move_source(world, 1).unwrap();
    let mut pipeline = PosePipeline::default();
    let first = pipeline.add_stage(stage);
    let second = pipeline.add_stage(PoseStage::new(StageKind::Unknown(237)));
    pipeline.move_stage(first, 1).unwrap();
    let mut output = RawDocument::new();
    let encoded = write(
        &mut output,
        &pipeline,
        &BTreeMap::from([(node, 0)]),
        None,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    assert_eq!(encoded.stages[&first], 1);
    assert_eq!(encoded.stages[&second], 0);
    assert_eq!(encoded.sources[&world], 2);
    assert_eq!(encoded.sources[&local], 1);
    let decoded = read(
        &output,
        encoded.value,
        &[node],
        None,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    let stage = decoded.value.stage(decoded.stages[&1]).unwrap();
    assert!(matches!(stage.name, Cow::Borrowed("weighted link")));
    let input = stage.source(decoded.sources[&(1, 2)]).unwrap();
    assert_eq!(input.node, None);
    assert_eq!(input.weight, 0.25);
    let input = stage.source(decoded.sources[&(1, 1)]).unwrap();
    assert_eq!(input.node, Some(node));
    assert_eq!(input.weight, 0.75);
}

#[test]
fn historical_operations_have_no_fabricated_simulation_selection() {
    let mut source = RawDocument::new();
    let offset = Transform::default();
    let rows = [records::PoseStage::<records::V1> {
        id: 117,
        name: Ref::default(),
        up_node: u32::MAX,
        offset: records::Transform {
            translation: offset.translation,
            rotation: offset.rotation.components(),
            scale: offset.scale,
        },
        ..Default::default()
    }];
    let reference = append(&mut source, &rows).unwrap().cast();
    let decoded = read(
        &source,
        reference,
        &[],
        None,
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    let stage = decoded.value.stage(decoded.stages[&117]).unwrap();
    assert!(stage.ragdoll.is_none());
    assert!(stage.cloth.is_none());
}
