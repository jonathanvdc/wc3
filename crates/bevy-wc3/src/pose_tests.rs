use super::*;
use crate::compile_model;
use crate::compiled::RuntimeNode;
use std::f32::consts::FRAC_PI_2;
use wc3::model::{AnimationSample, Channel, Clock, Curve, Model, Node, Quaternion, Time, Track};

fn runtime_node() -> RuntimeNode {
    let mut builder = Model::builder("pose reference");
    builder.add_node(Node::named("node")).unwrap();
    let model = builder.finish().unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let mut node = compiled.nodes[0].clone();
    node.billboard_convention = BillboardConvention::Warcraft3;
    node
}

#[path = "pose_reference.rs"]
mod reference;
use reference::{BILLBOARDS, INHERITANCE};

#[test]
fn native_billboards_preserve_authored_rotations_and_signed_or_collapsed_scales() {
    let camera = Transform::from_xyz(7., -9., 4.).with_rotation(Quat::from_euler(
        EulerRot::XYZ,
        0.3,
        -0.6,
        0.8,
    ));
    for (scale_index, scale) in [[-2., 3., -4.], [0., 2., -1.]].into_iter().enumerate() {
        let parent_pose = ModelTransform {
            translation: [2., -3., 1.],
            rotation: Quaternion(Quat::from_rotation_z(0.4).to_array()),
            scale,
            pivot: [1., 2., 3.],
        };
        let mut node = runtime_node();
        let parent = evaluate_pose(&node, parent_pose, EvaluatedPose::default(), None);
        for axis in 0..4 {
            node.billboard = if axis == 0 {
                Billboard::Spherical
            } else {
                Billboard::Locked {
                    enabled: true,
                    x: axis == 1,
                    y: axis == 2,
                    z: axis == 3,
                }
            };
            let authored = ModelTransform {
                translation: [0.5, 0.2, -0.3],
                rotation: Quaternion(Quat::from_euler(EulerRot::XYZ, 0.2, 0.5, -0.4).to_array()),
                scale: [1., -0.7, 1.3],
                pivot: [2., -1., 4.],
            };
            let actual = evaluate_pose(&node, authored, parent, Some(camera.compute_affine()));
            let (matrix, rotation, scale) = BILLBOARDS[scale_index * 4 + axis];
            let expected_matrix = Mat4::from_cols_array(&matrix);
            assert!(actual.matrix.is_finite());
            assert!(
                actual.matrix.abs_diff_eq(expected_matrix, 2e-5),
                "axis {axis}, scale {scale:?}: {:?} != {:?}",
                actual.matrix,
                expected_matrix
            );
            assert!(actual
                .rotation
                .abs_diff_eq(Quat::from_array(rotation), 1e-5));
            assert!(actual.scale.abs_diff_eq(Vec3::from_array(scale), 1e-5));
        }
    }
}

#[test]
fn native_nested_camera_anchors_apply_camera_translation_once() {
    let mut node = runtime_node();
    node.camera_anchored = true;
    let camera = Transform::from_xyz(7., -9., 4.);
    let first = evaluate_pose(
        &node,
        ModelTransform::default(),
        EvaluatedPose::default(),
        Some(camera.compute_affine()),
    );
    let second = evaluate_pose(
        &node,
        ModelTransform::default(),
        first,
        Some(camera.compute_affine()),
    );
    assert!(first
        .matrix
        .w_axis
        .truncate()
        .abs_diff_eq(camera.translation, 1e-6));
    assert!(second
        .matrix
        .w_axis
        .truncate()
        .abs_diff_eq(camera.translation, 1e-6));
}

#[test]
fn native_noninheritance_preserves_expected_pose_with_collapsed_parent_axes() {
    for (scale_index, scale) in [[-2., 3., -4.], [0., 2., -1.]].into_iter().enumerate() {
        let mut node = runtime_node();
        let parent_pose = ModelTransform {
            translation: [2., -3., 1.],
            rotation: Quaternion(Quat::from_rotation_z(0.4).to_array()),
            scale,
            pivot: [1., 2., 3.],
        };
        let parent = evaluate_pose(&node, parent_pose, EvaluatedPose::default(), None);
        for mask in 1..8 {
            node.inheritance.translation = mask & 1 == 0;
            node.inheritance.rotation = mask & 2 == 0;
            node.inheritance.scale = mask & 4 == 0;
            let authored = ModelTransform {
                translation: [0.5, 0.2, -0.3],
                rotation: Quaternion(Quat::from_rotation_y(0.2).to_array()),
                pivot: [2., -1., 4.],
                ..Default::default()
            };
            let actual = evaluate_pose(&node, authored, parent, None);
            let matrix = Mat4::from_cols_array(&INHERITANCE[scale_index * 7 + mask - 1]);
            assert!(actual.matrix.is_finite());
            assert!(
                actual.matrix.abs_diff_eq(matrix, 2e-5),
                "mask {mask}, scale {scale:?}"
            );
        }
    }
}

