//! Borrowed authored sampling and physical simulation inputs shared by effects.
use super::super::events::EventTraversal;
use super::super::pose::PoseContext;
use super::emission::EffectChange;
use bevy::math::Vec3;
use bevy::prelude::*;
use wc3::model::{AnimationSample, SampledAnimation};
#[derive(Clone, Copy)]
pub(crate) struct EffectFrame<'a, 'view, 'data> {
    pub pose: Option<PoseContext<'a>>,
    pub root_visible: bool,
    pub traversal: EventTraversal,
    pub change: EffectChange,
    pub time: f64,
    pub selection: AnimationSample,
    pub sample: &'a SampledAnimation<'view, 'data>,
    /// Current model-space hierarchy matrices, in compiler node order.
    pub node_poses: &'a [Mat4],
    /// Current inherited node visibility, in the same order.
    pub node_visible: &'a [bool],
    pub basis_world: Mat4,
}

impl EffectFrame<'_, '_, '_> {
    /// Current pivot-local feature frame, separate from the geometry skin matrix.
    pub fn feature_matrix(&self, index: usize) -> Mat4 {
        let pivot = self
            .pose
            .map_or(Vec3::ZERO, |pose| pose.nodes[index].rest.pivot);
        self.node_poses[index] * Mat4::from_translation(pivot)
    }
}

/// Initial occurrence frame, used once when a model actor is ready at birth.
pub(crate) struct EffectStart<'a, 'view, 'data> {
    pub frame: EffectFrame<'a, 'view, 'data>,
}
impl EffectStart<'_, '_, '_> {
    pub fn feature(&self, index: usize, scratch: &mut Vec<usize>) -> Mat4 {
        self.sample_feature(index, scratch).0
    }
    pub fn sample_feature(&self, index: usize, scratch: &mut Vec<usize>) -> (Mat4, bool) {
        let (matrix, visible) = self.frame.pose.map_or_else(
            || {
                (
                    self.frame.feature_matrix(index),
                    self.frame.node_visible[index],
                )
            },
            |pose| {
                pose.sample_feature_node(
                    index,
                    &self.frame.sample.at_times(
                        self.frame.selection.clip_time,
                        self.frame.selection.shared_time,
                    ),
                    scratch,
                )
            },
        );
        (self.frame.basis_world * matrix, visible)
    }
}
