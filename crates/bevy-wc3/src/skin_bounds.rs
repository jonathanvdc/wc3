//! Conservative eight-influence bounds and asset-specific invalidation.
use super::mesh_attributes::{EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT};
use crate::error::ModelError;
use bevy::asset::{AssetEvent, AssetId};
use bevy::camera::{primitives::Aabb, visibility::NoFrustumCulling};
use bevy::ecs::system::SystemParam;
use bevy::mesh::{
    skinning::{
        entity_aabb_from_skinned_mesh_bounds, JointAabb, JointIndex, SkinnedMesh,
        SkinnedMeshBounds, SkinnedMeshInverseBindposes,
    },
    VertexAttributeValues,
};
use bevy::prelude::*;
use std::collections::HashSet;

pub(super) fn compile_skin_bounds(mesh: &mut Mesh) -> Result<(), ModelError> {
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .and_then(VertexAttributeValues::as_float3)
        .ok_or_else(|| ModelError::new("skin bounds require positions"))?;
    let mut ranges: Vec<Option<(Vec3, Vec3)>> = Vec::new();
    let mut sums = vec![0f32; positions.len()];
    for (index_attribute, weight_attribute) in [
        (Mesh::ATTRIBUTE_JOINT_INDEX, Mesh::ATTRIBUTE_JOINT_WEIGHT),
        (EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT),
    ] {
        let (indices, weights) = match (
            mesh.attribute(index_attribute),
            mesh.attribute(weight_attribute),
        ) {
            (None, None) => continue,
            (
                Some(VertexAttributeValues::Uint16x4(indices)),
                Some(VertexAttributeValues::Float32x4(weights)),
            ) if indices.len() == positions.len() && weights.len() == positions.len() => {
                (indices, weights)
            }
            _ => return Err(ModelError::new("malformed skin bounds streams")),
        };
        for (vertex, ((position, indices), weights)) in
            positions.iter().zip(indices).zip(weights).enumerate()
        {
            let point = Vec3::from_array(*position);
            if !point.is_finite() {
                return Err(ModelError::new("nonfinite skin bounds positions"));
            }
            for (&index, &weight) in indices.iter().zip(weights) {
                if !weight.is_finite() || weight < 0. {
                    return Err(ModelError::new("invalid skin bounds weights"));
                }
                sums[vertex] += weight;
                if weight <= 0. {
                    continue;
                }
                let index = usize::from(index);
                ranges.resize(ranges.len().max(index + 1), None);
                ranges[index] = Some(match ranges[index] {
                    Some((min, max)) => (min.min(point), max.max(point)),
                    None => (point, point),
                });
            }
        }
    }
    if sums.iter().any(|sum| (*sum - 1.).abs() > 1e-4) {
        return Err(ModelError::new(
            "skin bounds require normalized positive weights",
        ));
    }
    let mut bounds = SkinnedMeshBounds::default();
    for (index, range) in ranges.into_iter().enumerate() {
        if let Some((min, max)) = range {
            bounds
                .aabb_index_to_joint_index
                .push(JointIndex(index as u16));
            bounds.aabbs.push(JointAabb {
                center: (min + max) * 0.5,
                half_size: (max - min) * 0.5,
            });
        }
    }
    mesh.set_skinned_mesh_bounds(Some(bounds));
    Ok(())
}
#[derive(Component)]
pub(crate) struct SkinBounds {
    dirty: bool,
    fallback: bool,
    #[cfg(test)]
    updates: usize,
}
impl Default for SkinBounds {
    fn default() -> Self {
        Self {
            dirty: true,
            fallback: false,
            #[cfg(test)]
            updates: 0,
        }
    }
}
#[derive(Default)]
pub(crate) struct BoundsChanges {
    meshes: HashSet<AssetId<Mesh>>,
    binds: HashSet<AssetId<SkinnedMeshInverseBindposes>>,
}
type Parts<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut SkinBounds,
        &'static mut Aabb,
        Ref<'static, Mesh3d>,
        Ref<'static, SkinnedMesh>,
        Ref<'static, GlobalTransform>,
        &'static InheritedVisibility,
    ),
