use wc3::model::profiles::{mdlx, Mdlx, Universal};
use wc3::model::TrailAppearance;
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
use wc3::model::{Curve, DynamicTrack, Model, Time, TimeSource, Track};
use wc3::model::{ParticleEmission, ParticleLifetime, ParticleMotion};

#[test]
fn atlas_speed_keeps_one_shared_identity_through_native_publication() {
    let mut slot_values = SlotValues::default();
    let mut draft = Model::<Mdlx>::builder("atlas animation");
    let node = draft.add_named_node("emitter").unwrap();
    let id = draft.reserve_feature(node);
    let native = slot_values.add(2f32);
    draft
        .set_feature(
            id,
            mdlx::Node::Particle2(mdlx::ParticleEmitter2 {
                motion: ParticleMotion {
                    speed: native,
                    ..SlotConstruct::construct(&mut slot_values)
                },
                emission: ParticleEmission {
                    variation: (),
                    ..SlotConstruct::construct(&mut slot_values)
                },
                lifetime: ParticleLifetime {
                    variation: (),
                    ..SlotConstruct::construct(&mut slot_values)
                },
                ..SlotConstruct::construct(&mut slot_values)
            }),
        )
        .unwrap();
    draft
        .add_global_track(DynamicTrack::Scalar(Track {
            target: native,
            curve: Curve::linear(
                [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                [2f32, 4.],
            )
            .unwrap(),
            clock: TimeSource::Animation,
        }))
        .unwrap();
    let model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap()
    .widen::<Universal>();
    assert_eq!(
        model
            .global_tracks()
            .find(|v| v.clock() == TimeSource::Animation)
            .unwrap()
            .target(),
        native.slot()
    );
    let restored = model.try_narrow::<Mdlx>().unwrap();
    assert_eq!(
        restored
            .global_tracks()
            .filter(|v| v.clock() == TimeSource::Animation)
            .count(),
        1
    );
    assert_eq!(
        restored
            .global_tracks()
            .find(|v| v.clock() == TimeSource::Animation)
            .unwrap()
            .target(),
        native.slot()
    );
}

#[test]
fn common_trail_fields_bind_the_owned_component_and_export_exactly() {
    let mut slot_values = SlotValues::default();
    let mut draft = Model::<Mdlx>::builder("trail animation");
    let node = draft.add_named_node("trail").unwrap();
    let id = draft.reserve_feature(node);
    let opacity = slot_values.add(1f32);
    let _rate = slot_values.add(12f32);
    draft
        .set_feature(
            id,
            mdlx::Node::Ribbon(mdlx::RibbonEmitter {
                appearance: TrailAppearance {
                    opacity,
                    ..SlotConstruct::construct(&mut slot_values)
                },
                ..SlotConstruct::construct(&mut slot_values)
            }),
        )
        .unwrap();
    let mut model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap()
    .widen::<Universal>();
    model
        .edit(|draft| {
            draft.add_global_track(DynamicTrack::Scalar(Track {
                target: opacity,
                curve: Curve::linear(
                    [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                    [1f32, 0.],
                )
                .unwrap(),
                clock: TimeSource::Animation,
            }))
        })
        .unwrap();
    let restored = model.try_narrow::<Mdlx>().unwrap();
    assert_eq!(
        restored
            .global_tracks()
            .filter(|v| v.clock() == TimeSource::Animation)
            .count(),
        1
    );
}

#[test]
fn a_shared_continuous_rate_narrows_on_non_burst_atlas_emitters() {
    let mut slot_values = SlotValues::default();
    let mut draft = Model::<Mdlx>::builder("continuous atlas");
    let node = draft.add_named_node("emitter").unwrap();
    let id = draft.reserve_feature(node);
    let _opacity = slot_values.add(1f32);
    let rate = slot_values.add(12f32);
    draft
        .set_feature(
            id,
            mdlx::Node::Particle2(mdlx::ParticleEmitter2 {
                emission: ParticleEmission {
                    rate,
                    variation: (),
                    ..SlotConstruct::construct(&mut slot_values)
                },
                lifetime: ParticleLifetime {
                    variation: (),
                    ..SlotConstruct::construct(&mut slot_values)
                },
                ..SlotConstruct::construct(&mut slot_values)
            }),
        )
        .unwrap();
    draft
        .add_global_track(DynamicTrack::Scalar(Track {
            target: rate,
            curve: Curve::linear([Time::seconds(0.).unwrap()], [12f32]).unwrap(),
            clock: TimeSource::Animation,
        }))
        .unwrap();
    let model = {
        draft.import_values(&slot_values).unwrap();
        draft.finish()
    }
    .unwrap()
    .widen::<Universal>();
    assert!(model.try_narrow::<Mdlx>().is_ok());
}
