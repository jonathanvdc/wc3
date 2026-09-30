use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::{Model, V1800};

use crate::animation::{AnimatedNode, Wc3Animation};

pub(super) struct Rig {
    pub(super) joints: Vec<Entity>,
    pub(super) inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
}

pub(super) fn joint_ids(model: &Model<V1800>) -> Vec<u32> {
    let mut ids: Vec<_> = model
        .bones()
        .iter()
        .map(|bone| bone.node.object_id)
        .collect();
    ids.sort_unstable();
    ids
}

pub(super) fn spawn_animation_root(commands: &mut Commands, model: &Model<V1800>, root: Entity) {
    commands.entity(root).insert(Wc3Animation {
        sequence: 0,
        elapsed_ms: 0.0,
        speed: 1.0,
        playing: true,
        sequences: model.sequences(),
        global_sequences: model.global_sequences(),
    });
}

pub(super) fn spawn_rig(
    commands: &mut Commands,
    model: &Model<V1800>,
    root: Entity,
    inverse_bindposes: &Handle<SkinnedMeshInverseBindposes>,
) -> Rig {
    let mut source_nodes: Vec<_> = model.bones().iter().map(|bone| bone.node.clone()).collect();
    source_nodes.extend(model.helpers());
    let pivots = model.pivot_points();
    let mut nodes = HashMap::new();
    for node in &source_nodes {
        let pivot = pivots
            .get(node.object_id as usize)
            .copied()
            .unwrap_or([0.0; 3]);
        let parent_pivot = pivots
            .get(node.parent_id as usize)
            .copied()
            .unwrap_or([0.0; 3]);
        let entity = commands
            .spawn((
                AnimatedNode {
                    root,
                    pivot: Vec3::from_array(pivot),
                    parent_pivot: Vec3::from_array(parent_pivot),
                    translation: node.translation.clone(),
                    rotation: node.rotation.clone(),
                    scaling: node.scaling.clone(),
                },
                Transform::from_translation(
                    Vec3::from_array(pivot) - Vec3::from_array(parent_pivot),
                ),
            ))
            .id();
        nodes.insert(node.object_id, entity);
    }
    for node in &source_nodes {
        let parent = nodes.get(&node.parent_id).copied().unwrap_or(root);
        commands.entity(parent).add_child(nodes[&node.object_id]);
    }
    let joint_ids = joint_ids(model);
    let joints = joint_ids.iter().map(|id| nodes[id]).collect();
    Rig {
        joints,
        inverse_bindposes: inverse_bindposes.clone(),
    }
}
