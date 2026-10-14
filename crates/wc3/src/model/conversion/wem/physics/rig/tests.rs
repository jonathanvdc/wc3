use super::super::codec::append;
use super::{read, write};
use crate::model::semantic::profiles::wem::physics::{
    BeforeActivation, FractureMethod, GrainAxis, InteriorUvs, OpenParts, Physics, PushMode,
    ReleaseOrder, RigStart, SliceAlong,
};
use crate::model::wem::{records, InlineOption, RawDocument, Ref};
use std::collections::BTreeMap;

#[test]
fn old_fracture_preparation_preserves_authored_values_without_inventing_gate_nodes() {
    let mut source = RawDocument::new();
    let seeds = append(
        &mut source,
        &[records::Vector3 {
            value: [2., 3., 4.],
        }],
    )
    .unwrap()
    .cast();
    let rig = records::PhysicsRig::<records::V4> {
        id: 125,
        name: Ref::default(),
        start: records::RigStart::Unknown(231),
        bodies: Ref::default(),
        wow: InlineOption::default(),
        recipe: InlineOption::default(),
        fracture: InlineOption::from(Some(records::FractureRecipe::<records::V4> {
            helper: u32::MAX,
            field: u32::MAX,
            inside: u32::MAX,
            seeds,
            seed: 98,
            grain: 227,
            near_blast: 0.75,
            ..Default::default()
        })),
    };
    let reference = append(&mut source, &[rig]).unwrap().cast();
    let mut physics = Physics::default();
    let ids = read(
        &source,
        reference,
        &mut physics,
        &BTreeMap::new(),
        &[],
        &[],
        &[],
    )
    .unwrap();
    let rig = physics.rig(ids[&125]).unwrap();
    assert_eq!(rig.start, RigStart::Unknown(231));
    let fracture = rig.fracture.as_ref().unwrap();
    assert_eq!(fracture.seeds.get(0), Some([2., 3., 4.]));
    assert_eq!(fracture.seed, 98);
    assert_eq!(fracture.near_blast, 0.75);
    assert_eq!(fracture.strength, 10.);
    assert_eq!(fracture.length, 0.1);
    assert_eq!(fracture.map_size, 1024);
    assert_eq!(fracture.before, BeforeActivation::Pieces);
    assert!(fracture.whole_gate.is_none());
    assert!(fracture.pieces_gate.is_none());
    let mut output = RawDocument::new();
    let mut next = 1;
    let (reference, ids) = write(
        &mut output,
        &physics,
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &mut next,
    )
    .unwrap();
    assert_eq!(ids.len(), 1);
    let row = output
        .resolve::<records::PhysicsRig<records::V8>>(reference.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(row.id, 1);
    let fracture = row.fracture.value.unwrap();
    assert_eq!(fracture.strength, 10.);
    assert_eq!(fracture.length, 0.1);
    assert_eq!(fracture.map_size, 1024);
    assert_eq!(fracture.before, 2);
    assert_eq!(fracture.whole_gate, u32::MAX);
    assert_eq!(fracture.pieces_gate, u32::MAX);
    assert_eq!(fracture.grain, 227);
    let mut reread = Physics::default();
    read(
        &output,
        reference,
        &mut reread,
        &BTreeMap::new(),
        &[],
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(
        reread
            .rigs()
            .next()
            .unwrap()
            .1
            .fracture
            .as_ref()
            .unwrap()
            .seeds
            .get(0),
        Some([2., 3., 4.])
    );
}

#[test]
fn explicit_fracture_zero_values_are_not_replaced_by_historical_defaults() {
    let mut source = RawDocument::new();
    let axes = append(
        &mut source,
        &[records::Byte { value: 3 }, records::Byte { value: 255 }],
    )
    .unwrap()
    .cast();
    let row = records::FractureRecipe::<records::V8> {
        slice_along: axes,
        helper: u32::MAX,
        field: u32::MAX,
        inside: u32::MAX,
        whole_gate: u32::MAX,
        pieces_gate: u32::MAX,
        strength: 0.,
        length: 0.,
        map_size: 0,
        before: 0,
        ..Default::default()
    };
    let decoded = super::fracture::read(&source, &row, &[], &[], &[]).unwrap();
    assert_eq!(decoded.strength, 0.);
    assert_eq!(decoded.length, 0.);
    assert_eq!(decoded.map_size, 0);
    assert_eq!(decoded.before, BeforeActivation::Auto);
    assert_eq!(decoded.grain, GrainAxis::None);
    assert_eq!(decoded.open_parts, OpenParts::ClosedSolid);
    assert_eq!(decoded.method, FractureMethod::Fracture);
    assert_eq!(decoded.push, PushMode::Blast);
    assert_eq!(decoded.release, ReleaseOrder::Together);
    assert_eq!(decoded.filling_uvs, InteriorUvs::Tiled);
    assert_eq!(decoded.slice_along.get(0), Some(SliceAlong::Direction));
    assert_eq!(decoded.slice_along.get(1), Some(SliceAlong::Unknown(255)));
}
