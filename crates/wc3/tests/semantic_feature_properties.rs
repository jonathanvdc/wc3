use wc3::model::feature_properties::camera;
use wc3::model::profiles::{m2, mdlx, Extends, Mdlx, Narrows, Universal, UniversalProperty, M2};
use wc3::model::{
    Camera, Channel, Clock, Curve, FeatureProperty, Model, Node, NodeFeature, Time, Track,
};

#[test]
fn vertical_field_of_view_publishes_in_universal_and_cannot_narrow_to_static_mdlx() {
    let mut draft = Model::<Universal>::builder("vertical camera");
    let node = draft.add_node(Node::named("camera")).unwrap();
    let id = draft.reserve_feature(node);
    let mut feature =
        <Universal as Extends<Mdlx>>::feature(mdlx::Node::Camera(mdlx::Camera::default()));
    let NodeFeature::Camera(Camera::Aimed(camera)) = &mut feature else {
        panic!("expected aimed camera")
    };
    camera.field_of_view.binding = Some(camera::VERTICAL_FIELD_OF_VIEW.bind(id).into());
    draft.set_feature(id, feature).unwrap();
    draft
        .add_timeline_channel(Channel::Scalar(Track {
            target: camera::VERTICAL_FIELD_OF_VIEW.bind(id),
            curve: Curve::linear(
                [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                vec![0.5f32, 1.],
            )
            .unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    let model = draft.finish().unwrap();
    assert!(model.try_narrow::<Mdlx>().is_err());
    assert!(
        <M2 as Narrows<Universal>>::property(UniversalProperty::Feature(
            FeatureProperty::CameraVerticalFieldOfView
        ))
        .is_err()
    );
    assert!(
        <Mdlx as Narrows<Universal>>::property(UniversalProperty::Feature(
            FeatureProperty::CameraVerticalFieldOfView
        ))
        .is_err()
    );
}

#[test]
fn diagonal_cameras_reject_vertical_field_of_view_channels() {
    let mut draft = Model::<Universal>::builder("diagonal camera");
    let node = draft.add_node(Node::named("camera")).unwrap();
    let id = draft
        .add_feature(
            node,
            NodeFeature::from(m2::ActorKind::Camera(m2::Camera {
                convention: m2::values::CameraKind::Unknown(0),
                field_of_view: Default::default(),
                clipping: [0.1, 100.],
                target: [0.; 3],
                eye_offset: Default::default(),
                target_offset: Default::default(),
                roll: Default::default(),
            })),
        )
        .unwrap();
    draft
        .add_timeline_channel(Channel::Scalar(Track {
            target: camera::VERTICAL_FIELD_OF_VIEW.bind(id),
            curve: Curve::linear(
                [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()],
                vec![0.5f32, 1.],
            )
            .unwrap(),
            clock: Clock::Timeline,
        }))
        .unwrap();
    assert!(draft.finish().is_err());
}
