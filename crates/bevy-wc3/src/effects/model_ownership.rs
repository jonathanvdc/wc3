//! World-space particle roots retain lifetime ownership without transform parenting.
use bevy::prelude::*;
use std::collections::VecDeque;

#[derive(Resource, Default)]
pub(crate) struct ModelParticleBirths(pub VecDeque<Entity>);

pub(crate) fn queue_model_birth(commands: &mut Commands, entity: Entity) {
    commands.queue(move |world: &mut World| {
        if let Some(mut births) = world.get_resource_mut::<ModelParticleBirths>() {
            births.0.push_back(entity);
        }
    });
}
#[derive(Component)]
#[relationship(relationship_target = ModelParticleChildren)]
pub(crate) struct ModelParticleOwner(pub Entity);
#[derive(Component, Default)]
#[relationship_target(relationship = ModelParticleOwner, linked_spawn)]
pub(crate) struct ModelParticleChildren(Vec<Entity>);
/// Never publish a request whose renderer owner disappeared before this flush.
/// The closure also owns any admission lease until insertion succeeds.
pub(crate) fn spawn_owned_model<B: Bundle>(
    commands: &mut Commands,
    owner: Entity,
    bundle: B,
) -> Entity {
    let entity = commands.spawn_empty().id();
    commands.queue(move |world: &mut World| {
        if world.get_entity(owner).is_ok() {
            if let Ok(mut child) = world.get_entity_mut(entity) {
                child.insert((ModelParticleOwner(owner), bundle));
                if let Some(mut births) = world.get_resource_mut::<ModelParticleBirths>() {
                    births.0.push_back(entity);
                }
            }
        } else if let Ok(child) = world.get_entity_mut(entity) {
            child.despawn();
        }
    });
    entity
}
#[cfg(test)]
#[path = "model_ownership_tests.rs"]
mod tests;
