use bevy::prelude::*;

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
