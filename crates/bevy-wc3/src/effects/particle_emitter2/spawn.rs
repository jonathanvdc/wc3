use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use std::collections::HashMap;

use super::{Particle2State, ParticleInstances, ParticleTextureSlot};
use crate::materials::textures::Wc3TextureBindings;
use crate::preparation::PreparedModel;

pub(crate) fn spawn_particles2(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    prepared: &PreparedModel,
    root: Entity,
    node_entities: &HashMap<u32, Entity>,
    bindings: &Wc3TextureBindings,
) {
    for (emitter_id, definition) in prepared.model.particle_emitters2().into_iter().enumerate() {
        let Some(&node) = node_entities.get(&definition.node.object_id) else {
            continue;
        };
        let texture = bindings.particle(emitter_id);
        let mesh = meshes.add(Rectangle::new(2.0, 2.0));
        let render = ParticleInstances {
            records: Default::default(),
            live_indices: Default::default(),
            uniform: Default::default(),
            texture,
            filter: definition.filter_mode,
            priority_plane: definition.priority_plane,
            sort_far: definition.node.flags.sort_prims_far_z(),
        };
        let entity = commands
            .spawn((
                Mesh3d(mesh.clone()),
                render,
                Particle2State::new(root, node, definition),
                ParticleTextureSlot(emitter_id),
                Transform::default(),
                NoFrustumCulling,
            ))
            .id();
        commands.entity(root).add_child(entity);
    }
}
