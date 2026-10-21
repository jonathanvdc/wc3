//! World-space particle roots retain lifetime ownership without transform parenting.
use super::model_budget::{ModelActorLease, ModelActorTree};
use bevy::prelude::*;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Resource, Default)]
pub(crate) struct ModelParticleBirths(pub VecDeque<Entity>);

fn record_birth(world: &mut World, entity: Entity) {
    if world
        .get::<ModelActorLease>(entity)
        .is_some_and(|lease| !lease.is_current(world.get::<ModelActorTree>(lease.budget().root())))
    {
        world.despawn(entity);
        return;
    }
    if let Some(mut births) = world.get_resource_mut::<ModelParticleBirths>() {
        births.0.push_back(entity);
    }
    let budget = world
        .get::<ModelActorLease>(entity)
        .map(|lease| lease.budget().clone());
    if let Some(budget) = budget {
        if let Some(mut tree) = world.get_mut::<ModelActorTree>(budget.root()) {
            if Arc::ptr_eq(&tree.budget, &budget) {
                tree.pending.push_back(entity);
            }
        }
    }
}
pub(crate) fn queue_model_birth(commands: &mut Commands, entity: Entity) {
    commands.queue(move |world: &mut World| {
        record_birth(world, entity);
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
    lease: Option<ModelActorLease>,
) -> Entity {
    let entity = commands.spawn_empty().id();
    commands.queue(move |world: &mut World| {
        let current = lease.as_ref().is_none_or(|lease| {
            lease.is_current(world.get::<ModelActorTree>(lease.budget().root()))
        });
        if current && world.get_entity(owner).is_ok() {
            if let Ok(mut child) = world.get_entity_mut(entity) {
                child.insert((ModelParticleOwner(owner), bundle));
                if let Some(lease) = lease {
                    child.insert(lease);
                }
                record_birth(world, entity);
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
