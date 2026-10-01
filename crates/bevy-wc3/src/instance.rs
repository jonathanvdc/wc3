//! Spawn file-backed model instances once their source assets are ready.
use bevy::asset::AssetId;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::{hash_map::Entry, HashMap, HashSet};

use crate::animation::Wc3Animation;
use crate::asset::Wc3ModelAsset;
use crate::material::Wc3LayerMaterial;
use crate::spawn::{prepare_resolved_model, spawn_prepared_into, PreparedModel};
use crate::texture_bindings::Wc3TextureBindings;

/// Attach to an entity to spawn an MDX or MDL asset beneath it when loading finishes.
/// The entity remains the transform and animation root.
#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Wc3ModelInstance(pub Handle<Wc3ModelAsset>);

impl Wc3ModelInstance {
    pub fn new(handle: Handle<Wc3ModelAsset>) -> Self {
        Self(handle)
    }
}

/// Lifetime owner of a child model root. Unlike `ChildOf`, this relationship
/// does not inherit transforms or visibility. Use both relationships for a
/// following attachment; use ownership alone for particles moving in world space.
/// Despawning the owner recursively despawns its owned models and their contents.
#[derive(Component)]
#[relationship(relationship_target = Wc3OwnedModels)]
pub struct Wc3ModelOwner(pub Entity);

/// Child model roots owned by this entity, maintained by `Wc3ModelOwner`.
#[derive(Component)]
#[relationship_target(relationship = Wc3ModelOwner, linked_spawn)]
pub struct Wc3OwnedModels(Vec<Entity>);

impl Wc3OwnedModels {
    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.0.iter().copied()
    }
}

#[derive(Component)]
pub(crate) struct BlockedChildModel;

#[derive(Resource, Default)]
pub(crate) struct PreparedModelCache {
    prepared: HashMap<AssetId<Wc3ModelAsset>, PreparedModel>,
    failed: HashSet<AssetId<Wc3ModelAsset>>,
}

type UnloadedInstances<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Wc3ModelInstance,
        Option<&'static Wc3TextureBindings>,
        Option<&'static Wc3ModelOwner>,
    ),
    (Without<Wc3Animation>, Without<BlockedChildModel>),
>;

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_loaded_instances(
    mut commands: Commands,
    instances: UnloadedInstances,
    owners: Query<(Option<&Wc3ModelInstance>, Option<&Wc3ModelOwner>)>,
    sources: Res<Assets<Wc3ModelAsset>>,
    mut cache: ResMut<PreparedModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
    mut inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    cache.prepared.retain(|id, _| sources.contains(*id));
    cache.failed.retain(|id| sources.contains(*id));
    for (root, instance, bindings, owner) in &instances {
        let id = instance.0.id();
        if owner.is_some() {
            let mut ancestor = owners
                .get(root)
                .ok()
                .and_then(|(_, owner)| owner.map(|owner| owner.0));
            let mut visited = HashSet::new();
            let mut cyclic = false;
            while let Some(entity) = ancestor {
                if !visited.insert(entity) {
                    cyclic = true;
                    break;
                }
                let Ok((model, owner)) = owners.get(entity) else {
                    break;
                };
                if model.is_some_and(|model| model.0.id() == id) {
                    cyclic = true;
                    break;
                }
                ancestor = owner.map(|owner| owner.0);
            }
            if cyclic {
                warn!("Skipping cyclic WC3 child model {:?}", instance.0);
                commands
                    .entity(root)
                    .insert((BlockedChildModel, Visibility::Hidden));
                continue;
            }
        }
        let Some(asset) = sources.get(id) else {
            continue;
        };
        if cache.failed.contains(&id) {
            continue;
        }
        if let Entry::Vacant(entry) = cache.prepared.entry(id) {
            match prepare_resolved_model(
                &mut meshes,
                &mut materials,
                &mut inverse_bindposes,
                &asset.source,
                asset.textures.clone(),
                asset.models.clone(),
            ) {
                Ok(prepared) => {
                    entry.insert(prepared);
                }
                Err(error) => {
                    warn!("Could not prepare WC3 model {:?}: {error}", instance.0);
                    cache.failed.insert(id);
                    continue;
                }
            }
        }
        spawn_prepared_into(
            &mut commands,
            &mut meshes,
            &mut materials,
            &cache.prepared[&id],
            root,
            bindings.cloned().unwrap_or_default(),
        );
    }
}

#[cfg(test)]
#[path = "instance_tests.rs"]
mod tests;
