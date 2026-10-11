//! Shared preparation and source-asset invalidation for future instances.
use bevy::asset::{AssetEvent, AssetId};
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::*;
use std::collections::{hash_map::Entry, HashMap, HashSet};

use crate::assets::{ModelError, Wc3ModelAsset};
use crate::preparation::{prepare_resolved_model, PreparedModel};

#[derive(Resource, Default)]
pub(crate) struct PreparedModelCache {
    prepared: HashMap<AssetId<Wc3ModelAsset>, PreparedModel>,
    failed: HashSet<AssetId<Wc3ModelAsset>>,
}

impl PreparedModelCache {
    pub(super) fn retain_live(&mut self, sources: &Assets<Wc3ModelAsset>) {
        self.prepared.retain(|id, _| sources.contains(*id));
        self.failed.retain(|id| sources.contains(*id));
    }

    fn invalidate(&mut self, id: AssetId<Wc3ModelAsset>) {
        self.prepared.remove(&id);
        self.failed.remove(&id);
    }

    pub(super) fn get_or_prepare(
        &mut self,
        id: AssetId<Wc3ModelAsset>,
        source: &Wc3ModelAsset,
        meshes: &mut Assets<Mesh>,
        inverse_bindposes: &mut Assets<SkinnedMeshInverseBindposes>,
    ) -> Result<Option<&PreparedModel>, ModelError> {
        if self.failed.contains(&id) {
            return Ok(None);
        }
        if let Entry::Vacant(entry) = self.prepared.entry(id) {
            match prepare_resolved_model(
                meshes,
                inverse_bindposes,
                &source.source,
                source.textures.clone(),
                source.models.clone(),
            ) {
                Ok(prepared) => {
                    entry.insert(prepared);
                }
                Err(error) => {
                    self.failed.insert(id);
                    return Err(error);
                }
            }
        }
        Ok(self.prepared.get(&id))
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
