//! Shared joint ordering and pivot-derived inverse bind matrices.
use bevy::prelude::{Mat4, Vec3};
use wc3::model::{Model, V1800};

pub(crate) fn joint_ids(model: &Model<V1800>) -> Vec<u32> {
    let mut ids: Vec<_> = model
        .bones()
        .iter()
        .map(|bone| bone.node.object_id)
        .collect();
    ids.sort_unstable();
    ids
}

pub(super) fn inverse_bind_matrices(model: &Model<V1800>, joint_ids: &[u32]) -> Vec<Mat4> {
    let pivots = model.pivot_points();
    joint_ids
        .iter()
        .map(|id| {
            let pivot = pivots.get(*id as usize).copied().unwrap_or([0.0; 3]);
            Mat4::from_translation(-Vec3::from_array(pivot))
        })
        .collect()
}
