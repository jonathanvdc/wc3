use super::*;
use crate::animation::pose::{animate_nodes, Wc3NodeCamera};
use crate::schedule::{configure, Wc3Systems};
use std::f32::consts::FRAC_PI_2;
use wc3::model::animation::{Sequence, Track, ValueKeyframe};
use wc3::model::scene::NodeFlags;

fn definition() -> ModelCamera<V1800> {
    let mut camera = ModelCamera::new("Portrait").unwrap();
    camera.position = [0.0, -10.0, 0.0];
    camera.target_position = [0.0; 3];
    camera.field_of_view = 0.8;
    camera.near_clip = 0.5;
    camera.far_clip = 100.0;
    camera
}

fn animation() -> Wc3Animation {
    Wc3Animation {
        sequence: 0,
        elapsed_ms: 0.0,
        speed: 1.0,
        playing: false,
        sequences: vec![Sequence::new("Stand", [1000, 2000]).unwrap()],
        global_sequences: vec![200],
        event_playback: Default::default(),
    }
}

fn vector_track(start: i32, end: i32, global: Option<u32>) -> Track<[f32; 3]> {
    Track::linear(
        vec![
            ValueKeyframe {
                frame: start,
                value: [0.0; 3],
            },
            ValueKeyframe {
                frame: end,
                value: [4.0, 0.0, 0.0],
            },
        ],
        global,
    )
    .unwrap()
}

fn scalar_track(value: f32) -> Track<f32> {
    Track::linear(vec![ValueKeyframe { frame: 1000, value }], None).unwrap()
}

fn scene() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin));
    configure(&mut app);
    // Deliberately register node evaluation first: the sets must establish order.
    app.add_systems(
        PostUpdate,
        animate_nodes.in_set(Wc3Systems::EvaluateNodePoses),
    );
    app.add_systems(
        PostUpdate,
        animate_cameras.in_set(Wc3Systems::AnimateCameras),
    );
    let root = app
        .world_mut()
        .spawn((
            Transform::default(),
            animation(),
            Wc3ModelCameras(vec![definition()]),
        ))
        .id();
    let camera = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            Transform::default(),
            Wc3CameraBinding::new(root, 0),
        ))
        .id();
    (app, root, camera)
}

fn close(actual: Vec3, expected: Vec3) {
    assert!(
        actual.abs_diff_eq(expected, 1e-4),
        "{actual:?} != {expected:?}"
    );
}

#[test]
fn sampling_adds_offsets_and_resolves_sequence_and_global_clocks() {
    let mut camera = definition();
    camera.translation = Some(vector_track(1000, 2000, None));
    camera.target_translation = Some(vector_track(0, 200, Some(0)));
    camera.rotation = Some(scalar_track(0.3));
    let cameras = Wc3ModelCameras(vec![camera]);
    let mut clock = animation();
    clock.elapsed_ms = 450.0;
    let sampled = cameras.sample(0, &clock).unwrap();
    close(sampled.position, Vec3::new(1.8, -10.0, 0.0));
    close(sampled.target, Vec3::X);
    assert_eq!(sampled.roll, 0.3);
    clock.elapsed_ms = 1450.0;
    let looped = cameras.sample(0, &clock).unwrap();
    close(looped.position, sampled.position);
    close(looped.target, sampled.target);
    clock
        .sequences
        .push(Sequence::new("Other", [3000, 4000]).unwrap());
    clock.play(1);
    let other = cameras.sample(0, &clock).unwrap();
    close(other.position, Vec3::from_array(definition().position));
    assert!(cameras.sample(1, &clock).is_none());
}

#[test]
fn roll_is_about_the_view_axis_for_multiple_directions() {
    let mut camera = definition();
    camera.rotation = Some(scalar_track(FRAC_PI_2));
    for eye in [Vec3::NEG_X * 10.0, Vec3::NEG_Y * 10.0, Vec3::NEG_Z * 10.0] {
        camera.position = eye.to_array();
        let cameras = Wc3ModelCameras(vec![camera.clone()]);
        let sampled = cameras.sample(0, &animation()).unwrap();
        let transform = sampled.world_transform(&GlobalTransform::IDENTITY).unwrap();
        let forward = -eye.normalize();
        close(transform.rotation * Vec3::NEG_Z, forward);
        let base_up = if eye.z != 0.0 { Vec3::Y } else { Vec3::Z };
        close(
            transform.rotation * Vec3::Y,
            Quat::from_axis_angle(forward, FRAC_PI_2) * base_up,
        );
    }
}

