//! Model-particle projection, separate from shared birth and motion simulation.
use bevy::math::{Mat3, Mat4, Quat, Vec2, Vec3};
use bevy::transform::components::Transform;

/// Identity remains stable through compaction and changes on every owner reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct ParticleKey {
    pub epoch: u64,
    pub ordinal: u64,
}

/// A finite orientation representable by Bevy's ordinary TRS model instances.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ModelBasis {
    rotation: Quat,
    reflection: Vec3,
    pub approximated: bool,
}
impl ModelBasis {
    pub fn from_frame(frame: Mat4) -> Option<Self> {
        if !frame.is_finite() {
            return None;
        }
        let x = frame.x_axis.truncate().try_normalize()?;
        let y = frame.y_axis.truncate().try_normalize()?;
        let z = frame.z_axis.truncate().try_normalize()?;
        let approximated = x.dot(y).abs() > 1e-5 || x.dot(z).abs() > 1e-5 || y.dot(z).abs() > 1e-5;
        // A Transform cannot retain shear. Orthogonalize while preserving the
        // first axis and determinant sign; report this representational loss.
        let y = (y - x * x.dot(y)).try_normalize()?;
        let cross = x.cross(y);
        let handedness = cross.dot(z);
        if handedness.abs() < 1e-5 {
            return None;
        }
        let rotation = Quat::from_mat3(&Mat3::from_cols(x, y, cross)).normalize();
        Some(Self {
            rotation,
            reflection: Vec3::new(1., 1., handedness.signum()),
            approximated,
        })
    }
    /// Size and twinkle belong to the parent particle; its RGB/alpha do not.
    /// Host units are compiled standalone coordinate units, not root/bone scale.
    pub fn sample(
        self,
        position: Vec3,
        size: Vec2,
        twinkle: f32,
        host_units: Vec3,
        omega: Vec3,
        age: f32,
    ) -> Option<Transform> {
        let scale = Vec3::new(size.x, size.y, (size.x + size.y) * 0.5)
            * twinkle
            * host_units.abs()
            * self.reflection;
        let rotation = self.rotation * Quat::from_scaled_axis(omega * age.max(0.));
        if !position.is_finite() || !scale.is_finite() || !rotation.is_finite() {
            return None;
        }
        Some(Transform {
            translation: position,
            rotation,
            scale,
        })
    }
}

