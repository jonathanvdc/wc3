//! Spawn file-backed model instances once their source assets are ready.
use bevy::asset::AssetId;
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::{hash_map::Entry, HashMap, HashSet};

use crate::animation::Wc3Animation;
use crate::asset::Wc3ModelAsset;
use crate::material::Wc3LayerMaterial;
use crate::spawn::{prepare_resolved_model, spawn_prepared_into, PreparedModel};

/// Attach to an entity to spawn an MDX asset beneath it when loading finishes.
/// The entity remains the transform and animation root.
#[derive(Component)]
#[require(Transform)]
pub struct Wc3ModelInstance(pub Handle<Wc3ModelAsset>);

impl Wc3ModelInstance {
    pub fn new(handle: Handle<Wc3ModelAsset>) -> Self {
        Self(handle)
    }
}

#[derive(Resource, Default)]
pub(crate) struct PreparedModelCache {
    prepared: HashMap<AssetId<Wc3ModelAsset>, PreparedModel>,
    failed: HashSet<AssetId<Wc3ModelAsset>>,
}

pub(crate) fn spawn_loaded_instances(
    mut commands: Commands,
    instances: Query<(Entity, &Wc3ModelInstance), Without<Wc3Animation>>,
    sources: Res<Assets<Wc3ModelAsset>>,
    mut cache: ResMut<PreparedModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<Wc3LayerMaterial>>,
    mut inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    cache.prepared.retain(|id, _| sources.contains(*id));
    cache.failed.retain(|id| sources.contains(*id));
    for (root, instance) in &instances {
        let id = instance.0.id();
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
        spawn_prepared_into(&mut commands, &mut materials, &cache.prepared[&id], root);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Wc3Model;

    #[test]
    fn repeated_instances_prepare_one_model() {
        let source = Wc3Model::decode(include_bytes!(
            "../../wc3/tests/fixtures/mdl/quad_model.mdx"
        ))
        .unwrap();
        let textures = vec![None; source.model.textures().len()];
        let mut app = App::new();
        app.insert_resource(Assets::<Wc3ModelAsset>::default());
        app.insert_resource(Assets::<Mesh>::default());
        app.insert_resource(Assets::<Wc3LayerMaterial>::default());
        app.insert_resource(Assets::<SkinnedMeshInverseBindposes>::default());
        app.init_resource::<PreparedModelCache>();
        app.add_systems(Update, spawn_loaded_instances);
        let handle = app
            .world_mut()
            .resource_mut::<Assets<Wc3ModelAsset>>()
            .add(Wc3ModelAsset { source, textures });
        let first = app
            .world_mut()
            .spawn(Wc3ModelInstance::new(handle.clone()))
            .id();
        let second = app
            .world_mut()
            .spawn(Wc3ModelInstance::new(handle.clone()))
            .id();
        app.update();
        assert!(app.world().entity(first).contains::<Wc3Animation>());
        assert!(app.world().entity(second).contains::<Wc3Animation>());
        assert_eq!(
            app.world().resource::<PreparedModelCache>().prepared.len(),
            1
        );
        let meshes = app.world().resource::<Assets<Mesh>>().len();
        let materials = app.world().resource::<Assets<Wc3LayerMaterial>>().len();
        let bindposes = app
            .world()
            .resource::<Assets<SkinnedMeshInverseBindposes>>()
            .len();
        app.world_mut().spawn(Wc3ModelInstance::new(handle));
        app.update();
        assert_eq!(app.world().resource::<Assets<Mesh>>().len(), meshes);
        assert_eq!(
            app.world().resource::<Assets<Wc3LayerMaterial>>().len(),
            materials
        );
        assert_eq!(
            app.world()
                .resource::<Assets<SkinnedMeshInverseBindposes>>()
                .len(),
            bindposes
        );
    }
}
