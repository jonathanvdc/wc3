use crate::assets::Wc3ModelAsset;
use bevy::asset::AssetId;
use bevy::prelude::*;
use std::collections::HashSet;

/// Lifetime owner of a child model root. Unlike `ChildOf`, this relationship
/// does not inherit transforms or visibility. Use both relationships for a
/// following attachment; use ownership alone for particles moving in world space.
/// Despawning the owner recursively despawns its owned models and their contents.
#[derive(Component)]
#[relationship(relationship_target = Wc3OwnedModels)]
pub struct Wc3ModelOwner(
    /// Root entity that owns this child model.
    pub Entity,
);

/// Child model roots owned by this entity, maintained by `Wc3ModelOwner`.
#[derive(Component)]
#[relationship_target(relationship = Wc3ModelOwner, linked_spawn)]
pub struct Wc3OwnedModels(Vec<Entity>);

impl Wc3OwnedModels {
    /// Iterates the currently owned child model roots in relationship order.
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }
}

#[derive(Component)]
pub(crate) struct BlockedChildModel;

/// Walk lifetime owners, including world-space model particles.
pub(super) fn has_model_cycle(
    root: Entity,
    model: AssetId<Wc3ModelAsset>,
    lookup: impl Fn(Entity) -> Option<(Option<AssetId<Wc3ModelAsset>>, Option<Entity>)>,
) -> bool {
    let mut ancestor = lookup(root).and_then(|(_, owner)| owner);
    let mut visited = HashSet::new();
    while let Some(entity) = ancestor {
        if !visited.insert(entity) {
            return true;
        }
        let Some((ancestor_model, owner)) = lookup(entity) else {
            break;
        };
        if ancestor_model == Some(model) {
            return true;
        }
        ancestor = owner;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn lifetime_ancestry_detects_models_and_owner_cycles() {
        let mut world = World::new();
        let root = world.spawn_empty().id();
        let parent = world.spawn_empty().id();
        let grandparent = world.spawn_empty().id();
        let model = AssetId::default();
        let other = AssetId::invalid();
        let mut owners = HashMap::from([
            (root, (Some(model), Some(parent))),
            (parent, (Some(other), Some(grandparent))),
            (grandparent, (Some(model), None)),
        ]);
        assert!(has_model_cycle(root, model, |entity| owners
            .get(&entity)
            .copied()));
        owners.insert(grandparent, (Some(other), None));
        assert!(!has_model_cycle(root, model, |entity| owners
            .get(&entity)
            .copied()));
        owners.insert(grandparent, (None, Some(parent)));
        assert!(has_model_cycle(root, model, |entity| owners
            .get(&entity)
            .copied()));
        owners.remove(&parent);
        assert!(!has_model_cycle(root, model, |entity| owners
            .get(&entity)
            .copied()));
    }
}