pub(super) fn tumble(minimum: Vec3, maximum: Vec3, unit: Vec3, negate: [bool; 3]) -> Vec3 {
    let value = minimum + (maximum - minimum) * unit;
    value * Vec3::from_array(negate.map(|negate| if negate { -1. } else { 1. }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;
    fn close(a: Mat4, b: Mat4) {
        assert!(a.abs_diff_eq(b, 1e-5), "{a:?} != {b:?}");
    }
    #[test]
    fn coordinate_reflection_is_not_applied_again_as_a_signed_unit_scale() {
        let frame = Mat4::from_scale(Vec3::new(-0.01, 0.01, 0.01));
        let basis = ModelBasis::from_frame(frame).unwrap();
        let output = basis
            .sample(
                Vec3::ZERO,
                Vec2::splat(2.),
                1.,
                Vec3::new(-0.01, 0.01, 0.01),
                Vec3::ZERO,
                0.,
            )
            .unwrap();
        close(
            output.to_matrix(),
            Mat4::from_scale(Vec3::new(-0.02, 0.02, 0.02)),
        );
        let negative_authored = basis
            .sample(
                Vec3::ZERO,
                Vec2::new(-2., 2.),
                1.,
                Vec3::splat(0.01),
                Vec3::ZERO,
                0.,
            )
            .unwrap();
        assert_eq!(negative_authored.scale.x, -0.02);
    }
    #[test]
    fn model_size_ignores_application_and_bone_scale_but_applies_units_once() {
        let root_rotation = Quat::from_rotation_y(0.3);
        let bone_rotation = Quat::from_rotation_z(0.7);
        let host_units = Vec3::splat(0.01);
        for root_scale in [1., 3., 10.] {
            for bone_scale in [1., 2., 5.] {
                let frame = Mat4::from_scale_rotation_translation(
                    Vec3::splat(root_scale * 0.01),
                    root_rotation,
                    Vec3::new(4., 5., 6.),
                ) * Mat4::from_scale_rotation_translation(
                    Vec3::splat(bone_scale),
                    bone_rotation,
                    Vec3::ZERO,
                );
                let basis = ModelBasis::from_frame(frame).unwrap();
                assert!(!basis.approximated);
                let actual = basis
                    .sample(
                        Vec3::new(7., 8., 9.),
                        Vec2::new(2., 4.),
                        0.5,
                        host_units,
                        Vec3::ZERO,
                        0.,
                    )
                    .unwrap();
                close(
                    actual.to_matrix(),
                    Mat4::from_scale_rotation_translation(
                        Vec3::new(0.01, 0.02, 0.015),
                        root_rotation * bone_rotation,
                        Vec3::new(7., 8., 9.),
                    ),
                );
            }
        }
    }
    #[test]
    fn mirrored_and_nonuniform_frames_keep_normalized_axes_without_inherited_size() {
        let rotation = Quat::from_rotation_x(0.6);
        for scale in [
            Vec3::new(-2., 3., 4.),
            Vec3::new(2., -3., -4.),
            Vec3::new(-2., -3., -4.),
        ] {
            let frame = Mat4::from_scale_rotation_translation(scale, rotation, Vec3::ZERO);
            let basis = ModelBasis::from_frame(frame).unwrap();
            let actual = basis
                .sample(Vec3::ZERO, Vec2::ONE, 1., Vec3::ONE, Vec3::ZERO, 0.)
                .unwrap();
            let expected = Mat4::from_cols(
                frame.x_axis.normalize(),
                frame.y_axis.normalize(),
                frame.z_axis.normalize(),
                Mat4::IDENTITY.w_axis,
            );
            close(actual.to_matrix(), expected);
        }
    }
    #[test]
    fn shear_is_explicit_and_collapsed_or_nonfinite_frames_are_rejected() {
        let mut frame = Mat4::IDENTITY;
        frame.y_axis.x = 0.5;
        let basis = ModelBasis::from_frame(frame).unwrap();
        assert!(basis.approximated);
        assert!(basis
            .sample(Vec3::ZERO, Vec2::ONE, 1., Vec3::ONE, Vec3::ZERO, 0.)
            .unwrap()
            .to_matrix()
            .is_finite());
        assert!(ModelBasis::from_frame(Mat4::from_scale(Vec3::new(0., 1., 1.))).is_none());
        assert!(ModelBasis::from_frame(Mat4::NAN).is_none());
    }
    #[test]
    fn tumble_preserves_constant_ranges_and_integrates_radians_about_local_axes() {
        let omega = tumble(
            Vec3::new(0., 0., FRAC_PI_2),
            Vec3::new(0., 0., FRAC_PI_2),
            Vec3::splat(0.7),
            [false; 3],
        );
        assert_eq!(omega, Vec3::Z * FRAC_PI_2);
        assert_eq!(
            tumble(
                Vec3::new(1., 2., 3.),
                Vec3::new(1., 2., 3.),
                Vec3::splat(0.2),
                [false, true, true]
            ),
            Vec3::new(1., -2., -3.)
        );
        let orientation = Quat::from_rotation_y(0.4);
        let basis = ModelBasis::from_frame(Mat4::from_quat(orientation)).unwrap();
        let sampled = basis
            .sample(Vec3::ZERO, Vec2::ONE, 1., Vec3::ONE, omega, 1.)
            .unwrap();
        assert!(sampled
            .rotation
            .abs_diff_eq(orientation * Quat::from_rotation_z(FRAC_PI_2), 1e-5));
    }
}