#[test]
fn birth_pose_follows_parent_rotation_between_frame_endpoints() {
    let mut builder = Model::builder("rotating parent");
    let parent = builder.add_node(Node::named("parent")).unwrap();
    let child = builder
        .add_node(
            Node::named("emitter")
                .with_parent(parent)
                .with_transform(ModelTransform {
                    translation: [1., 0., 0.],
                    ..Default::default()
                })
                .unwrap(),
        )
        .unwrap();
    let time = |value| Time::seconds(value).unwrap();
    builder
        .add_timeline_channel(Channel::Rotation(Track {
            target: parent.rotation::<Universal>(),
            clock: Clock::Timeline,
            curve: Curve::linear(
                [time(0.), time(1.)],
                [
                    Quaternion(Quat::IDENTITY.to_array()),
                    Quaternion(Quat::from_rotation_z(FRAC_PI_2).to_array()),
                ],
            )
            .unwrap(),
        }))
        .unwrap();
    let model = builder.finish().unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let sample = compiled
        .animation
        .sample(AnimationSample::default())
        .unwrap();
    let retimed = sample.at_times(time(0.5), time(0.5), time(0.5));
    let overrides = HashMap::new();
    let context = PoseContext {
        nodes: &compiled.nodes,
        transition: None,
        overrides: &overrides,
        camera: None,
    };
    let mut scratch = Vec::new();
    let index = compiled
        .nodes
        .iter()
        .position(|node| node.id == child)
        .unwrap();
    let (pose, visible) = context.sample_node(index, &retimed, &mut scratch);
    let expected = Vec3::new(0.5f32.sqrt(), 0.5f32.sqrt(), 0.);
    assert!(pose
        .transform_point3(Vec3::ZERO)
        .abs_diff_eq(expected, 1e-5));
    assert!(visible);
    let capacity = scratch.capacity();
    context.sample_node(index, &retimed, &mut scratch);
    assert_eq!(scratch.capacity(), capacity);
}

#[test]
fn camera_selection_preserves_generic_nonbillboard_affine_shear() {
    let mut node = runtime_node();
    node.billboard_convention = BillboardConvention::Generic;
    node.inheritance.translation = false;
    let parent = EvaluatedPose {
        matrix: Mat4::from_scale(Vec3::new(2., 1., 3.)),
        scale: Vec3::new(2., 1., 3.),
        ..Default::default()
    };
    let pose = ModelTransform {
        rotation: Quaternion(Quat::from_rotation_z(0.7).to_array()),
        ..Default::default()
    };
    let without = evaluate_pose(&node, pose, parent, None);
    let with = evaluate_pose(&node, pose, parent, Some(Affine3A::from_rotation_y(0.4)));
    assert_eq!(with.matrix, without.matrix);
}

#[test]
fn feature_origins_use_node_pivots_without_moving_geometry_coordinates() {
    let mut builder = Model::builder("pivot-local feature origin");
    builder
        .add_node(
            Node::named("feature anchor")
                .with_transform(ModelTransform {
                    pivot: [0., 0., 2.],
                    ..Default::default()
                })
                .unwrap(),
        )
        .unwrap();
    let model = builder.finish().unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let sample = compiled
        .animation
        .sample(AnimationSample::default())
        .unwrap();
    let sample = sample.at_times(Time::default(), Time::default(), Time::default());
    let overrides = HashMap::new();
    let context = PoseContext {
        nodes: &compiled.nodes,
        transition: None,
        overrides: &overrides,
        camera: None,
    };
    let mut scratch = Vec::new();
    let (geometry, _) = context.sample_node(0, &sample, &mut scratch);
    let (feature, _) = context.sample_feature_node(0, &sample, &mut scratch);
    assert_eq!(geometry.transform_point3(Vec3::ZERO), Vec3::ZERO);
    assert_eq!(feature.transform_point3(Vec3::ZERO), Vec3::new(0., 0., 2.));
    // Local feature offsets are measured from the pivot rather than the mesh origin.
    assert_eq!(
        feature.transform_point3(Vec3::Z * 3.),
        Vec3::new(0., 0., 5.)
    );
    assert_eq!(feature.transform_point3(Vec3::NEG_Z), Vec3::new(0., 0., 1.));
}

