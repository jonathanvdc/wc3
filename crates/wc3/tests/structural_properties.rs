use wc3::model::conversion::ExportOptions;
use wc3::model::formats::mdlx::V800;
use wc3::model::io::mdlx;
use wc3::model::profiles::{mdlx as profile, Mdlx};
use wc3::model::{Curve, DynamicTrack, Model, Time, TimeSource, Track};

#[test]
fn native_export_duplicates_curves_for_fields_sharing_a_slot() {
    let mut draft = Model::<Mdlx>::builder("shared intensity");
    let slot = draft.add_value(1f32);
    for name in ["first", "second"] {
        let node = draft.add_named_node(name).unwrap();
        let mut light: profile::Light = draft.construct();
        light.diffuse.intensity = slot;
        draft
            .add_feature(node, profile::Node::Light(light))
            .unwrap();
    }
    draft
        .add_global_track(DynamicTrack::Scalar(Track {
            target: slot,
            curve: Curve::step([Time::default()], [2.]).unwrap(),
            clock: TimeSource::Animation,
        }))
        .unwrap();
    let model = draft.finish().unwrap();
    let native = mdlx::plan::<V800>(&model, &ExportOptions::default())
        .unwrap()
        .execute()
        .unwrap()
        .value;
    let lights: Vec<_> = native.lights().collect();
    assert_eq!(lights.len(), 2);
    for light in lights {
        assert_eq!(light.intensity.track().unwrap().evaluate(0.), Some(2.));
    }
}

#[test]
fn native_profiles_reject_animation_without_a_consuming_field() {
    let mut draft = Model::<Mdlx>::builder("unused output");
    let slot = draft.add_value(1f32);
    draft
        .add_global_track(DynamicTrack::Scalar(Track {
            target: slot,
            curve: Curve::step([Time::default()], [2.]).unwrap(),
            clock: TimeSource::Animation,
        }))
        .unwrap();
    assert!(draft.finish().is_err());
}