>;

/// Asset stores and their per-system invalidation cursor stay one cohesive input.
#[derive(SystemParam)]
pub(crate) struct SkinBoundAssets<'w, 's> {
    meshes: ResMut<'w, Assets<Mesh>>,
    binds: Res<'w, Assets<SkinnedMeshInverseBindposes>>,
    mesh_events: MessageReader<'w, 's, AssetEvent<Mesh>>,
    bind_events: MessageReader<'w, 's, AssetEvent<SkinnedMeshInverseBindposes>>,
    changes: Local<'s, BoundsChanges>,
}

pub(crate) fn update_skin_bounds(
    mut commands: Commands,
    sources: SkinBoundAssets<'_, '_>,
    joints: Query<&GlobalTransform>,
    changed_joints: Query<Ref<GlobalTransform>>,
    mut parts: Parts,
) {
    let SkinBoundAssets {
        mut meshes,
        binds,
        mut mesh_events,
        mut bind_events,
        mut changes,
    } = sources;
    changes.meshes.clear();
    changes.binds.clear();
    for event in mesh_events.read() {
        if let AssetEvent::Modified { id } | AssetEvent::Removed { id } = event {
            changes.meshes.insert(*id);
        }
    }
    for event in bind_events.read() {
        if let AssetEvent::Modified { id } | AssetEvent::Removed { id } = event {
            changes.binds.insert(*id);
        }
    }
    // Bounds metadata is CPU-only; refreshing it must not emit a second mesh
    // modification event or rebuild unchanged GPU geometry.
    for id in &changes.meshes {
        if let Some(mesh) = meshes.bypass_change_detection().get_mut_untracked(*id) {
            if mesh.contains_attribute(Mesh::ATTRIBUTE_JOINT_INDEX)
                && compile_skin_bounds(mesh).is_err()
            {
                mesh.set_skinned_mesh_bounds(None);
            }
        }
    }
    for (entity, mut state, mut aabb, handle, skin, transform, visible) in &mut parts {
        let source = meshes.get(&handle.0);
        let dirty = state.dirty
            || handle.is_changed()
            || skin.is_changed()
            || transform.is_changed()
            || changes.meshes.contains(&handle.0.id())
            || changes.binds.contains(&skin.inverse_bindposes.id())
            || source
                .and_then(Mesh::skinned_mesh_bounds)
                .is_some_and(|bounds| {
                    bounds.aabb_index_to_joint_index.iter().any(|index| {
                        skin.joints
                            .get(usize::from(index.0))
                            .and_then(|joint| changed_joints.get(*joint).ok())
                            .is_some_and(|joint| joint.is_changed())
                    })
                });
        if !dirty {
            continue;
        }
        if !visible.get() {
            state.dirty = true;
            continue;
        }
        let next = source
            .zip(binds.get(&skin.inverse_bindposes))
            .and_then(|(mesh, bind)| {
                if transform.affine().matrix3.determinant().abs() <= 1e-12 {
                    return None;
                }
                entity_aabb_from_skinned_mesh_bounds(&joints, mesh, &skin, bind, Some(&transform))
                    .ok()
            })
            .map(Aabb::from)
            .filter(|next| next.center.is_finite() && next.half_extents.is_finite());
        if let Some(next) = next {
            if aabb.center != next.center || aabb.half_extents != next.half_extents {
                *aabb = next;
            }
            state.dirty = false;
            #[cfg(test)]
            {
                state.updates += 1;
            }
            if state.fallback {
                commands.entity(entity).remove::<NoFrustumCulling>();
                state.fallback = false;
            }
        } else {
            state.dirty = true;
            if !state.fallback {
                commands.entity(entity).insert(NoFrustumCulling);
                state.fallback = true;
            }
        }
    }
}
#[cfg(test)]
#[path = "skin_bounds_tests.rs"]
mod tests;
