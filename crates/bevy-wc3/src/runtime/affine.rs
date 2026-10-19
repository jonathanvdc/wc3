//! Publish exact model matrices after Bevy's TRS propagation.
use super::ModelRig;
use bevy::prelude::*;
use std::collections::HashMap;

/// Corrections can introduce shear. Transport their full matrices to skinning and
/// descendants instead of relying on the informational local Transform approximation.
pub(crate) fn publish_affine_poses(
    rigs: Query<&ModelRig>,
    mut hierarchy: Query<(&Transform, Option<&Children>, &mut GlobalTransform)>,
    mut nodes: Local<HashMap<Entity, (Entity, Mat4)>>,
    mut stack: Local<Vec<(Entity, Mat4)>>,
) {
    if !rigs
        .iter()
        .any(|rig| rig.model.nodes.iter().any(|node| node.correction.is_some()))
    {
        return;
    }
    nodes.clear();
    for rig in &rigs {
        for (&entity, &pose) in rig.nodes.iter().zip(&rig.poses) {
            nodes.insert(entity, (rig.basis_entity, pose));
        }
    }
    for rig in &rigs {
        if !rig.model.nodes.iter().any(|node| node.correction.is_some()) {
            continue;
        }
        let Ok((_, _, global)) = hierarchy.get(rig.basis_entity) else {
            continue;
        };
        stack.clear();
        stack.push((rig.basis_entity, global.to_matrix()));
        while let Some((entity, mut matrix)) = stack.pop() {
            if let Some(&(basis, pose)) = nodes.get(&entity) {
                if let Ok((_, _, global)) = hierarchy.get(basis) {
                    matrix = global.to_matrix() * pose;
                }
            }
            let Ok((_, children, mut global)) = hierarchy.get_mut(entity) else {
                continue;
            };
            let value = GlobalTransform::from(matrix);
            if *global != value {
                *global = value;
            }
            let children = children
                .map(|children| children.iter().collect::<Vec<_>>())
                .unwrap_or_default();
            for child in children {
                if let Ok((transform, _, _)) = hierarchy.get(child) {
                    stack.push((child, matrix * transform.to_matrix()));
                }
            }
        }
    }
}
