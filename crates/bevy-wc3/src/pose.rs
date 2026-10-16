//! Shared hierarchy evaluation for live poses and effect occurrence times.
use super::transition::TransitionView;
use crate::compiled::RuntimeNode;
use bevy::math::Affine3A;
use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;
use wc3::model::profiles::Universal;
use wc3::model::{Billboard, NodeId, RetimedAnimation, Transform as ModelTransform};

/// Camera-facing conventions whose native node controls establish their meaning.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum BillboardConvention {
    #[default]
    Generic,
    Warcraft3,
    M2,
}

/// Signed authored factors accompany the affine transform, including collapsed axes.
#[derive(Clone, Copy, Debug)]
pub(super) struct EvaluatedPose {
    pub matrix: Mat4,
    pub rotation: Quat,
    pub scale: Vec3,
    motion: Vec3,
    anchor: Vec3,
    pivot: Vec3,
}
impl EvaluatedPose {
    /// Signed factors remain defined for collapsed axes. Exceptional nodes flatten
    /// into Bevy TRS; affine shear is retained separately for effect sampling.
    pub fn as_transform(self) -> Transform {
        Transform {
            translation: self.matrix.w_axis.truncate(),
            rotation: self.rotation,
            scale: self.scale,
        }
    }
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

pub(super) fn hierarchical(node: &RuntimeNode) -> bool {
    node.inheritance.translation
        && node.inheritance.rotation
        && node.inheritance.scale
        && matches!(
            node.billboard,
            Billboard::None | Billboard::Locked { enabled: false, .. }
        )
        && !node.camera_anchored
}

pub(super) fn local_transform(rest: ModelTransform) -> Transform {
    let rotation = Quat::from_array(rest.rotation.0).normalize();
    let pivot = Vec3::from_array(rest.pivot);
    let scale = Vec3::from_array(rest.scale);
    let translation = Vec3::from_array(rest.translation) + pivot - rotation * (scale * pivot);
    Transform {
        translation,
        rotation,
        scale,
    }
}

pub(super) fn local_matrix(rest: ModelTransform) -> Mat4 {
    local_transform(rest).to_matrix()
}

/// Borrowed inputs for sampling a single node without allocating a full pose.
#[derive(Clone, Copy)]
pub(super) struct PoseContext<'a> {
    pub nodes: &'a [RuntimeNode],
    pub transition: Option<TransitionView<'a>>,
    pub overrides: &'a HashMap<NodeId, ModelTransform>,
    pub camera: Option<Affine3A>,
}
impl PoseContext<'_> {
    pub fn at_offset(mut self, seconds: f64) -> Self {
        self.transition = self.transition.map(|view| view.at_offset(seconds));
        self
    }
    /// Sample a feature's pivot-local frame. Geometry matrices continue mapping
    /// authored model coordinates; a feature origin is the carrying node pivot.
    pub fn sample_feature_node(
        &self,
        index: usize,
        sample: &RetimedAnimation<'_, '_, '_>,
        scratch: &mut Vec<usize>,
    ) -> (Mat4, bool) {
        let (matrix, visible) = self.sample_node(index, sample, scratch);
        (
            matrix * Mat4::from_translation(Vec3::from_array(self.nodes[index].rest.pivot)),
            visible,
        )
    }

    /// Scratch retains the largest ancestor chain across all births in an instance.
    pub fn sample_node(
        &self,
        index: usize,
        sample: &RetimedAnimation<'_, '_, '_>,
        scratch: &mut Vec<usize>,
    ) -> (Mat4, bool) {
        scratch.clear();
        let mut next = Some(index);
        while let Some(index) = next {
            scratch.push(index);
            next = self.nodes[index].parent;
        }
        let mut evaluated = EvaluatedPose::default();
        let mut visible = true;
        for &index in scratch.iter().rev() {
            let node = &self.nodes[index];
            let pose = self
                .overrides
                .get(&node.id)
                .copied()
                .unwrap_or_else(|| ModelTransform {
                    translation: if node.channels.translation {
                        sample
                            .property(&node.id.translation::<Universal>())
                            .unwrap_or(node.rest.translation)
                    } else {
                        node.rest.translation
                    },
                    rotation: if node.channels.rotation {
                        sample
                            .property(&node.id.rotation::<Universal>())
                            .unwrap_or(node.rest.rotation)
                    } else {
                        node.rest.rotation
                    },
                    scale: if node.channels.scale {
                        sample
                            .property(&node.id.scale::<Universal>())
                            .unwrap_or(node.rest.scale)
                    } else {
                        node.rest.scale
                    },
                    pivot: node.rest.pivot,
                });
            let pose = if self.overrides.contains_key(&node.id) {
                pose
            } else {
                self.transition.map_or(pose, |view| view.blend(index, pose))
            };
            evaluated = evaluate_pose(node, pose, evaluated, self.camera);
            visible &= if node.channels.visibility {
                sample
                    .property(&node.id.visibility::<Universal>())
                    .unwrap_or(node.visible)
            } else {
                node.visible
            };
        }
        (evaluated.matrix, visible)
    }
}

fn safe_ratio(numerator: Vec3, denominator: Vec3) -> Vec3 {
    let divide = |a: f32, b: f32| if b.abs() > 1e-8 { a / b } else { 0. };
    Vec3::new(
        divide(numerator.x, denominator.x),
        divide(numerator.y, denominator.y),
        divide(numerator.z, denominator.z),
    )
}
fn inverse_vector(matrix: Mat4, parent: EvaluatedPose, vector: Vec3) -> Vec3 {
    if matrix.determinant().abs() > 1e-12 {
        matrix.inverse().transform_vector3(vector)
    } else {
        safe_ratio(parent.rotation.inverse() * vector, parent.scale)
    }
}

pub(super) fn evaluate_pose(
    node: &RuntimeNode,
    pose: ModelTransform,
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
        matrix: inherited * local_matrix(pose),
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
fn evaluate_wc3(
    node: &RuntimeNode,
    pose: ModelTransform,
    parent: EvaluatedPose,
    camera: Option<Affine3A>,
) -> EvaluatedPose {
    let pivot = Vec3::from_array(pose.pivot);
    let parent_frame = parent.matrix * Mat4::from_translation(parent.pivot);
    let authored_translation = Vec3::from_array(pose.translation);
    let mut local = Transform {
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

#[cfg(test)]
#[path = "pose_tests.rs"]
mod tests;
