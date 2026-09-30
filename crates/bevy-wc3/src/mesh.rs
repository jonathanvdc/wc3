use super::model::ModelError;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::skinning::{JointAabb, JointIndex, SkinnedMeshBounds};
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::VertexFormat;
use std::collections::HashMap;
use wc3::model::geometry::Geoset;
use wc3::model::V1800;

pub(crate) const EXTRA_JOINT_INDEX: bevy::mesh::MeshVertexAttribute =
    bevy::mesh::MeshVertexAttribute::new(
        "Wc3_ExtraJointIndex",
        138_000_001,
        VertexFormat::Uint16x4,
    );
pub(crate) const EXTRA_JOINT_WEIGHT: bevy::mesh::MeshVertexAttribute =
    bevy::mesh::MeshVertexAttribute::new(
        "Wc3_ExtraJointWeight",
        138_000_002,
        VertexFormat::Float32x4,
    );

type JointRows = (Vec<[u16; 4]>, Vec<[f32; 4]>, Vec<[u16; 4]>, Vec<[f32; 4]>);

fn vertex_skin(
    geoset: &Geoset<V1800>,
    joint_index: &HashMap<u32, u16>,
) -> Result<JointRows, ModelError> {
    let skin = geoset.skin_weights();
    let mut group_starts = Vec::new();
    let mut offset = 0usize;
    for size in geoset.matrix_group_sizes() {
        group_starts.push(offset);
        offset += *size as usize;
    }
    let mut indices = Vec::with_capacity(geoset.vertices().len());
    let mut weights = Vec::with_capacity(geoset.vertices().len());
    let mut extra_indices = Vec::with_capacity(geoset.vertices().len());
    let mut extra_weights = Vec::with_capacity(geoset.vertices().len());
    for vertex in 0..geoset.vertices().len() {
        let (ids, values, extra_ids, extra_values) = if let Some(skin) = skin {
            let row = skin
                .get(vertex)
                .ok_or_else(|| ModelError("short SKIN section".into()))?;
            let ids = row
                .bone_indices
                .map(|index| *joint_index.get(&(index as u32)).unwrap_or(&0));
            (
                ids,
                row.weights.map(|weight| weight as f32 / 255.0),
                [0; 4],
                [0.0; 4],
            )
        } else {
            let group = *geoset.vertex_groups().get(vertex).unwrap_or(&0) as usize;
            let start = *group_starts.get(group).unwrap_or(&0);
            let count = *geoset.matrix_group_sizes().get(group).unwrap_or(&0) as usize;
            if count > 8 {
                return Err(ModelError(format!(
                    "matrix group {group} has {count} influences; WC3 GPU skinning supports eight"
                )));
            }
            let mut ids = [0u16; 4];
            let mut weights = [0.0f32; 4];
            let mut extra_ids = [0u16; 4];
            let mut extra_weights = [0.0f32; 4];
            for slot in 0..count {
                let object_id = geoset
                    .matrix_indices()
                    .get(start + slot)
                    .copied()
                    .unwrap_or(u32::MAX);
                let id = *joint_index.get(&object_id).unwrap_or(&0);
                let weight = 1.0 / count as f32;
                if slot < 4 {
                    ids[slot] = id;
                    weights[slot] = weight;
                } else {
                    extra_ids[slot - 4] = id;
                    extra_weights[slot - 4] = weight;
                }
            }
            (ids, weights, extra_ids, extra_weights)
        };
        indices.push(ids);
        weights.push(values);
        extra_indices.push(extra_ids);
        extra_weights.push(extra_values);
    }
    Ok((indices, weights, extra_indices, extra_weights))
}

fn skin_bounds(positions: &[[f32; 3]], rows: &JointRows) -> SkinnedMeshBounds {
    let mut ranges: Vec<Option<(Vec3, Vec3)>> = Vec::new();
    for (vertex, &position) in positions.iter().enumerate() {
        let position = Vec3::from_array(position);
        for (ids, weights) in [
            (&rows.0[vertex], &rows.1[vertex]),
            (&rows.2[vertex], &rows.3[vertex]),
        ] {
            for (&id, &weight) in ids.iter().zip(weights.iter()) {
                if weight <= 0.0 {
                    continue;
                }
                let index = id as usize;
                if ranges.len() <= index {
                    ranges.resize(index + 1, None);
                }
                ranges[index] = Some(match ranges[index] {
                    Some((min, max)) => (min.min(position), max.max(position)),
                    None => (position, position),
                });
            }
        }
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
    bounds
}

pub(crate) fn build_mesh(
    geoset: &Geoset<V1800>,
    joint_index: &HashMap<u32, u16>,
    skinned: bool,
) -> Result<Mesh, ModelError> {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, geoset.vertices().to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, geoset.normals().to_vec());
    if let Some(uv) = geoset.uv_sets().first() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv.clone());
    }
    if skinned {
        let rows = vertex_skin(geoset, joint_index)?;
        mesh.set_skinned_mesh_bounds(Some(skin_bounds(geoset.vertices(), &rows)));
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(rows.0),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, rows.1);
        mesh.insert_attribute(EXTRA_JOINT_INDEX, VertexAttributeValues::Uint16x4(rows.2));
        mesh.insert_attribute(EXTRA_JOINT_WEIGHT, rows.3);
    }
    mesh.insert_indices(Indices::U16(geoset.face_indices().to_vec()));
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::geometry::SkinWeights;

    #[test]
    fn reforged_skin_indices_address_nodes_directly() {
        let mut geoset = Geoset::<V1800>::new(&[[0.0; 3]], &[[0.0, 0.0, 1.0]], &[]).unwrap();
        geoset
            .set_skin_weights(Some(&[SkinWeights {
                bone_indices: [7, 0, 0, 0],
                weights: [255, 0, 0, 0],
            }]))
            .unwrap();
        let joints = HashMap::from([(7, 3)]);
        let rows = vertex_skin(&geoset, &joints).unwrap();
        assert_eq!(rows.0[0][0], 3);
        assert_eq!(rows.1[0][0], 1.0);
    }

    #[test]
    fn classic_five_bone_group_retains_all_influences() {
        let mut geoset = Geoset::<V1800>::new(&[[0.0; 3]], &[[0.0, 0.0, 1.0]], &[]).unwrap();
        geoset.set_matrix_groups(&[vec![1, 2, 3, 4, 5]]).unwrap();
        let joints = HashMap::from([(1, 0), (2, 1), (3, 2), (4, 3), (5, 4)]);
        let rows = vertex_skin(&geoset, &joints).unwrap();
        assert_eq!(rows.0[0], [0, 1, 2, 3]);
        assert_eq!(rows.1[0], [0.2; 4]);
        assert_eq!(rows.2[0], [4, 0, 0, 0]);
        assert_eq!(rows.3[0], [0.2, 0.0, 0.0, 0.0]);
        let bounds = skin_bounds(geoset.vertices(), &rows);
        assert_eq!(bounds.aabb_index_to_joint_index.len(), 5);
    }

    #[test]
    fn classic_nine_bone_group_reports_the_limit() {
        let mut geoset = Geoset::<V1800>::new(&[[0.0; 3]], &[[0.0, 0.0, 1.0]], &[]).unwrap();
        geoset.set_matrix_groups(&[(0..9).collect()]).unwrap();
        let error = vertex_skin(&geoset, &HashMap::new()).unwrap_err();
        assert!(error
            .to_string()
            .contains("matrix group 0 has 9 influences"));
    }
}
