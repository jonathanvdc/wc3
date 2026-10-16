//! Affine node evaluation shared by renderers and offline processing.
use crate::model::{Billboard, Inheritance, Transform};
use glam::{Affine3A, EulerRot, Mat4, Quat, Vec3};
use std::f32::consts::FRAC_PI_2;

/// Camera-facing conventions whose native node controls establish their meaning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BillboardConvention {
    #[default]
    /// Generic semantic camera facing.
    Generic,
    /// Warcraft III pivot-local camera facing.
    Warcraft3,
    /// World of Warcraft native billboard frame.
    M2,
}

/// Signed authored factors accompany the affine transform, including collapsed axes.
#[derive(Clone, Copy, Debug)]
pub struct EvaluatedPose {
    /// Full affine mapping from authored model coordinates into the evaluated frame.
    pub matrix: Mat4,
    /// Signed-factor rotation, retained separately from affine shear.
    pub rotation: Quat,
    /// Signed scale factors, including collapsed axes.
    pub scale: Vec3,
    /// Accumulated authored translation used for native inheritance compensation.
    pub motion: Vec3,
    /// Camera offset already applied along this hierarchy chain.
    pub anchor: Vec3,
    /// Current authored pivot used to enter the native bone-local frame.
    pub pivot: Vec3,
}
impl Default for EvaluatedPose {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            motion: Vec3::ZERO,
            anchor: Vec3::ZERO,
            pivot: Vec3::ZERO,
        }
    }
}

/// Camera and inheritance behavior for one node, independent of graph storage.
#[derive(Clone, Copy, Debug, Default)]
pub struct PoseBehavior {
    /// Components inherited from the parent.
    pub inheritance: Inheritance,
    /// Authored camera-facing behavior.
    pub billboard: Billboard,
    /// Native interpretation of camera-facing controls.
    pub billboard_convention: BillboardConvention,
    /// Follow the driving camera position.
    pub camera_anchored: bool,
}
impl PoseBehavior {
    /// Whether ordinary parent/local affine composition suffices.
    #[inline]
    pub fn hierarchical(&self) -> bool {
        self.inheritance.translation
            && self.inheritance.rotation
            && self.inheritance.scale
            && matches!(
                self.billboard,
                Billboard::None | Billboard::Locked { enabled: false, .. }
            )
            && !self.camera_anchored
    }
    /// Evaluates a local pose against its parent without allocation.
    /// A missing camera leaves camera-dependent corrections unapplied.
    #[inline]
    pub fn evaluate(
        &self,
        pose: Transform,
        parent: EvaluatedPose,
        camera: Option<Affine3A>,
    ) -> EvaluatedPose {
        evaluate_pose(self, pose, parent, camera)
    }
}
impl Transform {
    /// Pivot-adjusted local affine matrix, retaining signed and collapsed scales.
    #[inline]
    pub fn to_matrix(self) -> Mat4 {
        let (translation, rotation, scale) = self.local_components();
        Mat4::from_scale_rotation_translation(scale, rotation, translation)
    }
    /// Pivot-adjusted translation, normalized rotation and signed scale, in that order.
    /// Render integrations can publish local TRS without constructing a matrix.
    #[inline]
    pub fn local_components(self) -> (Vec3, Quat, Vec3) {
        let rotation = Quat::from_array(self.rotation.0).normalize();
        let scale = Vec3::from_array(self.scale);
        let pivot = Vec3::from_array(self.pivot);
        let translation = Vec3::from_array(self.translation) + pivot - rotation * (scale * pivot);
        (translation, rotation, scale)
    }

    /// Blends local translation and signed scale linearly and rotation along its shortest arc.
    /// The destination pivot is retained. The weight is clamped to the unit interval.
    #[inline]
    pub fn blend(self, destination: Self, weight: f32) -> Self {
        use crate::model::Quaternion;
        let t = weight.clamp(0., 1.);
        Self {
            translation: Vec3::from_array(self.translation)
                .lerp(Vec3::from_array(destination.translation), t)
                .to_array(),
            rotation: Quaternion(
                Quat::from_array(self.rotation.0)
                    .normalize()
                    .slerp(Quat::from_array(destination.rotation.0).normalize(), t)
                    .to_array(),
            ),
            scale: Vec3::from_array(self.scale)
                .lerp(Vec3::from_array(destination.scale), t)
                .to_array(),
            pivot: destination.pivot,
        }
    }
}
struct LocalPose {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
}
impl LocalPose {
    #[inline]
    fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}
#[inline]
fn safe_ratio(numerator: Vec3, denominator: Vec3) -> Vec3 {
    let divide = |a: f32, b: f32| if b.abs() > 1e-8 { a / b } else { 0. };
    Vec3::new(
        divide(numerator.x, denominator.x),
        divide(numerator.y, denominator.y),
        divide(numerator.z, denominator.z),
    )
}
#[inline]
fn inverse_vector(matrix: Mat4, parent: EvaluatedPose, vector: Vec3) -> Vec3 {
    if matrix.determinant().abs() > 1e-12 {
        matrix.inverse().transform_vector3(vector)
    } else {
        safe_ratio(parent.rotation.inverse() * vector, parent.scale)
    }
}

