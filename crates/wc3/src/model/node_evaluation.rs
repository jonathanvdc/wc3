//! Explicit node transform and facing evaluation shared by renderers and offline processing.
use crate::model::{
    Billboard, BillboardAxis, BillboardFrame, BillboardScale, EulerLocks, Inheritance, Transform,
    TransformSemantics,
};
use glam::{Affine3A, EulerRot, Mat4, Quat, Vec3};

/// Signed authored factors accompany the affine transform, including collapsed axes.
/// Explicit affine corrections affect `matrix`; these factors retain the authored TRS
/// sample and cannot represent corrective shear. Corrected descendants require full
/// affine inheritance rather than reconstructing a frame from the factors.
#[derive(Clone, Copy, Debug)]
pub struct EvaluatedPose {
    /// Full affine mapping from authored model coordinates into the evaluated frame.
    pub matrix: Mat4,
    /// Signed-factor rotation, retained separately from affine shear.
    pub rotation: Quat,
    /// Signed scale factors, including collapsed axes.
    pub scale: Vec3,
    /// Accumulated authored translation used by pivot-local inheritance compensation.
    pub motion: Vec3,
    /// Camera offset already applied along this hierarchy chain.
    pub anchor: Vec3,
    /// Current authored pivot used to enter the pivot-local bone frame.
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
    /// Transform composition and selective inheritance rules.
    pub transform_semantics: TransformSemantics,
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
            && self.billboard == Billboard::None
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
    if node.transform_semantics == TransformSemantics::PivotLocal {
        return evaluate_pivot_local(node, pose, parent, camera);
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
        motion: if node.inheritance.translation {
            parent.motion
        } else {
            Vec3::ZERO
        } + inherited.transform_vector3(Vec3::from_array(pose.translation)),
        anchor: parent.anchor,
        pivot: Vec3::from_array(pose.pivot),
    };
    if let Some(camera) =
        camera.filter(|_| node.camera_anchored || node.billboard != Billboard::None)
    {
        let mut anchor = result.matrix.transform_point3(result.pivot);
        if node.camera_anchored {
            anchor += Vec3::from(camera.translation) - parent.anchor;
            result.anchor = camera.translation.into();
        }
        let target = Quat::from_mat3(&camera.matrix3.into()).normalize();
        let next = match node.billboard {
            Billboard::Spherical { frame, scale } => {
                if scale == BillboardScale::BasisLengths {
                    result.scale = basis_lengths(result.matrix).copysign(result.scale);
                }
                target * facing_frame(frame) * Quat::from_array(pose.rotation.0).normalize()
            }
            Billboard::Axial(axis) => {
                let correction = axial_rotation(axis, result.rotation, target, &mut result.scale);
                result.rotation * correction
            }
            Billboard::Euler(locks) => euler_facing(locks, result.rotation, target),
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

// Pivot-local composition works in bone frames. Public semantic matrices
// map model coordinates, so move into and out of that frame without decomposing.
#[inline]
fn evaluate_pivot_local(
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
    if !node.inheritance.rotation {
        local.rotation = parent.rotation.inverse() * local.rotation;
    }
    if let Some(camera) = camera {
        let camera_rotation = Quat::from_mat3(&camera.matrix3.into()).normalize();
        match node.billboard {
            Billboard::Spherical { frame, scale } => {
                let before = if scale == BillboardScale::BasisLengths {
                    Some(parent_frame * local.to_matrix())
                } else {
                    None
                };
                local.rotation = parent.rotation.inverse()
                    * camera_rotation
                    * facing_frame(frame)
                    * Quat::from_array(pose.rotation.0).normalize();
                if let Some(before) = before {
                    // Rescale each corrected basis to its pre-facing length. This also
                    // handles shear in a pivot-local parent without decomposing it.
                    let after = parent_frame * local.to_matrix();
                    local.scale *= safe_ratio(basis_lengths(before), basis_lengths(after));
                }
            }
            Billboard::Axial(axis) => {
                let correction = axial_rotation(
                    axis,
                    parent.rotation * local.rotation,
                    camera_rotation,
                    &mut local.scale,
                );
                local.rotation *= correction;
            }
            Billboard::Euler(locks) => {
                local.rotation = parent.rotation.inverse()
                    * euler_facing(locks, parent.rotation * local.rotation, camera_rotation);
            }
            Billboard::None => {}
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

#[inline]
fn facing_frame(frame: BillboardFrame) -> Quat {
    match frame {
        BillboardFrame::Camera => Quat::IDENTITY,
        BillboardFrame::Yz => Quat::from_xyzw(-0.5, -0.5, -0.5, 0.5),
    }
}
#[inline]
fn basis_lengths(matrix: Mat4) -> Vec3 {
    Vec3::new(
        matrix.x_axis.truncate().length(),
        matrix.y_axis.truncate().length(),
        matrix.z_axis.truncate().length(),
    )
}
#[inline]
fn axial_rotation(axis: BillboardAxis, rotation: Quat, camera: Quat, scale: &mut Vec3) -> Quat {
    let ray = rotation.inverse() * (camera * Vec3::Z);
    let (axis, angle) = match axis {
        BillboardAxis::X => {
            scale.z *= -1.;
            (Vec3::X, ray.z.atan2(ray.y))
        }
        BillboardAxis::Y => (Vec3::Y, (-ray.z).atan2(ray.x)),
        BillboardAxis::Z => (Vec3::Z, ray.y.atan2(ray.x)),
    };
    if ray.cross(axis).length_squared() > 1e-12 {
        Quat::from_axis_angle(axis, angle)
    } else {
        Quat::IDENTITY
    }
}
#[inline]
fn euler_facing(locks: EulerLocks, rotation: Quat, camera: Quat) -> Quat {
    let [x, y, z] = locks.axes();
    let (tx, ty, tz) = camera.to_euler(EulerRot::XYZ);
    let (rx, ry, rz) = rotation.to_euler(EulerRot::XYZ);
    Quat::from_euler(
        EulerRot::XYZ,
        if x { rx } else { tx },
        if y { ry } else { ty },
        if z { rz } else { tz },
    )
}
