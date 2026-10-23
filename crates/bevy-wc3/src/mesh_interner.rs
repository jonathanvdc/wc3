//! Content sharing after complete vertex projection, skinning and indexing.
use bevy::mesh::Indices;
use bevy::prelude::Mesh;
use bytemuck::cast_slice;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Default)]
pub(super) struct MeshInterner {
    buckets: HashMap<u64, Vec<Arc<Mesh>>>,
}
impl MeshInterner {
    pub(super) fn intern(&mut self, mesh: Mesh) -> Arc<Mesh> {
        self.intern_bucket(mesh_hash(&mesh), mesh)
    }
    fn intern_bucket(&mut self, hash: u64, mesh: Mesh) -> Arc<Mesh> {
        let bucket = self.buckets.entry(hash).or_default();
        if let Some(existing) = bucket.iter().find(|existing| mesh_equal(existing, &mesh)) {
            return existing.clone();
        }
        let mesh = Arc::new(mesh);
        bucket.push(mesh.clone());
        mesh
    }
}
pub(super) fn mesh_hash(mesh: &Mesh) -> u64 {
    let mut hash = DefaultHasher::new();
    mesh.primitive_topology().hash(&mut hash);
    for (attribute, values) in mesh.attributes() {
        attribute.id.hash(&mut hash);
        attribute.format.hash(&mut hash);
        values.get_bytes().hash(&mut hash);
    }
    match mesh.indices() {
        None => 0u8.hash(&mut hash),
        Some(Indices::U16(values)) => {
            1u8.hash(&mut hash);
            values.hash(&mut hash);
        }
        Some(Indices::U32(values)) => {
            2u8.hash(&mut hash);
            values.hash(&mut hash);
        }
    }
    hash.finish()
}
pub(super) fn mesh_equal(left: &Mesh, right: &Mesh) -> bool {
    // Full Bevy equality additionally checks morph, usage and skin-bound metadata.
    // Explicit bytes and float bits preserve signed zero throughout the payload.
    left == right
        && left
            .attributes()
            .zip(right.attributes())
            .all(|((_, left), (_, right))| left.get_bytes() == right.get_bytes())
        && left.get_morph_targets().map(cast_slice::<_, u8>)
            == right.get_morph_targets().map(cast_slice::<_, u8>)
        && left.final_aabb.map(|bounds| {
            [
                bounds.min.to_array().map(f32::to_bits),
                bounds.max.to_array().map(f32::to_bits),
            ]
        }) == right.final_aabb.map(|bounds| {
            [
                bounds.min.to_array().map(f32::to_bits),
                bounds.max.to_array().map(f32::to_bits),
            ]
        })
        && match (left.skinned_mesh_bounds(), right.skinned_mesh_bounds()) {
            (Some(left), Some(right)) => {
                left.aabbs.iter().zip(&right.aabbs).all(|(left, right)| {
                    left.center.to_array().map(f32::to_bits)
                        == right.center.to_array().map(f32::to_bits)
                        && left.half_size.to_array().map(f32::to_bits)
                            == right.half_size.to_array().map(f32::to_bits)
                })
            }
            (None, None) => true,
            _ => false,
        }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::RenderAssetUsages;
    use bevy::mesh::PrimitiveTopology;

    fn mesh() -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]],
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.], [1., 0.], [0., 1.]]);
        mesh.insert_indices(Indices::U16(vec![0, 1, 2]));
        mesh
    }
    #[test]
    fn identical_outputs_share_but_collision_does_not_merge_distinct_geometry() {
        let mut interner = MeshInterner::default();
        let first = interner.intern(mesh());
        assert!(Arc::ptr_eq(&first, &interner.intern(mesh())));
        let mut different = mesh();
        different.insert_attribute(Mesh::ATTRIBUTE_TANGENT, vec![[1., 0., 0., 1.]; 3]);
        let collision = interner.intern_bucket(mesh_hash(&first), different);
        assert!(!Arc::ptr_eq(&first, &collision));
        let mut different = mesh();
        different.insert_indices(Indices::U16(vec![0, 2, 1]));
        assert!(!Arc::ptr_eq(&first, &interner.intern(different)));
    }
}