#[test]
fn m2_billboard_convention_is_selected_from_bone_controls() {
    use wc3::model::controls::model::BoneControls;
    use wc3::model::NodeControls;
    let mut builder = Model::builder("M2 bone convention");
    let mut node = Node::named("native billboard")
        .with_controls(Some(NodeControls::M2(BoneControls::default())));
    node.billboard = Billboard::Spherical;
    builder.add_node(node).unwrap();
    let model = builder.finish().unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    assert_eq!(
        compiled.nodes[0].billboard_convention,
        BillboardConvention::M2
    );
}

#[test]
fn m2_spherical_billboards_face_yz_geometry_toward_rotated_cameras() {
    let mut node = runtime_node();
    node.billboard_convention = BillboardConvention::M2;
    node.billboard = Billboard::Spherical;
    let pivot = Vec3::new(2., -1., 3.);
    let pose = ModelTransform {
        pivot: pivot.to_array(),
        ..Default::default()
    };
    for camera_rotation in [
        Quat::IDENTITY,
        Quat::from_euler(EulerRot::XYZ, 0.3, -0.7, 0.8),
    ] {
        let camera = Affine3A::from_rotation_translation(camera_rotation, Vec3::new(4., 6., 8.));
        let actual = evaluate_pose(&node, pose, EvaluatedPose::default(), Some(camera));
        let y = actual.matrix.transform_vector3(Vec3::Y);
        let z = actual.matrix.transform_vector3(Vec3::Z);
        let normal = y.cross(z).normalize();
        assert!(normal.abs_diff_eq(camera_rotation * Vec3::Z, 1e-5));
        assert!(z.abs_diff_eq(camera_rotation * Vec3::Y, 1e-5));
        assert!(y.abs_diff_eq(camera_rotation * Vec3::X, 1e-5));
        assert!(actual
            .matrix
            .transform_point3(pivot)
            .abs_diff_eq(pivot, 1e-5));
    }
}

#[test]
fn m2_spherical_billboards_preserve_authored_rotation_pivot_and_basis_lengths() {
    let mut node = runtime_node();
    node.billboard_convention = BillboardConvention::M2;
    node.billboard = Billboard::Spherical;
    let authored_rotation = Quat::from_euler(EulerRot::XYZ, 0.4, -0.3, 0.2);
    let pose = ModelTransform {
        translation: [0.5, 0.2, -0.3],
        rotation: Quaternion(authored_rotation.to_array()),
        scale: [1., -0.7, 1.3],
        pivot: [2., -1., 4.],
    };
    let camera_rotation = Quat::from_euler(EulerRot::XYZ, -0.2, 0.6, -0.4);
    let camera = Affine3A::from_rotation_translation(camera_rotation, Vec3::new(5., 6., 7.));
    for parent_scale in [Vec3::new(-2., 3., -4.), Vec3::new(0., 2., -1.), Vec3::ZERO] {
        let rotation = Quat::from_rotation_z(0.6);
        let parent = EvaluatedPose {
            matrix: Mat4::from_scale_rotation_translation(
                parent_scale,
                rotation,
                Vec3::new(2., -3., 1.),
            ),
            rotation,
            scale: parent_scale,
            ..Default::default()
        };
        let before = evaluate_pose(&node, pose, parent, None);
        let actual = evaluate_pose(&node, pose, parent, Some(camera));
        assert!(actual.matrix.is_finite());
        assert!(actual.rotation.is_finite());
        assert!(actual.scale.is_finite());
        let pivot = Vec3::from_array(pose.pivot);
        assert!(actual
            .matrix
            .transform_point3(pivot)
            .abs_diff_eq(before.matrix.transform_point3(pivot), 1e-5));
        let facing_x = camera_rotation * Vec3::Z;
        let facing_y = camera_rotation * Vec3::X;
        let facing_z = camera_rotation * Vec3::Y;
        for (axis, signed_scale) in [
            (Vec3::X, actual.scale.x),
            (Vec3::Y, actual.scale.y),
            (Vec3::Z, actual.scale.z),
        ] {
            let authored = authored_rotation * axis;
            let expected = (facing_x * authored.x + facing_y * authored.y + facing_z * authored.z)
                * signed_scale;
            assert!(actual
                .matrix
                .transform_vector3(axis)
                .abs_diff_eq(expected, 1e-5));
            assert!(
                (actual.matrix.transform_vector3(axis).length()
                    - before.matrix.transform_vector3(axis).length())
                .abs()
                    < 1e-5
            );
        }
        let signs = before.scale.to_array().map(f32::is_sign_negative);
        assert_eq!(actual.scale.to_array().map(f32::is_sign_negative), signs);
    }
}

