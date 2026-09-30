use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::{Model, V1800};

use crate::animation::{AnimatedNode, Wc3Animation};

pub(super) struct Rig {
    pub(super) joints: Vec<Entity>,
    pub(super) inverse_bindposes: Handle<SkinnedMeshInverseBindposes>,
}

/// Entities for a model instance's animated nodes, keyed by MDX object ID.
///
/// The component lives on the model's animation root. Effects can use these
/// entities as their transforms when their record types are implemented.
#[derive(Component)]
pub struct Wc3NodeEntities {
    by_object_id: HashMap<u32, Entity>,
}

impl Wc3NodeEntities {
    pub fn get(&self, object_id: u32) -> Option<Entity> {
        self.by_object_id.get(&object_id).copied()
    }
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
    let source_nodes = model.nodes();
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
    commands.entity(root).insert(Wc3NodeEntities {
        by_object_id: nodes,
    });
    Rig {
        joints,
        inverse_bindposes: inverse_bindposes.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::world::{CommandQueue, World};
    use wc3::model::mdl::Read as _;

    #[test]
    fn every_node_type_keeps_its_entity_and_parent_from_mdl() {
        let source = include_str!("../../tests/fixtures/attachment_parent.mdl");
        let model = Model::<V1800>::decode_mdl(source).unwrap();
        assert_eq!(model.geosets()[0].matrix_indices(), [2, 10]);
        let mut world = World::new();
        let root = world.spawn(Transform::default()).id();
        let mut bindposes = Assets::<SkinnedMeshInverseBindposes>::default();
        let inverse_bindposes = bindposes.add(vec![Mat4::IDENTITY]);
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, &world);
        let rig = spawn_rig(&mut commands, &model, root, &inverse_bindposes);
        queue.apply(&mut world);

        let entities = world.entity(root).get::<Wc3NodeEntities>().unwrap();
        for object_id in 0..11 {
            assert!(
                entities.get(object_id).is_some(),
                "missing node {object_id}"
            );
        }
        let bone = entities.get(2).unwrap();
        let attached_bone = entities.get(10).unwrap();
        assert_eq!(rig.joints, [bone, attached_bone]);
        assert_eq!(
            world.entity(bone).get::<ChildOf>(),
            Some(&ChildOf(entities.get(9).unwrap()))
        );
        assert_eq!(
            world.entity(attached_bone).get::<ChildOf>(),
            Some(&ChildOf(entities.get(1).unwrap()))
        );
        for object_id in 1..10 {
            if object_id == 2 {
                continue;
            }
            let entity = entities.get(object_id).unwrap();
            assert_eq!(
                world.entity(entity).get::<ChildOf>(),
                Some(&ChildOf(entities.get(0).unwrap()))
            );
        }
    }
}
