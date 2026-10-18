use wc3::model::profiles::{m2, mdlx, Mdlx, Universal, M2};
use wc3::model::{
    AnimationSample, Camera, Channel, Clip, Construct, Curve, FromFeature, Model, NodeFeature,
    PropertyValues, Time, TimeSource, Track,
};

#[test]
fn vertical_fov_animation_cannot_narrow_to_a_static_native_field() {
    let mut values = PropertyValues::default();
    let camera: mdlx::Camera = Construct::construct(&mut values);
    let mut feature = NodeFeature::from_feature(mdlx::Node::Camera(camera), &mut values);
    let NodeFeature::Camera(Camera::Aimed(camera)) = &mut feature else {
        panic!()
    };
    let fov = camera.field_of_view;
    let mut draft = Model::<Universal>::builder("vertical camera");
    draft.import_values(&values).unwrap();
    let node = draft.add_named_node("camera").unwrap();
    draft.add_feature(node, feature).unwrap();
    draft
        .add_global_channel(Channel::Scalar(Track {
            target: fov,
            curve: Curve::linear([Time::default(), Time::seconds(1.).unwrap()], [0.5f32, 1.])
                .unwrap(),
            clock: TimeSource::Timeline,
        }))
        .unwrap();
    assert!(draft.finish().unwrap().try_narrow::<Mdlx>().is_err());
}

#[test]
fn a_fov_slot_keeps_its_diagonal_camera_convention() {
    let mut draft = Model::<M2>::builder("diagonal camera");
    let mut camera: m2::Camera = draft.construct();
    camera.convention = m2::values::CameraKind::Unknown(0);
    camera.clipping = [0.1, 100.];
    let fov = camera.field_of_view;
    let node = draft.add_named_node("camera").unwrap();
    let feature = draft
        .add_feature(node, m2::ActorKind::Camera(camera))
        .unwrap();
    let clip = draft
        .add_clip(Clip::named("lens").with_duration(Time::seconds(1.).unwrap()))
        .unwrap();
    draft
        .add_channel(
            clip,
            Channel::Scalar(Track {
                target: fov,
                curve: Curve::step([Time::default()], [0.75f32]).unwrap(),
                clock: TimeSource::Clip,
            }),
        )
        .unwrap();
    let universal = draft.finish().unwrap().widen::<Universal>();
    let NodeFeature::Camera(Camera::Aimed(camera)) = &universal.feature(feature).unwrap().payload
    else {
        panic!()
    };
    assert_eq!(camera.field_of_view, fov);
    let sample = universal
        .animation_sample(AnimationSample {
            clip: Some(clip),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(sample.animated(&fov), 0.75);
    assert!(universal.try_narrow::<M2>().is_ok());
}
