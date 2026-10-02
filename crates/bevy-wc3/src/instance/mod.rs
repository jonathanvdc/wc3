//! Spawn file-backed model instances once their source assets are ready.
mod cache;
mod ownership;
pub(crate) mod spawn;
pub(crate) use cache::PreparedModelCache;
pub(crate) use ownership::BlockedChildModel;
pub use ownership::{Wc3ModelOwner, Wc3OwnedModels};

use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::{hash_map::Entry, HashSet};

use crate::animation::Wc3Animation;
use crate::assets::loader::Wc3ModelAsset;
use crate::instance::spawn::spawn_prepared_into;
use crate::materials::textures::Wc3TextureBindings;
use crate::materials::Wc3LayerMaterial;
use crate::preparation::prepare_resolved_model;

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
#[path = "tests.rs"]
mod tests;