#[test]
fn transformed_eye_target_and_up_handle_mirrored_nonuniform_roots() {
    let sampled = Wc3ModelCameras(vec![definition()])
        .sample(0, &animation())
        .unwrap();
    let root = GlobalTransform::from(Transform {
        translation: Vec3::new(4.0, 5.0, 6.0),
        rotation: Quat::from_rotation_x(0.4) * Quat::from_rotation_z(0.7),
        scale: Vec3::new(-2.0, 3.0, 4.0),
    });
    let view = sampled.world_transform(&root).unwrap();
    close(view.translation, root.transform_point(sampled.position));
    close(
        view.rotation * Vec3::NEG_Z,
        (root.transform_point(sampled.target) - view.translation).normalize(),
    );
    close(
        view.rotation * Vec3::Y,
        root.affine().transform_vector3(Vec3::Z).normalize(),
    );
    assert_eq!(view.scale, Vec3::ONE);
}

#[test]
fn binding_uses_current_hierarchy_and_preserves_application_camera_settings() {
    let (mut app, root, camera) = scene();
    let parent = app
        .world_mut()
        .spawn(Transform::from_xyz(10.0, 0.0, 0.0))
        .id();
    app.world_mut().entity_mut(root).insert(ChildOf(parent));
    app.world_mut().get_mut::<Camera>(camera).unwrap().is_active = false;
    app.world_mut().get_mut::<Camera>(camera).unwrap().order = 7;
    app.world_mut()
        .get_mut::<Wc3CameraBinding>(camera)
        .unwrap()
        .fov_multiplier = 0.75;
    if let Projection::Perspective(lens) =
        &mut *app.world_mut().get_mut::<Projection>(camera).unwrap()
    {
        lens.aspect_ratio = 2.0;
    }
    app.update();
    close(
        app.world()
            .get::<GlobalTransform>(camera)
            .unwrap()
            .translation(),
        Vec3::new(10.0, -10.0, 0.0),
    );
    app.world_mut()
        .get_mut::<Transform>(parent)
        .unwrap()
        .translation
        .x = 20.0;
    app.update();
    close(
        app.world()
            .get::<GlobalTransform>(camera)
            .unwrap()
            .translation(),
        Vec3::new(20.0, -10.0, 0.0),
    );
    let Projection::Perspective(lens) = app.world().get::<Projection>(camera).unwrap() else {
        panic!()
    };
    assert!((lens.fov - 0.6).abs() < 1e-6);
    assert_eq!((lens.near, lens.far, lens.aspect_ratio), (0.5, 100.0, 2.0));
    assert_eq!(lens.near_clip_plane, Vec4::new(0.0, 0.0, -1.0, -0.5));
    let view = app.world().get::<Camera>(camera).unwrap();
    assert!(!view.is_active);
    assert_eq!(view.order, 7);
}

#[test]
fn rigid_camera_parent_is_compensated_in_current_frame() {
    let (mut app, _, camera) = scene();
    let parent = app
        .world_mut()
        .spawn(Transform {
            translation: Vec3::X * 5.0,
            rotation: Quat::from_rotation_z(0.7),
            scale: Vec3::splat(2.0),
        })
        .id();
    app.world_mut().entity_mut(camera).insert(ChildOf(parent));
    app.update();
    let global = app.world().get::<GlobalTransform>(camera).unwrap();
    close(global.translation(), Vec3::new(0.0, -10.0, 0.0));
    close(global.affine().transform_vector3(Vec3::NEG_Z), Vec3::Y);
    app.world_mut()
        .get_mut::<Transform>(parent)
        .unwrap()
        .translation
        .y = 20.0;
    app.update();
    close(
        app.world()
            .get::<GlobalTransform>(camera)
            .unwrap()
            .translation(),
        Vec3::new(0.0, -10.0, 0.0),
    );
}

#[test]
fn invalid_views_retain_previous_state_and_recover() {
    let (mut app, root, camera) = scene();
    app.update();
    let original = *app.world().get::<Transform>(camera).unwrap();
    for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY, 10.0] {
        app.world_mut()
            .get_mut::<Wc3CameraBinding>(camera)
            .unwrap()
            .fov_multiplier = invalid;
        app.world_mut()
            .get_mut::<Transform>(root)
            .unwrap()
            .translation = Vec3::X;
        app.update();
        assert_eq!(*app.world().get::<Transform>(camera).unwrap(), original);
    }
    app.world_mut()
        .get_mut::<Wc3CameraBinding>(camera)
        .unwrap()
        .fov_multiplier = 1.0;
    app.world_mut()
        .get_mut::<Wc3CameraBinding>(camera)
        .unwrap()
        .camera_index = 10;
    app.update();
    assert_eq!(*app.world().get::<Transform>(camera).unwrap(), original);
    app.world_mut()
        .get_mut::<Wc3CameraBinding>(camera)
        .unwrap()
        .camera_index = 0;
    for field in ["near", "far", "position", "roll"] {
        let mut cameras = app.world_mut().get_mut::<Wc3ModelCameras>(root).unwrap();
        cameras.0[0] = definition();
        match field {
            "near" => cameras.0[0].near_clip = 0.0,
            "far" => cameras.0[0].far_clip = 0.1,
            "position" => cameras.0[0].position = cameras.0[0].target_position,
            "roll" => cameras.0[0].rotation = Some(scalar_track(f32::NAN)),
            _ => unreachable!(),
        }
        app.update();
        assert_eq!(*app.world().get::<Transform>(camera).unwrap(), original);
    }
    app.world_mut().get_mut::<Wc3ModelCameras>(root).unwrap().0[0] = definition();
    app.update();
    close(
        app.world().get::<Transform>(camera).unwrap().translation,
        Vec3::new(1.0, -10.0, 0.0),
    );
}

