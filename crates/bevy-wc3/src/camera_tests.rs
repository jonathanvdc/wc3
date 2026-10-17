use super::*;
use crate::{compile_model, spawn_model};
use bevy::ecs::world::CommandQueue;
use wc3::model::{
    AimedCamera, Camera as ModelCamera, CameraConvention, Channel, Clock, Curve, Model,
    NodeFeature, Time as ModelTime, Track,
};
use wc3::model::{Construct as SlotConstruct, Values as SlotValues};

#[test]
fn binding_samples_current_animation_and_root_without_pose_or_global_update_lag() {
    let mut slot_values = SlotValues::default();
    let mut builder = Model::builder("bound camera");
    let node = builder.add_named_node("camera").unwrap();
    builder
        .add_timeline_channel(Channel::Vector3(Track {
            target: builder.model().node(node).unwrap().transform.translation,
            clock: Clock::Timeline,
            curve: Curve::linear(
                [
                    ModelTime::seconds(0.).unwrap(),
                    ModelTime::seconds(1.).unwrap(),
                ],
                [[0.; 3], [2., 0., 0.]],
            )
            .unwrap(),
        }))
        .unwrap();
    let feature = builder
        .add_feature(
            node,
            NodeFeature::Camera(ModelCamera::Aimed(Box::new(AimedCamera {
                eye_offset: slot_values.add([0.; 3]),
                target: [0., 10., 0.],
                target_offset: slot_values.add([0.; 3]),
                field_of_view: slot_values.add(1.),
                clipping: [0.1, 100.],
                roll: slot_values.add(0.),
                convention: CameraConvention::Vertical(Box::new(SlotConstruct::construct(
                    &mut slot_values,
                ))),
            }))),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let mut meshes = Assets::default();
    let mut binds = Assets::default();
    let mut materials = Assets::default();
    let prepared = compiled.prepare(&mut meshes, &mut binds);
    let mut app = App::new();
    app.add_systems(Update, animate_cameras);
    let mut queue = CommandQueue::default();
    let root = spawn_model(
        &mut Commands::new(&mut queue, app.world()),
        &mut meshes,
        &mut materials,
        &prepared,
        Transform::default(),
    );
    queue.apply(app.world_mut());
    let camera = app
        .world_mut()
        .spawn((Camera3d::default(), CameraBinding::new(root, feature)))
        .id();
    app.world_mut()
        .get_mut::<Playback>(root)
        .unwrap()
        .timeline_time = 1.;
    app.world_mut()
        .get_mut::<Transform>(root)
        .unwrap()
        .translation = Vec3::new(7., 3., 1.);
    // Neither rig evaluation nor transform propagation has run. The camera must
    // nevertheless observe both current authored and application transforms.
    app.update();
    let coordinate = GlobalTransform::from(*app.world().get::<Transform>(root).unwrap())
        .mul_transform(prepared.basis);
    let expected = coordinate.transform_point(Vec3::new(2., 0., 0.));
    assert!(app
        .world()
        .get::<Transform>(camera)
        .unwrap()
        .translation
        .abs_diff_eq(expected, 1e-5));
    let projection = app.world().get::<Projection>(camera).unwrap();
    let Projection::Perspective(projection) = projection else {
        panic!("perspective lost")
    };
    assert_eq!(projection.fov, 1.);
    assert_eq!(projection.near, 0.1);
    assert_eq!(projection.far, 100.);
}

#[test]
fn world_view_rejects_invalid_lens_and_handles_vertical_targets() {
    let view = SampledCamera {
        eye: Vec3::ZERO,
        target: Vec3::Z,
        up: Vec3::Z,
        vertical_field_of_view: 1.,
        clipping: [0.1, 100.],
        visibility: 1.,
    };
    assert!(view
        .world_transform(GlobalTransform::IDENTITY)
        .unwrap()
        .to_matrix()
        .is_finite());
    assert!(view.perspective_projection(f32::NAN).is_none());
    assert!(view.perspective_projection(4.).is_none());
    assert!(SampledCamera {
        clipping: [1., 0.1],
        ..view
    }
    .perspective_projection(1.)
    .is_none());
}
