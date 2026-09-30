use super::model::ModelError;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use std::collections::HashMap;
use wc3::model::geometry::Geoset;
use wc3::model::V1800;

fn vertex_skin(
    geoset: &Geoset<V1800>,
    joint_index: &HashMap<u32, u16>,
) -> Result<(Vec<[u16; 4]>, Vec<[f32; 4]>), ModelError> {
    let skin = geoset.skin_weights();
    let mut group_starts = Vec::new();
    let mut offset = 0usize;
    for size in geoset.matrix_group_sizes() {
        group_starts.push(offset);
        offset += *size as usize;
    }
    let mut indices = Vec::with_capacity(geoset.vertices().len());
    let mut weights = Vec::with_capacity(geoset.vertices().len());
    for vertex in 0..geoset.vertices().len() {
        let (ids, values) = if let Some(skin) = skin {
            let row = skin
                .get(vertex)
                .ok_or_else(|| ModelError("short SKIN section".into()))?;
            let ids = row
                .bone_indices
                .map(|index| *joint_index.get(&(index as u32)).unwrap_or(&0));
            (ids, row.weights.map(|weight| weight as f32 / 255.0))
        } else {
            let group = *geoset.vertex_groups().get(vertex).unwrap_or(&0) as usize;
            let start = *group_starts.get(group).unwrap_or(&0);
            let count = *geoset.matrix_group_sizes().get(group).unwrap_or(&0) as usize;
            if count > 4 {
                return Err(ModelError(format!(
                    "matrix group {group} has {count} influences; Bevy supports four"
                )));
            }
            let mut ids = [0u16; 4];
            let mut weights = [0.0f32; 4];
            for slot in 0..count {
                let object_id = geoset
                    .matrix_indices()
                    .get(start + slot)
                    .copied()
                    .unwrap_or(u32::MAX);
                ids[slot] = *joint_index.get(&object_id).unwrap_or(&0);
                weights[slot] = 1.0 / count as f32;
            }
            (ids, weights)
        };
        indices.push(ids);
        weights.push(values);
    }
    Ok((indices, weights))
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
        let (indices, weights) = vertex_skin(geoset, joint_index)?;
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(indices),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights);
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
        let (indices, weights) = vertex_skin(&geoset, &joints).unwrap();
        assert_eq!(indices[0][0], 3);
        assert_eq!(weights[0][0], 1.0);
    }
}
