use super::codec::append;
use super::*;
use crate::model::semantic::profiles::wem::actors::starcraft::{EmitterShape, ParticleFlags};
use crate::model::semantic::Scope;
use crate::model::wem::records::V19;
#[test]
fn particle_values_and_reset_controls_edit_without_encoded_backing() {
    let scope = Scope::default();
    let base = scope.allocate();
    let trail = scope.allocate();
    let ribbon = scope.allocate();
    let slot = scope.allocate();
    let nodes = [scope.allocate()];
    let features = BTreeMap::from([(4, trail), (5, ribbon)]);
    let mut source = RawDocument::new();
    let path = super::super::super::text::write_bytes(&mut source, b"units/robot.m3").unwrap();
    let models = append(
        &mut source,
        &[records::AssetKey {
            group: 7,
            id: 31,
            path,
        }],
    )
    .unwrap()
    .cast();
    let spline_points = append(
        &mut source,
        &[records::Sc2Property {
            init_value: [1., 2., 3.],
            null_value: [4., 5., 6.],
        }],
    )
    .unwrap()
    .cast();
    let row = records::Sc2ParticleEmitterPayload {
        copy_of: u32::MAX,
        material_slot: 0,
        collision_spawn: u32::MAX,
        trail_link: 4,
        ribbon_link: 5,
        models,
        spline_points,
        flags: records::Sc2ParticleFlags::from_bits_retain(0x8000_0005),
        emitter_shape: records::Sc2EmitterShape::Unknown(999),
        initial_speed: records::Sc2Property {
            init_value: 2.,
            null_value: 7.,
        },
        squirt_amount: records::Sc2Property {
            init_value: 16_777_217,
            null_value: u32::MAX,
        },
        splat_projection_index: 12,
        ..Default::default()
    };
    let payload = records::NodePayload::<V19>::Sc2ParticleEmitter(Box::new(row));
    let mut actor = read(&source, &payload, &nodes, &features, &[slot], &[])
        .unwrap()
        .unwrap();
    let Feature::Particle(value) = &mut actor else {
        panic!()
    };
    assert!(value.models.is_borrowed() && value.spline_points.is_borrowed());
    assert_eq!(
        value.models.get(0).unwrap().path.as_ref(),
        b"units/robot.m3"
    );
    assert_eq!(value.trail_link, Some(trail));
    assert_eq!(value.ribbon_link, Some(ribbon));
    assert_eq!(value.initial_speed.reset, 7.);
    assert_eq!(value.flags.bits(), 0x8000_0005);
    assert_eq!(value.emitter_shape, EmitterShape::Unknown(999));
    value.initial_speed.initial = 3.5;
    value.flags = ParticleFlags::from_bits_retain(0xc000_0005);
    let Bound::Vector3(property) = bind_property(
        &mut actor,
        base,
        records::Sc2ParticleProperty::SplinePoint.raw(),
        Some(0),
    )
    .unwrap()
    .unwrap() else {
        panic!()
    };
    let target = property.target().clone();
    assert_eq!(
        property_code(&actor, &target),
        Some((records::Sc2ParticleProperty::SplinePoint.raw(), Some(0)))
    );
    let Bound::Unsigned(property) = bind_property(
        &mut actor,
        base,
        records::Sc2ParticleProperty::SquirtAmount.raw(),
        None,
    )
    .unwrap()
    .unwrap() else {
        panic!()
    };
    assert_eq!(
        property_code(&actor, property.target()),
        Some((records::Sc2ParticleProperty::SquirtAmount.raw(), None))
    );
    let mut output = RawDocument::new();
    let payload: records::NodePayload<V19> = write(
        &mut output,
        &actor,
        &BTreeMap::new(),
        &BTreeMap::from([(trail, 40), (ribbon, 50)]),
        &BTreeMap::from([(slot, 2)]),
        &BTreeMap::new(),
    )
    .unwrap();
    let records::NodePayload::Sc2ParticleEmitter(row) = payload else {
        panic!()
    };
    assert_eq!(row.initial_speed.init_value, 3.5);
    assert_eq!(row.initial_speed.null_value, 7.);
    assert_eq!(row.squirt_amount.init_value, 16_777_217);
    assert_eq!(row.squirt_amount.null_value, u32::MAX);
    assert_eq!(row.trail_link, 40);
    assert_eq!(row.ribbon_link, 50);
    assert_eq!(row.material_slot, 2);
    assert_eq!(row.flags.bits(), 0xc000_0005);
    assert_eq!(row.splat_projection_index, 12);
}
#[test]
fn copy_keeps_only_actual_emission_point_overrides() {
    let scope = Scope::default();
    let source = scope.allocate();
    let id = scope.allocate();
    let document = RawDocument::new();
    let payload = records::NodePayload::<V19>::Sc2ParticleEmitter(Box::new(
        records::Sc2ParticleEmitterPayload {
            copy_of: 3,
            emission_rate: records::Sc2Property {
                init_value: 8.,
                null_value: 2.,
            },
            squirt_amount: records::Sc2Property {
                init_value: 4,
                null_value: 0,
            },
            gravity: 123.,
            ..Default::default()
        },
    ));
    let mut actor = read(
        &document,
        &payload,
        &[],
        &BTreeMap::from([(3, source)]),
        &[],
        &[],
    )
    .unwrap()
    .unwrap();
    let Feature::ParticleCopy(value) = &actor else {
        panic!()
    };
    assert_eq!(value.source, source);
    assert_eq!(value.emission_rate.initial, 8.);
    assert!(bind_property(
        &mut actor,
        id,
        records::Sc2ParticleProperty::InitialSpeed.raw(),
        None
    )
    .unwrap()
    .is_none());
    let Bound::Scalar(property) = bind_property(
        &mut actor,
        id,
        records::Sc2ParticleProperty::EmissionRate.raw(),
        None,
    )
    .unwrap()
    .unwrap() else {
        panic!()
    };
    assert_eq!(
        property_code(&actor, property.target()),
        Some((records::Sc2ParticleProperty::EmissionRate.raw(), None))
    );
    let payload: records::NodePayload<V19> = write(
        &mut RawDocument::new(),
        &actor,
        &BTreeMap::new(),
        &BTreeMap::from([(source, 10)]),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    let records::NodePayload::Sc2ParticleEmitter(value) = payload else {
        panic!()
    };
    assert_eq!(value.copy_of, 10);
    assert_eq!(value.emission_rate.init_value, 8.);
    assert_eq!(value.gravity, 0.);
}
#[test]
fn ribbon_point_properties_borrow_and_keep_independent_initial_reset_values() {
    let scope = Scope::default();
    let node = scope.allocate();
    let id = scope.allocate();
    let mut document = RawDocument::new();
    let points = append(
        &mut document,
        &[records::Sc2RibbonSplinePoint {
            node: 0,
            velocity: records::Sc2Property {
                init_value: 1.,
                null_value: 2.,
            },
            yaw_variation: records::Sc2Variation {
                type_: records::Sc2VariationKind::Unknown(987),
                amplitude: records::Sc2Property {
                    init_value: 3.,
                    null_value: 4.,
                },
                ..Default::default()
            },
            reserved: 0xdead_beef,
            ..Default::default()
        }],
    )
    .unwrap()
    .cast();
    let payload =
        records::NodePayload::<V19>::Sc2RibbonEmitter(Box::new(records::Sc2RibbonEmitterPayload {
            material_slot: u32::MAX,
            spline_points: points,
            ..Default::default()
        }));
    let mut actor = read(&document, &payload, &[node], &BTreeMap::new(), &[], &[])
        .unwrap()
        .unwrap();
    let Feature::Ribbon(value) = &actor else {
        panic!()
    };
    assert!(value.spline_points.is_borrowed());
    assert_eq!(value.spline_points.get(0).unwrap().node, Some(node));
    let Bound::Scalar(property) = bind_property(
        &mut actor,
        id,
        records::Sc2RibbonProperty::SplineYawAmplitude.raw(),
        Some(0),
    )
    .unwrap()
    .unwrap() else {
        panic!()
    };
    assert_eq!(
        property_code(&actor, property.target()),
        Some((
            records::Sc2RibbonProperty::SplineYawAmplitude.raw(),
            Some(0)
        ))
    );
    assert!(bind_property(
        &mut actor,
        id,
        records::Sc2RibbonProperty::SplineVelocity.raw(),
        Some(2)
    )
    .is_err());
    let mut target = RawDocument::new();
    let row: records::NodePayload<V19> = write(
        &mut target,
        &actor,
        &BTreeMap::from([(node, 2)]),
        &BTreeMap::new(),
        &BTreeMap::new(),
        &BTreeMap::new(),
    )
    .unwrap();
    let records::NodePayload::Sc2RibbonEmitter(row) = row else {
        panic!()
    };
    let point = target
        .resolve::<records::Sc2RibbonSplinePoint>(row.spline_points.cast())
        .unwrap()
        .get(0)
        .unwrap()
        .unwrap();
    assert_eq!(point.node, 2);
    assert_eq!(point.reserved, 0xdead_beef);
    assert_eq!(point.velocity.null_value, 2.);
    assert_eq!(point.yaw_variation.type_.raw(), 987);
    assert_eq!(point.yaw_variation.amplitude.null_value, 4.);
}
