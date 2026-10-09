use crate::assets::model::ModelError;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::skinning::{JointAabb, JointIndex, SkinnedMeshBounds};
use bevy::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::VertexFormat;
use std::collections::HashMap;
use wc3::model::geometry::Geoset;
use wc3::model::V1800;

pub(crate) const EXTRA_JOINT_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("Wc3_ExtraJointIndex", 138_000_001, VertexFormat::Uint16x4);
pub(crate) const EXTRA_JOINT_WEIGHT: MeshVertexAttribute =
    MeshVertexAttribute::new("Wc3_ExtraJointWeight", 138_000_002, VertexFormat::Float32x4);

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

/// Select a layer's UV set as Bevy UV0. Authored tangents belong to UV0;
/// other sets get a generated basis matching their own texture coordinates.
pub(crate) fn canonical_uv_coordinate(geoset: &Geoset<V1800>, coordinate: u32) -> u32 {
    // UV0's authored tangent basis must remain distinct from generated bases,
    // even when its UVs match another set. Equality is exact: tolerance-based
    // merging can change sampling near atlas edges and tangent directions.
    if coordinate == 0 && geoset.tangents().is_some() {
        return coordinate;
    }
    let sets = geoset.uv_sets();
    let Some(selected) = sets.get(coordinate as usize) else {
        return coordinate;
    };
    sets.iter()
        .enumerate()
        .find(|(index, uv)| (*index != 0 || geoset.tangents().is_none()) && *uv == selected)
        .map(|(index, _)| index as u32)
        .unwrap_or(coordinate)
}

/// Build geometry with the selected UVs and their corresponding tangent basis.
pub(crate) fn build_mesh_with_uv(
    geoset: &Geoset<V1800>,
    joint_index: &HashMap<u32, u16>,
    skinned: bool,
    coordinate: u32,
) -> Result<Mesh, ModelError> {
    let mut mesh = build_mesh(geoset, joint_index, skinned)?;
    if let Some(uv) = geoset.uv_sets().get(coordinate as usize) {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv.clone());
    } else if coordinate != 0 || !geoset.uv_sets().is_empty() {
        return Err(ModelError(format!("missing UV set {coordinate}")));
    }
    if coordinate == 0 {
        if let Some(tangents) = geoset.tangents() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents.to_vec());
        }
    }
    if !mesh.contains_attribute(Mesh::ATTRIBUTE_TANGENT)
        && mesh.contains_attribute(Mesh::ATTRIBUTE_UV_0)
    {
        mesh.generate_tangents()
            .map_err(|error| ModelError(error.to_string()))?;
    }
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3::model::geometry::SkinWeights;

    #[test]
    fn selected_uv_set_uses_matching_generated_tangents() {
        let mut geoset = Geoset::<V1800>::new(
            &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[[0.0, 0.0, 1.0]; 3],
            &[0, 1, 2],
        )
        .unwrap();
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let mirrored = vec![[1.0, 0.0], [0.0, 0.0], [1.0, 1.0]];
        geoset.set_uv_sets(&[uv.clone(), uv.clone(), mirrored.clone()]);
        assert_eq!(canonical_uv_coordinate(&geoset, 1), 0);
        assert_eq!(canonical_uv_coordinate(&geoset, 2), 2);
        let authored = [[0.0, 1.0, 0.0, 1.0]; 3];
        geoset.set_tangents(Some(&authored));
        assert_eq!(canonical_uv_coordinate(&geoset, 0), 0);
        assert_eq!(canonical_uv_coordinate(&geoset, 1), 1);
        let first = build_mesh_with_uv(&geoset, &HashMap::new(), false, 0).unwrap();
        assert_eq!(
            first.attribute(Mesh::ATTRIBUTE_TANGENT),
            Some(&VertexAttributeValues::Float32x4(authored.to_vec()))
        );
        let third = build_mesh_with_uv(&geoset, &HashMap::new(), false, 2).unwrap();
        assert_eq!(
            third.attribute(Mesh::ATTRIBUTE_UV_0),
            Some(&VertexAttributeValues::Float32x2(mirrored))
        );
        let Some(VertexAttributeValues::Float32x4(tangents)) =
            third.attribute(Mesh::ATTRIBUTE_TANGENT)
        else {
            panic!("missing tangent basis");
        };
        for tangent in tangents {
            assert!(Vec4::from_array(*tangent).abs_diff_eq(Vec4::new(-1.0, 0.0, 0.0, 1.0), 1e-5));
        }
        assert!(build_mesh_with_uv(&geoset, &HashMap::new(), false, 3)
            .unwrap_err()
            .to_string()
            .contains("missing UV set 3"));
    }

    #[test]
    fn identical_uv_sets_share_but_nearby_coordinates_remain_distinct() {
        let mut geoset = Geoset::<V1800>::new(&[[0.0; 3]; 3], &[[0.0, 0.0, 1.0]; 3], &[]).unwrap();
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
        let mut near = uv.clone();
        near[0][0] += 0.000001;
        geoset.set_uv_sets(&[uv.clone(), uv.clone(), near, uv]);
        assert_eq!(canonical_uv_coordinate(&geoset, 1), 0);
        assert_eq!(canonical_uv_coordinate(&geoset, 2), 2);
        assert_eq!(canonical_uv_coordinate(&geoset, 3), 0);
        geoset.set_tangents(Some(&[[1.0, 0.0, 0.0, 1.0]; 3]));
        assert_eq!(canonical_uv_coordinate(&geoset, 0), 0);
        assert_eq!(canonical_uv_coordinate(&geoset, 3), 1);
    }

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
