//! Borrowed authored sampling and physical simulation inputs shared by effects.
use super::super::events::EventTraversal;
use super::super::pose::PoseContext;
use super::emission::EffectChange;
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
    pub camera_world: Option<Mat4>,
}

impl EffectFrame<'_, '_, '_> {
    /// Current pivot-local feature frame, separate from the geometry skin matrix.
    pub fn feature_matrix(&self, index: usize) -> Mat4 {
        let pivot = self.pose.map_or(Vec3::ZERO, |pose| {
            Vec3::from_array(pose.nodes[index].rest.pivot)
        });
        self.node_poses[index] * Mat4::from_translation(pivot)
    }
}
