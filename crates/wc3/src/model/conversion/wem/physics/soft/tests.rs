use super::super::codec::{append, vectors, words};
use super::*;
use crate::model::semantic::{
    profiles::wem::physics::{soft::*, Physics},
    Scope, Values,
};
use crate::model::wem::{records, Bool, RawDocument};
#[test]
fn all_soft_families_rebuild_from_borrowed_graph_and_keep_actual_local_targets() {
    let scope = Scope::default();
    let mesh = scope.allocate();
    let nodes = vec![scope.allocate(), scope.allocate(), scope.allocate()];
    let slot = scope.allocate();
    let mut source = RawDocument::new();
    let meshes = words(&mut source, [0]).unwrap();
    let chain = words(&mut source, [0, 2]).unwrap();
    let line = vectors(&mut source, &Values::owned(vec![[1., 2., 3.]])).unwrap();
    let kick_ids = words(&mut source, [17, 18]).unwrap();
    let kick_mesh = words(&mut source, [0, 0]).unwrap();
    let kick_face = words(&mut source, [3, 4]).unwrap();
    let kick_weights = vectors(
        &mut source,
        &Values::owned(vec![[1., 0., 0.], [0., 1., 0.]]),
    )
    .unwrap();
    let kick_direction = vectors(&mut source, &Values::owned(vec![[0., 0., 1.]])).unwrap();
    let kick_strength = leaf::real_records(&mut source, [2., 3.]).unwrap();
    let strand_nodes = words(&mut source, [0, 1, 2]).unwrap();
    let strand_at = vectors(&mut source, &Values::owned(vec![[4., 5., 6.]])).unwrap();
    let strand_width = leaf::real_records(&mut source, [0.125]).unwrap();
    let drop_nodes = words(&mut source, [u32::MAX]).unwrap();
    let name = leaf::write_string(&mut source, "jelly").unwrap();
    let event = leaf::write_string(&mut source, "impact").unwrap();
    let effect = records::SoftEffect {
        id: 19,
        name,
        kind: records::SoftEffectKind::Jiggle,
        enabled: true.into(),
        amount: 0.75,
        everywhere: false.into(),
        drives_jiggle: true.into(),
        seed: 37,
        place: records::SoftPlace {
            mesh: 0,
            face: 3,
            weights: [1., 0., 0.],
            radius: 2.,
        },
        inflate: Some(records::SoftInflate {
            along: records::SoftAlong::from_raw(231),
            toward_place: true.into(),
            round: 0.4,
            rate: 2.,
            depth: 0.3,
            ..Default::default()
        })
        .into(),
        bulge: Some(records::SoftBulge {
            event,
            chain,
            line,
            rise: 0.2,
            fall: 0.7,
            travels: true.into(),
            ..Default::default()
        })
        .into(),
        jiggle: Some(records::SoftJiggle {
            kick_ids,
            kick_mesh,
            kick_face,
            kick_weights,
            kick_direction,
            kick_strength,
            looseness: 3.,
            bounce: 0.8,
            ..Default::default()
        })
        .into(),
        wobble: Some(records::SoftWobble {
            axis: records::SoftAxis::Bone,
            axis_bone: 1,
            speed: 4.,
            size: 2.,
            ..Default::default()
        })
        .into(),
        blobify: Some(records::SoftBlobify {
            roundness: 0.6,
            swell: 0.2,
            keep_volume: true.into(),
        })
        .into(),
        drip: Some(records::SoftDrip {
            material: 0,
            strand_nodes,
            strand_at,
            strand_width,
            drop_nodes,
            length: 2.,
            thickness: 0.1,
            ..Default::default()
        })
        .into(),
        squash: Some(records::SoftSquash {
            axis_bone: u32::MAX,
            stretch: 0.8,
            turning: 0.3,
            ..Default::default()
        })
        .into(),
        melt: Some(records::SoftMelt {
            floor: -3.,
            puddle: 2.,
            backward: true.into(),
            ..Default::default()
        })
        .into(),
        ripple: Some(records::SoftRipple {
            event,
            wavelength: 2.,
            speed: 4.,
            fade_time: 1.,
            ..Default::default()
        })
        .into(),
    };
    let effects = append(&mut source, &[effect]).unwrap().cast();
    let riders = append(
        &mut source,
        &[records::SoftPart {
            mesh: 0,
            material: 0,
        }],
    )
    .unwrap()
    .cast();
    let lumps = append(
        &mut source,
        &[records::SoftLump {
            kind: records::SoftLumpKind::Surface,
            effect: 19,
            rider: 1,
            carrier: 0,
            node: u32::MAX,
            depth: 0.5,
            swings: Bool::from(true),
            ..Default::default()
        }],
    )
    .unwrap()
    .cast();
    let reference = append(
        &mut source,
        &[records::SoftBody {
            id: 11,
            name,
            meshes,
            effects,
            lumps,
            riders,
            drips: u32::MAX,
            next_effect: 300,
            ..Default::default()
        }],
    )
    .unwrap()
    .cast();
    let mut physics = Physics::default();
    let maps = read(
        &source,
        reference,
        &mut physics,
        &BTreeMap::new(),
        &nodes,
        &[mesh],
        &[slot],
    )
    .unwrap();
    let body_id = maps.bodies[&11];
    let body = physics.soft_body(body_id).unwrap();
    assert!(body.meshes.is_borrowed());
    let effect_id = maps.effects[&(11, 19)];
    let lump_id = maps.lumps[&(11, 1)];
    assert_eq!(body.lump(lump_id).unwrap().effect, Some(effect_id));
    let mut effect = body.effect(effect_id).unwrap().clone();
    assert!(effect
        .settings
        .jiggle
        .as_ref()
        .unwrap()
        .impulses
        .is_borrowed());
    assert!(effect.settings.drip.as_ref().unwrap().strands.is_borrowed());
    assert_eq!(
        effect
            .settings
            .jiggle
            .as_ref()
            .unwrap()
            .impulses
            .get(1)
            .unwrap()
            .direction,
        None
    );
    assert_eq!(effect.settings.bulge.as_ref().unwrap().event, "impact");
    effect.amount = 0.375;
    effect.settings.blobify.as_mut().unwrap().swell = 0.9;
    let mut edited = body.clone();
    edited.set_effect(effect_id, effect).unwrap();
    physics.set_soft_body(body_id, edited).unwrap();
    let mut target = RawDocument::new();
    let node_map = nodes
        .iter()
        .enumerate()
        .map(|(i, id)| (*id, i as u32))
        .collect();
    let mut next = 70_000;
    let written = write(
        &mut target,
        &physics,
        super::Inputs {
            colliders: &BTreeMap::new(),
            nodes: &node_map,
            meshes: &BTreeMap::from([(mesh, 0)]),
            slots: &BTreeMap::from([(slot, 0)]),
            reserved_bodies: &BTreeMap::from([(body_id, 7)]),
            reserved_effects: &BTreeMap::from([((body_id, effect_id), 1)]),
        },
        &mut next,
    )
    .unwrap();
    assert_eq!(written.bodies[&body_id], 7);
    assert_eq!(next, 70_000);
    let body = target
        .resolve::<records::SoftBody>(written.reference.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    let effect = target
        .resolve::<records::SoftEffect>(body.effects.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(effect.amount, 0.375);
    assert_eq!(effect.blobify.value.unwrap().swell, 0.9);
    assert_eq!(effect.inflate.value.unwrap().along.raw(), 231);
    assert!(
        effect.bulge.value.is_some()
            && effect.jiggle.value.is_some()
            && effect.wobble.value.is_some()
            && effect.drip.value.is_some()
            && effect.squash.value.is_some()
            && effect.melt.value.is_some()
            && effect.ripple.value.is_some()
    );
    let lump = target
        .resolve::<records::SoftLump>(body.lumps.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(lump.effect, written.effects[&(body_id, effect_id)]);
    assert_eq!(lump.rider, 1);
    assert_eq!(written.lumps[&(body_id, lump_id)], 1);
}
#[test]
fn malformed_parallel_impulses_fail_before_publishing_a_lazy_view() {
    let mut source = RawDocument::new();
    let kick_ids = words(&mut source, [1]).unwrap();
    let value = records::SoftJiggle {
        kick_ids,
        ..Default::default()
    };
    let context = settings::ReadContext {
        document: &source,
        nodes: &[],
        meshes: &[],
        slots: &[],
    };
    let row = records::SoftEffect {
        jiggle: Some(value).into(),
        ..Default::default()
    };
    assert!(settings::read(&context, &row).is_err());
}
#[test]
fn body_local_associations_reject_handles_minted_by_another_body() {
    let effect = || SoftEffect {
        name: "test".into(),
        kind: EffectKind::Jiggle,
        enabled: true,
        amount: 1.,
        everywhere: true,
        drives_jiggle: false,
        seed: 1,
        place: None,
        settings: Settings::default(),
    };
    let mut first = SoftBody::new("first");
    let foreign = first.add_effect(effect());
    let mut second = SoftBody::new("second");
    second.add_effect(effect());
    second.add_lump(SoftLump {
        kind: LumpKind::Surface,
        anchor: [0.; 3],
        rest: [0.; 3],
        depth: 0.,
        clear: 0.,
        carrier: None,
        node: None,
        effect: Some(foreign),
        rider: None,
        swings: false,
        rigid: false,
    });
    assert!(second.validate().is_err());
}
