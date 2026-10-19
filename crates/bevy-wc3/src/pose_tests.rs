use super::*;
use crate::compile_model;
use crate::compiled::RuntimeNode;
use bevy::math::Vec3;
use std::f32::consts::FRAC_PI_2;
use wc3::model::ModelMetadata;
use wc3::model::{AnimationSample, Curve, DynamicTrack, Node, Quat, Time, TimeSource, Track};
use wc3::model::{BillboardAxis, BillboardFrame, BillboardScale, ModelBuilder, TransformSemantics};
use wc3::model::{Construct as SlotConstruct, PropertyValues as SlotValues};
fn runtime_node() -> RuntimeNode {
    let slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    builder.add_named_node("node").unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let mut node = compiled.nodes[0].clone();
    node.transform_semantics = TransformSemantics::PivotLocal;
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
            translation: Vec3::from_array([2., -3., 1.]),
            rotation: Quat::from_array(Quat::from_rotation_z(0.4).to_array()),
            scale: Vec3::from_array(scale),
            pivot: Vec3::from_array([1., 2., 3.]),
        };
        let mut node = runtime_node();
        let parent = evaluate_pose(&node, parent_pose, EvaluatedPose::default(), None);
        for axis in 0..4 {
            node.billboard = if axis == 0 {
                Billboard::Spherical {
                    frame: BillboardFrame::Yz,
                    scale: BillboardScale::SignedFactors,
                }
            } else {
                Billboard::Axial(match axis {
                    1 => BillboardAxis::X,
                    2 => BillboardAxis::Y,
                    _ => BillboardAxis::Z,
                })
            };
            let authored = ModelTransform {
                translation: Vec3::from_array([0.5, 0.2, -0.3]),
                rotation: Quat::from_array(
                    Quat::from_euler(EulerRot::XYZ, 0.2, 0.5, -0.4).to_array(),
                ),
                scale: Vec3::from_array([1., -0.7, 1.3]),
                pivot: Vec3::from_array([2., -1., 4.]),
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
    let mut slot_values = SlotValues::default();
    for (scale_index, scale) in [[-2., 3., -4.], [0., 2., -1.]].into_iter().enumerate() {
        let mut node = runtime_node();
        let parent_pose = ModelTransform {
            translation: Vec3::from_array([2., -3., 1.]),
            rotation: Quat::from_array(Quat::from_rotation_z(0.4).to_array()),
            scale: Vec3::from_array(scale),
            pivot: Vec3::from_array([1., 2., 3.]),
        };
        let parent = evaluate_pose(&node, parent_pose, EvaluatedPose::default(), None);
        for mask in 1..8 {
            node.inheritance.translation = mask & 1 == 0;
            node.inheritance.rotation = mask & 2 == 0;
            node.inheritance.scale = mask & 4 == 0;
            let authored = ModelTransform {
                translation: Vec3::from_array([0.5, 0.2, -0.3]),
                rotation: Quat::from_array(Quat::from_rotation_y(0.2).to_array()),
                pivot: Vec3::from_array([2., -1., 4.]),
                ..SlotConstruct::construct(&mut slot_values)
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
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let parent = builder.add_named_node("parent").unwrap();
    let child = builder
        .add_node(
            Node::named("emitter", &mut slot_values)
                .with_parent(parent)
                .with_transform(
                    ModelTransform {
                        translation: Vec3::from_array([1., 0., 0.]),
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    &mut slot_values,
                )
                .unwrap(),
        )
        .unwrap();
    let time = |value| Time::seconds(value).unwrap();
    builder
        .add_global_track(DynamicTrack::Rotation(Track {
            target: builder.model().node(parent).unwrap().transform.rotation,
            clock: TimeSource::Animation,
            curve: Curve::linear(
                [time(0.), time(1.)],
                [
                    Quat::from_array(Quat::IDENTITY.to_array()),
                    Quat::from_array(Quat::from_rotation_z(FRAC_PI_2).to_array()),
                ],
            )
            .unwrap(),
        }))
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let sample = compiled
        .animation
        .sample(AnimationSample::default())
        .unwrap();
    let retimed = sample.at_times(time(0.5), time(0.5));
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
    let mut slot_values = SlotValues::default();
    let mut node = runtime_node();
    node.transform_semantics = TransformSemantics::Affine;
    node.inheritance.translation = false;
    let parent = EvaluatedPose {
        matrix: Mat4::from_scale(Vec3::new(2., 1., 3.)),
        scale: Vec3::new(2., 1., 3.),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let pose = ModelTransform {
        rotation: Quat::from_array(Quat::from_rotation_z(0.7).to_array()),
        ..SlotConstruct::construct(&mut slot_values)
    };
    let without = evaluate_pose(&node, pose, parent, None);
    let with = evaluate_pose(&node, pose, parent, Some(Affine3A::from_rotation_y(0.4)));
    assert_eq!(with.matrix, without.matrix);
}

#[test]
fn feature_origins_use_node_pivots_without_moving_geometry_coordinates() {
    let mut slot_values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    builder
        .add_node(
            Node::named("feature anchor", &mut slot_values)
                .with_transform(
                    ModelTransform {
                        pivot: Vec3::from_array([0., 0., 2.]),
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    &mut slot_values,
                )
                .unwrap(),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    let sample = compiled
        .animation
        .sample(AnimationSample::default())
        .unwrap();
    let sample = sample.at_times(Time::default(), Time::default());
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
fn native_controls_do_not_select_transform_or_facing_behavior() {
    use wc3::model::controls::model::BoneControls;
    use wc3::model::NodeControls;
    let mut values = SlotValues::default();
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let plain = Node::named("plain", &mut values);
    let mut with_controls = plain.clone();
    with_controls.name = "with controls".into();
    with_controls.controls = Some(NodeControls::M2(BoneControls::default()));
    builder.add_node(plain).unwrap();
    builder.add_node(with_controls).unwrap();
    builder.import_values(&values).unwrap();
    let model = builder.finish().unwrap();
    let compiled = compile_model(&model, |_| None).unwrap();
    assert_eq!(
        compiled.nodes[0].transform_semantics,
        compiled.nodes[1].transform_semantics
    );
    assert_eq!(compiled.nodes[0].billboard, compiled.nodes[1].billboard);
}

#[test]
fn m2_spherical_billboards_face_yz_geometry_toward_rotated_cameras() {
    let mut slot_values = SlotValues::default();
    let mut node = runtime_node();
    node.transform_semantics = TransformSemantics::Affine;
    node.billboard = Billboard::Spherical {
        frame: BillboardFrame::Yz,
        scale: BillboardScale::BasisLengths,
    };
    let pivot = Vec3::new(2., -1., 3.);
    let pose = ModelTransform {
        pivot,
        ..SlotConstruct::construct(&mut slot_values)
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
    let mut slot_values = SlotValues::default();
    let mut node = runtime_node();
    node.transform_semantics = TransformSemantics::Affine;
    node.billboard = Billboard::Spherical {
        frame: BillboardFrame::Yz,
        scale: BillboardScale::BasisLengths,
    };
    let authored_rotation = Quat::from_euler(EulerRot::XYZ, 0.4, -0.3, 0.2);
    let pose = ModelTransform {
        translation: Vec3::from_array([0.5, 0.2, -0.3]),
        rotation: Quat::from_array(authored_rotation.to_array()),
        scale: Vec3::from_array([1., -0.7, 1.3]),
        pivot: Vec3::from_array([2., -1., 4.]),
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
            ..SlotConstruct::construct(&mut slot_values)
        };
        let before = evaluate_pose(&node, pose, parent, None);
        let actual = evaluate_pose(&node, pose, parent, Some(camera));
        assert!(actual.matrix.is_finite());
        assert!(actual.rotation.is_finite());
        assert!(actual.scale.is_finite());
        let pivot = pose.pivot;
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
fn channel_presence_keeps_late_clip_shared_animation_and_static_node_poses() {
    let mut slot_values = SlotValues::default();
    use wc3::model::NodeChannels;
    use wc3::model::{Clip, SharedClock};
    let mut builder = ModelBuilder::new(ModelMetadata::Mdlx(Default::default()));
    let parent = builder.add_named_node("animated parent").unwrap();
    let child = builder
        .add_node(
            Node::named("static child", &mut slot_values)
                .with_parent(parent)
                .with_transform(
                    ModelTransform {
                        translation: Vec3::from_array([0., 1., 0.]),
                        ..SlotConstruct::construct(&mut slot_values)
                    },
                    &mut slot_values,
                )
                .unwrap(),
        )
        .unwrap();
    let times = [Time::seconds(0.).unwrap(), Time::seconds(1.).unwrap()];
    builder
        .add_global_track(DynamicTrack::Vector3(Track {
            target: builder.model().node(parent).unwrap().transform.translation,
            clock: TimeSource::Animation,
            curve: Curve::linear(
                times,
                ([[0.; 3], [2., 0., 0.]])
                    .into_iter()
                    .map(Vec3::from_array)
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        }))
        .unwrap();
    let shared = builder
        .add_clock(SharedClock::Repeating {
            period: Time::seconds(1.).unwrap(),
        })
        .unwrap();
    builder
        .add_global_track(DynamicTrack::Rotation(Track {
            target: builder.model().node(parent).unwrap().transform.rotation,
            clock: TimeSource::Shared(shared),
            curve: Curve::linear(
                times,
                [
                    Quat::default(),
                    Quat::from_array(Quat::from_rotation_z(FRAC_PI_2).to_array()),
                ],
            )
            .unwrap(),
        }))
        .unwrap();
    builder
        .add_clip(Clip::named("initial empty").with_duration(times[1]))
        .unwrap();
    let clip = builder
        .add_clip(Clip::named("late scale").with_duration(times[1]).once())
        .unwrap();
    builder
        .add_track(
            clip,
            DynamicTrack::Vector3(Track {
                target: builder.model().node(parent).unwrap().transform.scale,
                clock: TimeSource::Animation,
                curve: Curve::linear(
                    times,
                    ([[1.; 3], [3.; 3]])
                        .into_iter()
                        .map(Vec3::from_array)
                        .collect::<Vec<_>>(),
                )
                .unwrap(),
            }),
        )
        .unwrap();
    builder
        .add_track(
            clip,
            DynamicTrack::Boolean(Track {
                target: builder.model().node(parent).unwrap().visible,
                clock: TimeSource::Animation,
                curve: Curve::step(times, [true, false]).unwrap(),
            }),
        )
        .unwrap();
    let model = {
        builder.import_values(&slot_values).unwrap();
        builder.finish()
    }
    .unwrap();
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
    let sample = compiled
        .animation
        .sample(AnimationSample {
            clip: Some(clip),
            clip_time: time,
            shared_time: time,
        })
        .unwrap();
    let retimed = sample.at_times(time, time);
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
    let retimed = sample.at_times(Time::seconds(1.).unwrap(), time);
    let (_, visible) = context.sample_node(child_index, &retimed, &mut scratch);
    assert!(
        !visible,
        "animation visibility still propagates through static children"
    );
}