#[inline]
fn evaluate_pose(
    node: &PoseBehavior,
    pose: Transform,
    parent: EvaluatedPose,
    camera: Option<Affine3A>,
) -> EvaluatedPose {
    if node.billboard_convention == BillboardConvention::Warcraft3 {
        return evaluate_wc3(node, pose, parent, camera);
    }
    let scale = if node.inheritance.scale {
        parent.scale
    } else {
        Vec3::ONE
    };
    let rotation = if node.inheritance.rotation {
        parent.rotation
    } else {
        Quat::IDENTITY
    };
    let translation = if node.inheritance.translation {
        parent.matrix.w_axis.truncate()
    } else {
        Vec3::ZERO
    };
    let inherited =
        if node.inheritance.translation && node.inheritance.rotation && node.inheritance.scale {
            parent.matrix
        } else {
            Mat4::from_scale_rotation_translation(scale, rotation, translation)
        };
    let mut result = EvaluatedPose {
        matrix: inherited * pose.to_matrix(),
        rotation: (rotation * Quat::from_array(pose.rotation.0).normalize()).normalize(),
        scale: scale * Vec3::from_array(pose.scale),
        motion: parent.motion,
        anchor: parent.anchor,
        pivot: Vec3::from_array(pose.pivot),
    };
    if let Some(camera) = camera.filter(|_| {
        node.camera_anchored
            || matches!(
                node.billboard,
                Billboard::Spherical | Billboard::Locked { enabled: true, .. }
            )
    }) {
        let mut anchor = result.matrix.transform_point3(result.pivot);
        if node.camera_anchored {
            anchor += Vec3::from(camera.translation) - parent.anchor;
            result.anchor = camera.translation.into();
        }
        let target = Quat::from_mat3(&camera.matrix3.into()).normalize();
        let next = match node.billboard {
            Billboard::Spherical => {
                let authored = Quat::from_array(pose.rotation.0).normalize();
                if node.billboard_convention == BillboardConvention::M2 {
                    // Native billboard geometry lies in YZ: +X faces the camera,
                    // +Y runs across the image, and +Z points up. Keep authored
                    // animation after this frame conversion, independent of the
                    // inherited rotation that the billboard replaces.
                    let facing = Quat::from_xyzw(-0.5, -0.5, -0.5, 0.5);
                    result.scale = Vec3::new(
                        result
                            .matrix
                            .x_axis
                            .truncate()
                            .length()
                            .copysign(result.scale.x),
                        result
                            .matrix
                            .y_axis
                            .truncate()
                            .length()
                            .copysign(result.scale.y),
                        result
                            .matrix
                            .z_axis
                            .truncate()
                            .length()
                            .copysign(result.scale.z),
                    );
                    target * facing * authored
                } else {
                    target * authored
                }
            }
            Billboard::Locked {
                enabled: true,
                x,
                y,
                z,
            } => {
                let (tx, ty, tz) = target.to_euler(EulerRot::XYZ);
                let (rx, ry, rz) = result.rotation.to_euler(EulerRot::XYZ);
                Quat::from_euler(
                    EulerRot::XYZ,
                    if x { rx } else { tx },
                    if y { ry } else { ty },
                    if z { rz } else { tz },
                )
            }
            _ => result.rotation,
        };
        result.rotation = next;
        result.matrix = Mat4::from_scale_rotation_translation(
            result.scale,
            next,
            anchor - next * (result.scale * result.pivot),
        );
    }
    result
}

// The native evaluator works in pivot-local bone frames. Public semantic matrices
// map model coordinates, so move into and out of that frame without decomposing.
#[inline]
fn evaluate_wc3(
    node: &PoseBehavior,
    pose: Transform,
    parent: EvaluatedPose,
    camera: Option<Affine3A>,
) -> EvaluatedPose {
    let pivot = Vec3::from_array(pose.pivot);
    let parent_frame = parent.matrix * Mat4::from_translation(parent.pivot);
    let authored_translation = Vec3::from_array(pose.translation);
    let mut local = LocalPose {
        translation: pivot - parent.pivot + authored_translation,
        rotation: Quat::from_array(pose.rotation.0).normalize(),
        scale: Vec3::from_array(pose.scale),
    };
    let mut motion = parent.motion;
    let mut anchor = parent.anchor;
    if !node.inheritance.translation {
        local.translation -= inverse_vector(parent_frame, parent, motion);
        motion = Vec3::ZERO;
    }
    motion += parent_frame.transform_vector3(authored_translation);
    if !node.inheritance.scale {
        local.scale *= safe_ratio(Vec3::ONE, parent.scale);
    }
    let full = matches!(node.billboard, Billboard::Spherical);
    if !(node.inheritance.rotation || full && camera.is_some()) {
        local.rotation = parent.rotation.inverse() * local.rotation;
    }
    if let Some(camera) = camera {
        let camera_rotation = Quat::from_mat3(&camera.matrix3.into()).normalize();
        if full {
            let basis = Quat::from_rotation_y(-FRAC_PI_2) * Quat::from_rotation_x(-FRAC_PI_2);
            local.rotation = parent.rotation.inverse() * camera_rotation * basis * local.rotation;
        } else if let Billboard::Locked {
            enabled: true,
            x,
            y,
            z,
        } = node.billboard
        {
            if x || y || z {
                let ray =
                    (parent.rotation * local.rotation).inverse() * (camera_rotation * Vec3::Z);
                let (axis, angle) = if x {
                    local.scale.z *= -1.;
                    (Vec3::X, ray.z.atan2(ray.y))
                } else if y {
                    (Vec3::Y, (-ray.z).atan2(ray.x))
                } else {
                    (Vec3::Z, ray.y.atan2(ray.x))
                };
                if ray.cross(axis).length_squared() > 1e-12 {
                    local.rotation *= Quat::from_axis_angle(axis, angle);
                }
            }
        }
        if node.camera_anchored {
            local.translation += inverse_vector(
                parent_frame,
                parent,
                Vec3::from(camera.translation) - anchor,
            );
            anchor = camera.translation.into();
        }
    }
    EvaluatedPose {
        matrix: parent_frame * local.to_matrix() * Mat4::from_translation(-pivot),
        rotation: (parent.rotation * local.rotation).normalize(),
        scale: parent.scale * local.scale,
        motion,
        anchor,
        pivot,
    }
}
