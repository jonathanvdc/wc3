//! Bevy transport and occurrence-time orchestration for shared model evaluation.
use super::transition::TransitionView;
use crate::compiled::RuntimeNode;
use bevy::math::Affine3A;
use bevy::prelude::*;
use std::collections::HashMap;
pub(crate) use wc3::model::EvaluatedPose;
use wc3::model::{NodeId, RetimedAnimation, Transform as ModelTransform};

pub(super) fn hierarchical(node: &RuntimeNode) -> bool {
    node.correction.is_none() && node.behavior().hierarchical()
}
pub(super) fn evaluate_pose(
    node: &RuntimeNode,
    pose: ModelTransform,
    parent: EvaluatedPose,
    camera: Option<Affine3A>,
) -> EvaluatedPose {
    node.evaluate(pose, parent, camera)
}
pub(super) fn as_transform(pose: EvaluatedPose) -> Transform {
    Transform {
        translation: pose.matrix.w_axis.truncate(),
        rotation: pose.rotation,
        scale: pose.scale,
    }
}
pub(super) fn local_transform(rest: ModelTransform) -> Transform {
    let (translation, rotation, scale) = rest.local_components();
    Transform {
        translation,
        rotation,
        scale,
    }
}

pub(super) fn local_matrix(rest: ModelTransform) -> Mat4 {
    rest.to_matrix()
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
                .unwrap_or_else(|| node.sample(sample));
            let pose = if self.overrides.contains_key(&node.id) {
                pose
            } else {
                self.transition.map_or(pose, |view| view.blend(index, pose))
            };
            evaluated = evaluate_pose(node, pose, evaluated, self.camera);
            visible &= node.sample_visibility(sample);
        }
        (evaluated.matrix, visible)
    }
}

#[cfg(test)]
use wc3::model::Billboard;
#[cfg(test)]
#[path = "pose_tests.rs"]
mod tests;
