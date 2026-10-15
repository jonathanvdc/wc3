//! Asset-backed instance requests and shared GPU preparation.
use super::runtime::populate_universal_model;
use super::{
    PreparedUniversalModel, UniversalInstance, UniversalMaterial, UniversalModelAsset,
    UniversalPlayback,
};
use bevy::asset::{AssetEvent, AssetId};
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::HashMap;

/// Load and populate a Universal model at an existing application entity.
///
/// Add this component with a typed model handle. Once ready, the plugin replaces
/// the request with [`super::UniversalInstance`] and [`UniversalModelSource`].
/// Existing root playback and pose overrides are retained. Replacing a request
/// despawns only the old renderer-owned hierarchy, preserving application children.
#[derive(Component, Clone)]
#[require(Transform, Visibility)]
pub struct UniversalModelInstance(pub Handle<UniversalModelAsset>);

/// Source identity retained after an asset-backed instance has been populated.
#[derive(Component, Clone, Copy, Debug)]
pub struct UniversalModelSource(pub AssetId<UniversalModelAsset>);

#[derive(Resource, Default)]
pub(crate) struct UniversalPreparedCache(
    HashMap<AssetId<UniversalModelAsset>, PreparedUniversalModel>,
);

pub(crate) fn invalidate_universal_models(
    mut events: MessageReader<AssetEvent<UniversalModelAsset>>,
    mut cache: ResMut<UniversalPreparedCache>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } =
            event
        {
            cache.0.remove(id);
        }
    }
}

pub(crate) fn spawn_loaded_universal_models(
    mut commands: Commands,
    requests: Query<(
        Entity,
        &UniversalModelInstance,
        Option<&UniversalPlayback>,
        Option<&UniversalInstance>,
    )>,
    assets: Res<Assets<UniversalModelAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut binds: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    mut materials: ResMut<Assets<UniversalMaterial>>,
    mut cache: ResMut<UniversalPreparedCache>,
) {
    for (root, request, playback, previous) in &requests {
        let Some(source) = assets.get(&request.0) else {
            continue;
        };
        let prepared = cache
            .0
            .entry(request.0.id())
            .or_insert_with(|| source.compiled.prepare(&mut meshes, &mut binds));
        if let Some(previous) = previous {
            previous.despawn_owned_subtree(&mut commands);
        }
        populate_universal_model(&mut commands, &mut meshes, &mut materials, prepared, root);
        if let Some(playback) = playback {
            commands.entity(root).insert(playback.clone());
        }
        commands
            .entity(root)
            .remove::<UniversalModelInstance>()
            .insert(UniversalModelSource(request.0.id()));
    }
}
