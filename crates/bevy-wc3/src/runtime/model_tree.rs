//! Application-root budgets and ordered pending work for model-particle trees.
use super::model_driver::actor_parent;
use super::ModelRig;
use crate::effects::model_budget::{
    ActorBudget, ActorFrame, ModelActorLease, ModelActorTree, ModelParticleLimits,
    ModelParticleTreeStats,
};
use crate::instances::{ModelInstance, ModelInstanceError, PreparedModelRequest};
use crate::ModelAsset;
use bevy::asset::{AssetServer, LoadState};
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};
use std::mem::take;
use std::sync::Arc;

pub(super) fn actor_frame(world: &mut World, entity: Entity, owned: bool) -> Option<ActorFrame> {
    let frame = world
        .get_resource::<FrameCount>()
        .copied()
        .unwrap_or_default();
    if let Some(lease) = world.get::<ModelActorLease>(entity) {
        let budget = lease.budget().clone();
        let tree = world.get::<ModelActorTree>(budget.root())?;
        return Arc::ptr_eq(&tree.budget, &budget).then_some(ActorFrame {
            budget,
            frame,
            owned,
        });
    }
    let mut root = entity;
    let mut seen = HashSet::new();
    while let Some(parent) = actor_parent(world, root) {
        if !seen.insert(root) || seen.len() > 4096 {
            return None;
        }
        root = parent;
    }
    world.get::<ModelRig>(root)?;
    let budget = if let Some(tree) = world.get::<ModelActorTree>(root) {
        tree.budget.clone()
    } else {
        let limits = world
            .get::<ModelParticleLimits>(root)
            .copied()
            .unwrap_or_default();
        let budget = ActorBudget::new(root, limits);
        world.entity_mut(root).insert((
            ModelActorTree {
                budget: budget.clone(),
                pending: VecDeque::new(),
            },
            ModelParticleTreeStats::default(),
        ));
        budget
    };
    Some(ActorFrame {
        budget,
        frame,
        owned,
    })
}
fn ready(world: &World, entity: Entity) -> bool {
    if world.get::<PreparedModelRequest>(entity).is_some() {
        return true;
    }
    let Some(request) = world.get::<ModelInstance>(entity) else {
        return false;
    };
    if let Some(server) = world.get_resource::<AssetServer>() {
        if matches!(server.load_state(request.0.id()), LoadState::Failed(_)) {
            return true;
        }
        if server.get_load_states(request.0.id()).is_some()
            && !server.is_loaded_with_dependencies(request.0.id())
        {
            return false;
        }
    }
    world
        .resource::<Assets<ModelAsset>>()
        .get(&request.0)
        .is_some()
}
/// Reserve work for previously accepted ready requests before fresh emissions.
/// Unresolved or quota-deferred requests rotate, without producing catch-up births.
pub(super) fn prepare_tree(world: &mut World, root: Entity) {
    let frame = world
        .get_resource::<FrameCount>()
        .copied()
        .unwrap_or_default();
    let limits = world
        .get::<ModelParticleLimits>(root)
        .copied()
        .unwrap_or_default();
    let Some(mut tree) = world.get_mut::<ModelActorTree>(root) else {
        return;
    };
    let budget = tree.budget.clone();
    budget.set_limits(limits);
    let mut pending = take(&mut tree.pending);
    let count = pending.len();
    for _ in 0..count {
        let Some(entity) = pending.pop_front() else {
            break;
        };
        let Some(lease) = world.get::<ModelActorLease>(entity) else {
            continue;
        };
        if !Arc::ptr_eq(lease.budget(), &budget) || !lease.is_pending() {
            continue;
        }
        if world.get::<ModelInstanceError>(entity).is_some()
            && world.get::<ModelRig>(entity).is_none()
        {
            world.despawn(entity);
            continue;
        }
        if ready(world, entity) {
            if let Some(mut lease) = world.get_mut::<ModelActorLease>(entity) {
                if !lease.admit_ready(frame) {
                    pending.push_front(entity);
                    break;
                }
            }
        }
        pending.push_back(entity);
    }
    if let Some(mut tree) = world.get_mut::<ModelActorTree>(root) {
        tree.pending = pending;
    }
}
pub(super) fn publish_tree(world: &mut World, root: Entity) {
    let frame = world
        .get_resource::<FrameCount>()
        .copied()
        .unwrap_or_default();
    let Some(tree) = world.get::<ModelActorTree>(root) else {
        return;
    };
    let stats = tree.budget.stats(frame);
    if world.get::<ModelParticleTreeStats>(root) != Some(&stats) {
        world.entity_mut(root).insert(stats);
    }
}

#[cfg(test)]
#[path = "model_tree_tests.rs"]
mod tests;
