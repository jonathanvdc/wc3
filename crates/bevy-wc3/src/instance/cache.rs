//! Shared preparation and source-asset invalidation for future instances.
use bevy::asset::{AssetEvent, AssetId};
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::assets::Wc3ModelAsset;
use crate::preparation::{prepare_model_asset, PreparedModel};

#[derive(Resource, Default)]
pub(crate) struct PreparedModelCache {
    prepared: HashMap<AssetId<Wc3ModelAsset>, PreparedModel>,
}

impl PreparedModelCache {
    pub(super) fn retain_live(&mut self, sources: &Assets<Wc3ModelAsset>) {
        self.prepared.retain(|id, _| sources.contains(*id));
    }

    fn invalidate(&mut self, id: AssetId<Wc3ModelAsset>) {
        self.prepared.remove(&id);
    }

    pub(super) fn get_or_prepare(
        &mut self,
        id: AssetId<Wc3ModelAsset>,
        source: &Wc3ModelAsset,
        meshes: &mut Assets<Mesh>,
        inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> &PreparedModel {
        self.prepared
            .entry(id)
            .or_insert_with(|| prepare_model_asset(meshes, inverse_bindposes, source))
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.prepared.len()
    }
}

pub(crate) fn invalidate_prepared_models(
    mut events: MessageReader<AssetEvent<Wc3ModelAsset>>,
    mut cache: ResMut<PreparedModelCache>,
) {
    for event in events.read() {
        match *event {
            AssetEvent::Added { id } | AssetEvent::Modified { id } | AssetEvent::Removed { id } => {
                cache.invalidate(id);
            }
            AssetEvent::Unused { .. } | AssetEvent::LoadedWithDependencies { .. } => {}
        }
    }
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