#[test]
fn channel_presence_keeps_late_clip_shared_controller_and_static_node_poses() {
    use wc3::model::NodeChannels;
    use wc3::model::{Clip, Controller, SharedClock};
    let mut builder = Model::builder("node channel inventory");
    let parent = builder.add_node(Node::named("animated parent")).unwrap();
    let child = builder
        .add_node(
            Node::named("static child")
                .with_parent(parent)
                .with_transform(ModelTransform {
                    translation: [0., 1., 0.],
                    ..Default::default()
                })
                .unwrap(),
        )
        .unwrap();
    let times = [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()];
    builder
        .add_timeline_channel(Channel::Vector3(Track {
            target: parent.translation(),
            clock: Clock::Timeline,
            curve: Curve::linear(times, [[0.; 3], [2., 0., 0.]]).unwrap(),
        }))
        .unwrap();
    let shared = builder
        .add_clock(SharedClock::Repeating {
            period: Time::seconds(1.).unwrap(),
        })
        .unwrap();
    builder
        .add_shared_channel(Channel::Rotation(Track {
            target: parent.rotation(),
            clock: Clock::Shared(shared),
            curve: Curve::linear(
                times,
                [
                    Quaternion::default(),
                    Quaternion(Quat::from_rotation_z(FRAC_PI_2).to_array()),
                ],
            )
            .unwrap(),
        }))
        .unwrap();
    builder
        .add_clip(Clip::named("initial empty").with_duration(times[1]))
        .unwrap();
    let clip = builder
        .add_clip(Clip::named("late scale").with_duration(times[1]))
        .unwrap();
    builder
        .add_channel(
            clip,
            Channel::Vector3(Track {
                target: parent.scale(),
                clock: Clock::Clip,
                curve: Curve::linear(times, [[1.; 3], [3.; 3]]).unwrap(),
            }),
        )
        .unwrap();
    let controller = builder.add_controller(Controller::default()).unwrap();
    builder
        .add_controller_channel(
            controller,
            Channel::Boolean(Track {
                target: parent.visibility(),
                clock: Clock::Timeline,
                curve: Curve::step(times, [true, false]).unwrap(),
            }),
        )
        .unwrap();
    let model = builder.finish().unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let parent_index = compiled
        .nodes
        .iter()
        .position(|node| node.id == parent)
        .unwrap();
    let child_index = compiled
        .nodes
        .iter()
        .position(|node| node.id == child)
        .unwrap();
    assert_eq!(
        compiled.nodes[parent_index].channels,
        NodeChannels {
            translation: true,
            rotation: true,
            scale: true,
            visibility: true
        }
    );
    assert_eq!(
        compiled.nodes[child_index].channels,
        NodeChannels::default()
    );
    let time = Time::seconds(0.5).unwrap();
    let controllers = [controller];
    let sample = compiled
        .animation
        .sample(AnimationSample {
            clip: Some(clip),
            controllers: &controllers,
            clip_time: time,
            timeline_time: time,
            shared_time: time,
            ..Default::default()
        })
        .unwrap();
    let retimed = sample.at_times(time, time, time);
    let overrides = HashMap::new();
    let context = PoseContext {
        nodes: &compiled.nodes,
        transition: None,
        overrides: &overrides,
        camera: None,
    };
    let mut scratch = Vec::new();
    let (matrix, visible) = context.sample_node(child_index, &retimed, &mut scratch);
    let expected = Mat4::from_scale_rotation_translation(
        Vec3::splat(2.),
        Quat::from_rotation_z(FRAC_PI_2 * 0.5),
        Vec3::X,
    ) * Mat4::from_translation(Vec3::Y);
    assert!(matrix.abs_diff_eq(expected, 1e-5));
    assert!(visible);
    let end = Time::seconds(1.).unwrap();
    let retimed = sample.at_times(time, end, time);
    let (_, visible) = context.sample_node(child_index, &retimed, &mut scratch);
    assert!(
        !visible,
        "controller visibility still propagates through static children"
    );
}
