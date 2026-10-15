use wc3::model::semantic::feature_properties::{particle, trail};
use wc3::model::semantic::profiles::{mdlx, Mdlx, Universal};
use wc3::model::semantic::{Animated, Channel, Clock, Curve, Model, Node, Time, Track};

#[test]
fn atlas_speed_keeps_one_shared_identity_through_native_publication() {
    let mut draft = Model::<Mdlx>::builder("atlas animation");
    let node = draft.add_node(Node::named("emitter")).unwrap();
    let id = draft.reserve_feature(node);
    let native = mdlx::properties::particle2::SPEED.bind(id);
    draft
        .set_feature(
            id,
            mdlx::Node::Particle2(mdlx::Particle2 {
                speed: Animated::new(2., native.clone()),
                ..Default::default()
            }),
        )
        .unwrap();
    draft
        .add_timeline_channel(Channel::Scalar(Track {
            target: native.clone(),
            curve: Curve::linear(
                [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                [2f32, 4.],
            )
            .unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    let model = draft.finish().unwrap().widen::<Universal>();
    assert_eq!(native.widen::<Universal>(), particle::SPEED.bind(id));
    let restored = model.try_narrow::<Mdlx>().unwrap();
    assert_eq!(restored.timeline_channels().len(), 1);
    assert_eq!(
        restored.timeline_channels()[0].target(),
        particle::SPEED.bind(id).target()
    );
}

#[test]
fn common_trail_fields_bind_the_owned_component_and_export_exactly() {
    let mut draft = Model::<Mdlx>::builder("trail animation");
    let node = draft.add_node(Node::named("trail")).unwrap();
    let id = draft.reserve_feature(node);
    draft
        .set_feature(
            id,
            mdlx::Node::Ribbon(mdlx::Ribbon {
                opacity: Animated::new(1., mdlx::properties::ribbon::OPACITY.bind(id)),
                ..Default::default()
            }),
        )
        .unwrap();
    let mut model = draft.finish().unwrap().widen::<Universal>();
    model
        .edit(|draft| {
            draft.add_timeline_channel(Channel::Scalar(Track {
                target: trail::OPACITY.bind(id),
                curve: Curve::linear(
                    [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                    [1f32, 0.],
                )
                .unwrap(),
                clock: Clock::Timeline,
            }))
        })
        .unwrap();
    let restored = model.try_narrow::<Mdlx>().unwrap();
    assert_eq!(restored.timeline_channels().len(), 1);
}

#[test]
fn a_shared_continuous_rate_narrows_on_non_burst_atlas_emitters() {
    let mut draft = Model::<Mdlx>::builder("continuous atlas");
    let node = draft.add_node(Node::named("emitter")).unwrap();
    let id = draft.reserve_feature(node);
    draft
        .set_feature(
            id,
            mdlx::Node::Particle2(mdlx::Particle2 {
                emission_rate: Animated::new(12., particle::CONTINUOUS_RATE.bind(id)),
                ..Default::default()
            }),
        )
        .unwrap();
    draft
        .add_timeline_channel(Channel::Scalar(Track {
            target: particle::CONTINUOUS_RATE
                .bind(id)
                .try_narrow::<Mdlx>()
                .unwrap(),
            curve: Curve::linear([Time::seconds(0.).unwrap()], [12f32]).unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    let model = draft.finish().unwrap().widen::<Universal>();
    assert!(model.try_narrow::<Mdlx>().is_ok());
}

#[test]
fn changing_gravity_representation_removes_the_old_scalar_destination() {
    use wc3::model::semantic::profiles::FeaturePayload;
    use wc3::model::semantic::{NodeFeature, ParticleDefinition, ParticleGravity, PropertyPayload};
    let mut definition = ParticleDefinition::from(mdlx::Particle2::default());
    let key = mdlx::properties::Target::Particle2(mdlx::properties::Particle2Field::Gravity)
        .identity()
        .to_owned();
    assert!(NodeFeature::from(definition.clone()).supports_property(&key));
    definition.motion.gravity = ParticleGravity::Vector(Animated::constant([0., 0., -9.8]));
    assert!(!NodeFeature::from(definition).supports_property(&key));
}