#[test]
fn removal_and_model_unloading_leave_application_owned_view_alive() {
    let (mut app, root, camera) = scene();
    app.update();
    app.world_mut()
        .entity_mut(camera)
        .remove::<Wc3CameraBinding>();
    app.world_mut()
        .get_mut::<Transform>(camera)
        .unwrap()
        .translation = Vec3::X;
    app.update();
    close(
        app.world().get::<Transform>(camera).unwrap().translation,
        Vec3::X,
    );
    app.world_mut()
        .entity_mut(camera)
        .insert(Wc3CameraBinding::new(root, 0));
    app.world_mut().despawn(root);
    app.update();
    close(
        app.world().get::<Transform>(camera).unwrap().translation,
        Vec3::X,
    );
}

#[test]
fn independent_camera_selection() {
    let (mut app, root, camera) = scene();
    let mut other = definition();
    other.position = [0.0, -20.0, 0.0];
    app.world_mut()
        .get_mut::<Wc3ModelCameras>(root)
        .unwrap()
        .0
        .push(other);
    let second = app
        .world_mut()
        .spawn((Camera3d::default(), Wc3CameraBinding::new(root, 1)))
        .id();
    app.update();
    close(
        app.world().get::<Transform>(camera).unwrap().translation,
        Vec3::NEG_Y * 10.0,
    );
    close(
        app.world().get::<Transform>(second).unwrap().translation,
        Vec3::NEG_Y * 20.0,
    );
}

#[test]
fn camera_animation_drives_billboards_without_a_frame_of_lag() {
    let (mut app, root, camera) = scene();
    app.world_mut()
        .entity_mut(root)
        .insert(Wc3NodeCamera(camera));
    let node = app
        .world_mut()
        .spawn((
            Transform::default(),
            ChildOf(root),
            AnimatedNode {
                root,
                pivot: Vec3::ZERO,
                parent_pivot: Vec3::ZERO,
                flags: NodeFlags(8),
                camera: None,
                translation: None,
                rotation: None,
                scaling: None,
            },
        ))
        .id();
    app.update();
    let first = app.world().get::<Transform>(node).unwrap().rotation;
    app.world_mut().get_mut::<Wc3ModelCameras>(root).unwrap().0[0].rotation =
        Some(scalar_track(0.7));
    app.update();
    let changed = app.world().get::<Transform>(node).unwrap().rotation;
    assert!(!first.abs_diff_eq(changed, 1e-4));
    app.update();
    assert!(changed.abs_diff_eq(app.world().get::<Transform>(node).unwrap().rotation, 1e-4));
}

#[test]
fn rejects_bound_camera_dependencies_and_nonperspective_views() {
    let (mut app, root, camera) = scene();
    app.world_mut().entity_mut(root).insert(ChildOf(camera));
    app.update();
    assert_eq!(
        *app.world().get::<Transform>(camera).unwrap(),
        Transform::default()
    );
    app.world_mut().entity_mut(root).remove::<ChildOf>();
    *app.world_mut().get_mut::<Projection>(camera).unwrap() =
        Projection::Orthographic(OrthographicProjection::default_3d());
    app.update();
    assert_eq!(
        *app.world().get::<Transform>(camera).unwrap(),
        Transform::default()
    );
    *app.world_mut().get_mut::<Projection>(camera).unwrap() =
        Projection::Perspective(PerspectiveProjection::default());
    app.update();
    close(
        app.world().get::<Transform>(camera).unwrap().translation,
        Vec3::NEG_Y * 10.0,
    );
}

#[test]
fn rejects_singular_and_sheared_camera_parent_conversion() {
    let view = Transform::from_rotation(Quat::from_rotation_z(0.7));
    assert!(local_view(
        view,
        GlobalTransform::from(Transform::from_scale(Vec3::ZERO))
    )
    .is_err());
    assert!(local_view(
        view,
        GlobalTransform::from(Transform::from_scale(Vec3::new(2.0, 3.0, 4.0)))
    )
    .is_err());
}
