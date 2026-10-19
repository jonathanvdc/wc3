use wc3::model::profiles::{m2, mdlx, Mdlx, Universal, M2};
use wc3::model::{
    Camera, Curve, DynamicTrack, FieldOfViewAxis, FromFeature, Light, LightShadows, Model,
    ModelMetadata, NodeFeature, Time, TimeSource, Track,
};

#[test]
fn animated_fov_shared_with_another_consumer_cannot_be_frozen_by_narrowing() {
    let mut draft = Model::<Universal>::builder(ModelMetadata::Mdlx(Default::default()));
    let camera: mdlx::Camera = draft.construct();
    let feature = NodeFeature::from_feature(mdlx::Node::Camera(camera), draft.values_mut());
    let NodeFeature::Camera(Camera::LookAt(camera)) = &feature else {
        panic!("expected look-at camera");
    };
    let fov = camera.projection.field_of_view;
    let mut light: mdlx::Light = draft.construct();
    light.diffuse.intensity = fov;
    let node = draft.add_named_node("camera and light").unwrap();
    draft.add_feature(node, feature).unwrap();
    draft
        .add_feature(
            node,
            NodeFeature::Light(Light::AmbientDiffuse(Box::new(light.into()))),
        )
        .unwrap();
    draft
        .add_global_track(DynamicTrack::Scalar(Track {
            target: fov,
            curve: Curve::linear([Time::default(), Time::seconds(1.).unwrap()], [0.5, 1.]).unwrap(),
            clock: TimeSource::Animation,
        }))
        .unwrap();
    let source = draft.finish().unwrap();
    let error = source.try_narrow::<Mdlx>().unwrap_err();
    assert_eq!(error.path(), "camera.projection.field_of_view");
}

#[test]
fn mdlx_camera_projection_must_have_native_semantics() {
    for with_role in [false, true] {
        let mut draft = Model::<Mdlx>::builder("");
        let mut camera: mdlx::Camera = draft.construct();
        if with_role {
            camera.role = Some(m2::values::CameraKind::Unknown(0));
        } else {
            camera.projection.axis = FieldOfViewAxis::Diagonal;
        }
        let node = draft.add_named_node("camera").unwrap();
        draft.add_feature(node, mdlx::Node::Camera(camera)).unwrap();
        assert!(draft.finish().is_err());
    }
}

#[test]
fn m2_cameras_reject_non_native_projection_visibility_and_lens_parameters() {
    for variant in 0..4 {
        let mut draft = Model::<M2>::builder("");
        let mut camera: m2::Camera = draft.construct();
        camera.projection.axis = FieldOfViewAxis::Diagonal;
        camera.role = Some(m2::values::CameraKind::Unknown(0));
        match variant {
            0 => camera.projection.axis = FieldOfViewAxis::Vertical,
            1 => camera.role = None,
            2 => camera.visibility = Some(draft.add_value(1.)),
            3 => camera.lens.focus_distance = Some(draft.add_value(10.)),
            _ => unreachable!(),
        }
        let node = draft.add_named_node("camera").unwrap();
        draft
            .add_feature(node, m2::ActorKind::Camera(camera))
            .unwrap();
        assert!(draft.finish().is_err());
    }
}

#[test]
fn m2_lights_reject_independent_shadow_and_falloff_parameters() {
    for with_falloff in [false, true] {
        let mut draft = Model::<M2>::builder("");
        let mut light: m2::Light = draft.construct();
        if with_falloff {
            light.falloff = Some([draft.add_value(1.); 3]);
        } else {
            light.shadows = LightShadows {
                casting: Some(true),
                ..Default::default()
            };
        }
        let node = draft.add_named_node("light").unwrap();
        draft
            .add_feature(node, m2::ActorKind::Light(light))
            .unwrap();
        assert!(draft.finish().is_err());
    }
}
