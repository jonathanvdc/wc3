//! Shared joint ordering and pivot-derived inverse bind matrices.
use super::shared::RuntimeNode;
use bevy::prelude::{Mat4, Vec3};
use std::collections::{HashMap, VecDeque};
use wc3::model::mdlx::{Model, V1800};

pub(crate) fn joint_ids<'data>(model: &Model<'data, V1800>) -> Vec<u32> {
    let mut ids: Vec<_> = model.bones().map(|bone| bone.node.object_id).collect();
    ids.sort_unstable();
    ids
}

pub(super) fn inverse_bind_matrices<'data>(
    model: &Model<'data, V1800>,
    joint_ids: &[u32],
) -> Vec<Mat4> {
    let pivots = model.pivot_points();
    joint_ids
        .iter()
        .map(|id| {
            let pivot = pivots.get(*id as usize).copied().unwrap_or([0.0; 3]);
            Mat4::from_translation(-Vec3::from_array(pivot))
        })
        .collect()
}

/// Stable parent-first traversal, with malformed cycles left to pose cycle detection.
pub(crate) fn node_order(nodes: &[RuntimeNode]) -> Vec<usize> {
    let indices: HashMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| (node.object_id, index))
        .collect();
    let mut children = vec![Vec::new(); nodes.len()];
    let mut waiting = vec![false; nodes.len()];
    let mut ready = VecDeque::new();
    for (index, node) in nodes.iter().enumerate() {
        if let Some(&parent) = indices.get(&node.parent_id) {
            children[parent].push(index);
            waiting[index] = true;
        } else {
            ready.push_back(index);
        }
    }
    let mut order = Vec::with_capacity(nodes.len());
    while let Some(index) = ready.pop_front() {
        order.push(index);
        for &child in &children[index] {
            waiting[child] = false;
            ready.push_back(child);
        }
    }
    order.extend(
        waiting
            .iter()
            .enumerate()
            .filter_map(|(index, waiting)| waiting.then_some(index)),
    );
    order
}
