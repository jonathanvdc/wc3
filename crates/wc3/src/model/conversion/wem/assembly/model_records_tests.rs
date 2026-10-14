use super::*;
use crate::model::conversion::wem::hierarchy::poses::rows as decode_rows;

fn declaration() -> EncodedModel {
    EncodedModel {
        name: Ref::default(),
        meshes: Ref::default(),
        nodes: Ref::default(),
        poses: Ref::default(),
        reference_pose: u32::MAX,
        convention: None,
        slots: InlineList::default(),
        channels: Ref::default(),
        animation_set: 0,
        materials: Ref::default(),
        bounds: records::Extent::default(),
        tests: Ref::default(),
        lod: None,
        selected_test: None,
        selections: Ref::default(),
        stages: Ref::default(),
        physics: None,
    }
}

#[test]
fn absent_model_declarations_stay_absent() {
    let mut document = RawDocument::new();
    let reference = write(&mut document, &[declaration()]).unwrap();
    assert_eq!(
        document
            .referenced_section(reference)
            .unwrap()
            .unwrap()
            .revision(),
        1
    );
    let decoded =
        decode_rows::<records::ModelRecord<records::V1>, _>(&document, reference).unwrap();
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded[0].nodes.authoritative_pose, u32::MAX);
}

#[test]
fn a_hierarchy_convention_does_not_invent_lod_or_physics() {
    let mut model = declaration();
    model.convention = Some(records::RigConvention::ExplicitBind);
    let mut document = RawDocument::new();
    let reference = write(&mut document, &[model]).unwrap();
    assert_eq!(
        document
            .referenced_section(reference)
            .unwrap()
            .unwrap()
            .revision(),
        3
    );
    let decoded =
        decode_rows::<records::ModelRecord<records::V3>, _>(&document, reference).unwrap();
    assert_eq!(decoded[0].nodes.rig, records::RigConvention::ExplicitBind);
}

#[test]
fn incompatible_model_declarations_fail_before_mutating_the_document() {
    let absent = declaration();
    let mut present = declaration();
    present.convention = Some(records::RigConvention::ExplicitBind);
    let mut document = RawDocument::new();
    assert!(write(&mut document, &[absent, present]).is_err());
    assert_eq!(document.sections().len(), 0);
}
