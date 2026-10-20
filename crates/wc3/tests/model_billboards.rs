use glam::Mat3;
use glam::{Affine3A, Mat4, Quat, Vec3};
use wc3::model::{
    Billboard, BillboardAxis, BillboardFrame, BillboardScale, EvaluatedPose, PoseBehavior,
    Transform, TransformSemantics,
};

fn lengths(matrix: Mat4) -> Vec3 {
    Vec3::new(
        matrix.x_axis.truncate().length(),
        matrix.y_axis.truncate().length(),
        matrix.z_axis.truncate().length(),
    )
}

#[test]
fn screen_axes_preserve_locked_direction_lengths_and_pivot_under_shear() {
    let parent = EvaluatedPose {
        matrix: Mat4::from_cols(
            Vec3::new(2., 1., 0.).extend(0.),
            Vec3::new(1., 3., 1.).extend(0.),
            Vec3::new(0., 2., 4.).extend(0.),
            Vec3::new(5., -2., 1.).extend(1.),
        ),
        ..Default::default()
    };
    let pose = Transform {
        pivot: Vec3::new(1., 2., 3.),
        rotation: Quat::from_rotation_z(0.4),
        ..Default::default()
    };
    let camera =
        Affine3A::from_rotation_translation(Quat::from_rotation_y(0.6), Vec3::new(5., 6., 7.));
    for semantics in [TransformSemantics::Affine, TransformSemantics::PivotLocal] {
        for (axis, direction) in [
            (BillboardAxis::X, Vec3::X),
            (BillboardAxis::Y, Vec3::Y),
            (BillboardAxis::Z, Vec3::Z),
        ] {
            let behavior = PoseBehavior {
                billboard: Billboard::ScreenAxis(axis),
                transform_semantics: semantics,
                ..Default::default()
            };
            let before = behavior.evaluate(pose, parent, None);
            let after = behavior.evaluate(pose, parent, Some(camera));
            assert!(after.matrix.is_finite());
            assert!(lengths(after.matrix).abs_diff_eq(lengths(before.matrix), 1e-5));
            assert!(after
                .matrix
                .transform_vector3(direction)
                .abs_diff_eq(before.matrix.transform_vector3(direction), 1e-5));
            assert!(after
                .matrix
                .transform_point3(pose.pivot)
                .abs_diff_eq(before.matrix.transform_point3(pose.pivot), 1e-5));
        }
    }
}

#[test]
fn canonical_screen_locks_reconstruct_image_plane_axes() {
    for (axis, expected) in [
        (BillboardAxis::X, Vec3::new(1., -1., -1.)),
        (BillboardAxis::Y, Vec3::new(-1., 1., -1.)),
    ] {
        let behavior = PoseBehavior {
            billboard: Billboard::ScreenAxis(axis),
            ..Default::default()
        };
        let after = behavior.evaluate(
            Transform::default(),
            EvaluatedPose::default(),
            Some(Affine3A::IDENTITY),
        );
        assert!(after.matrix.abs_diff_eq(Mat4::from_scale(expected), 1e-6));
    }
}

#[test]
fn spherical_lengths_keep_local_reflection_and_remove_parent_reflection() {
    let behavior = PoseBehavior {
        billboard: Billboard::Spherical {
            frame: BillboardFrame::Yz,
            scale: BillboardScale::LocalBasisLengths,
        },
        ..Default::default()
    };
    let pose = Transform {
        pivot: Vec3::new(1., 2., 3.),
        ..Default::default()
    };
    let parent = EvaluatedPose {
        matrix: Mat4::from_scale(Vec3::new(-2., 3., 4.)),
        scale: Vec3::new(-2., 3., 4.),
        ..Default::default()
    };
    let before = behavior.evaluate(pose, parent, None);
    let after = behavior.evaluate(pose, parent, Some(Affine3A::IDENTITY));
    assert!(after.matrix.determinant() > 0.);
    assert!(after.scale.abs_diff_eq(Vec3::new(2., 3., 4.), 1e-6));
    assert!(after
        .matrix
        .transform_point3(pose.pivot)
        .abs_diff_eq(before.matrix.transform_point3(pose.pivot), 1e-5));
    let local = Transform {
        scale: parent.scale,
        ..pose
    };
    let local_after = behavior.evaluate(local, EvaluatedPose::default(), Some(Affine3A::IDENTITY));
    assert!(local_after.matrix.determinant() < 0.);
    assert_eq!(local_after.scale, local.scale);
}

#[test]
fn invalid_or_parallel_frames_retain_finite_composed_transform() {
    let behavior = PoseBehavior {
        billboard: Billboard::ScreenAxis(BillboardAxis::Z),
        ..Default::default()
    };
    let pose = Transform {
        translation: Vec3::new(1., 2., 3.),
        ..Default::default()
    };
    let before = behavior.evaluate(pose, EvaluatedPose::default(), None);
    for camera in [Affine3A::IDENTITY, Affine3A::from_scale(Vec3::ZERO)] {
        assert_eq!(
            behavior
                .evaluate(pose, EvaluatedPose::default(), Some(camera))
                .matrix,
            before.matrix
        );
    }
}

#[test]
fn camera_orthonormalization_uses_inverse_affine_and_output_handedness() {
    let view = Mat3::from_rotation_y(0.7)
        * Mat3::from_cols(
            Vec3::new(2., 0., 0.),
            Vec3::new(1., 3., 0.),
            Vec3::new(1., 1., 4.),
        );
    let rotation = Quat::from_rotation_y(-0.7);
    let pose = Transform {
        rotation: Quat::from_rotation_x(0.3),
        ..Default::default()
    };
    let behavior = PoseBehavior {
        billboard: Billboard::ScreenAxis(BillboardAxis::X),
        ..Default::default()
    };
    let a = Affine3A::from_rotation_translation(rotation, Vec3::ZERO);
    let mut b = Affine3A::from_mat3(view.inverse());
    b.translation = Vec3::splat(20.).into();
    assert!(behavior
        .evaluate(pose, EvaluatedPose::default(), Some(a))
        .matrix
        .abs_diff_eq(
            behavior
                .evaluate(pose, EvaluatedPose::default(), Some(b))
                .matrix,
            1e-5
        ));
}
