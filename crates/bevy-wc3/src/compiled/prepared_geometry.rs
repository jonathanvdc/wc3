//! Non-owning content lookup for renderer-owned geometry in live asset stores.
use crate::mesh_interner::{mesh_equal, mesh_hash};
use bevy::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::prelude::{AssetId, Assets, Handle, Mat4, Mesh, Resource};
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};

/// Non-owning lookup for equal prepared meshes and inverse-bind palettes.
///
/// Reuse with [`CompiledModel::prepare_with_cache`](super::CompiledModel::prepare_with_cache)
/// to share assets across independently compiled models. The cache holds only
/// candidate asset IDs; prepared models own their strong handles. Every hit
/// compares current contents and acquires ownership from the supplied stores,
/// so reusing the cache with different stores never transfers foreign handles.
/// Missing assets are pruned during preparation.
/// [`Wc3BevyPlugin`](crate::Wc3BevyPlugin) initializes this resource for asset-backed
/// instances; manual systems can borrow it to share the same prepared assets.
#[derive(Default, Resource)]
pub struct PreparedGeometryCache {
    meshes: HashMap<u64, Vec<AssetId<Mesh>>>,
    palettes: HashMap<u64, Vec<AssetId<SkinnedMeshInverseBindposes>>>,
}

impl PreparedGeometryCache {
    /// Remove stale lookup metadata once per newly prepared model, rather than per draw.
    pub(crate) fn prune(
        &mut self,
        meshes: &Assets<Mesh>,
        palettes: &Assets<SkinnedMeshInverseBindposes>,
    ) {
        self.meshes.retain(|_, bucket| {
            bucket.retain(|id| meshes.contains(*id));
            !bucket.is_empty()
        });
        self.palettes.retain(|_, bucket| {
            bucket.retain(|id| palettes.contains(*id));
            !bucket.is_empty()
        });
    }

    pub(super) fn mesh(&mut self, assets: &mut Assets<Mesh>, mesh: &Mesh) -> Handle<Mesh> {
        self.mesh_bucket(mesh_hash(mesh), assets, mesh)
    }

    fn mesh_bucket(&mut self, hash: u64, assets: &mut Assets<Mesh>, mesh: &Mesh) -> Handle<Mesh> {
        let bucket = self.meshes.entry(hash).or_default();
        bucket.retain(|id| assets.contains(*id));
        for &id in bucket.iter() {
            if !assets.get(id).is_some_and(|value| mesh_equal(value, mesh)) {
                continue;
            }
            // Equal IDs can belong to unrelated stores. Obtain ownership from
            // the store whose live contents were compared, never from the cache.
            if let Some(handle) = assets.get_strong_handle(id) {
                return handle;
            }
        }
        let handle = assets.add(mesh.clone());
        bucket.push(handle.id());
        handle
    }

    pub(super) fn palette(
        &mut self,
        assets: &mut Assets<SkinnedMeshInverseBindposes>,
        matrices: &[Mat4],
    ) -> Handle<SkinnedMeshInverseBindposes> {
        let mut hash = DefaultHasher::new();
        matrices.len().hash(&mut hash);
        for matrix in matrices {
            matrix.to_cols_array().map(f32::to_bits).hash(&mut hash);
        }
        self.palette_bucket(hash.finish(), assets, matrices)
    }

    fn palette_bucket(
        &mut self,
        hash: u64,
        assets: &mut Assets<SkinnedMeshInverseBindposes>,
        matrices: &[Mat4],
    ) -> Handle<SkinnedMeshInverseBindposes> {
        let bucket = self.palettes.entry(hash).or_default();
        bucket.retain(|id| assets.contains(*id));
        for &id in bucket.iter() {
            if !assets
                .get(id)
                .is_some_and(|value| palettes_equal(value, matrices))
            {
                continue;
            }
            if let Some(handle) = assets.get_strong_handle(id) {
                return handle;
            }
        }
        let handle = assets.add(matrices.to_vec());
        bucket.push(handle.id());
        handle
    }
}

fn palettes_equal(left: &[Mat4], right: &[Mat4]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.to_cols_array().map(f32::to_bits) == right.to_cols_array().map(f32::to_bits)
        })
}

#[cfg(test)]
#[path = "prepared_geometry_tests.rs"]
mod tests;
