use bevy::prelude::*;
use std::collections::HashMap;

use super::ParticleState;
use crate::spawn::PreparedModel;

pub(crate) fn spawn_particles(
    commands: &mut Commands,
    prepared: &PreparedModel,
    root: Entity,
    node_entities: &HashMap<u32, Entity>,
) {
    for (index, definition) in prepared.model.particle_emitters().into_iter().enumerate() {
        let Some(model) = prepared.model_resources().particle(index) else {
            continue;
        };
        let Some(&node) = node_entities.get(&definition.node.object_id) else {
            continue;
        };
        commands.spawn((
            ParticleState::new(root, node, definition, model),
            ChildOf(root),
        ));
    }
}
