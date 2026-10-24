//! Direct mesh projection, preserving face and corner attribute correspondence.
use super::mesh_attributes::{
    EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT, EXTRA_TEXCOORD, FOURTH_TEXCOORD, SECOND_COLOR,
};
use crate::ModelError;
use bevy::asset::RenderAssetUsages;
use bevy::math::Vec4;
use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::Mesh;
use wc3::model::{skin_weights, DrawSubset, Mesh as ModelMesh, NodeId, PrimitiveKind};
#[cfg(test)]
use wc3::model::{AttributeDomain, AttributeSemantic, AttributeValues, PrimitiveRange, Topology};
pub(super) struct GeometryPart {
    pub mesh: Mesh,
    pub vertices: Vec<usize>,
}

/// Split mixed topology into Bevy-compatible draws while preserving authored order.
#[cfg(test)]
pub(super) fn compile_geometry(
    mesh: &ModelMesh<'_>,
    subset: &DrawSubset,
    coordinate: u32,
) -> Result<Vec<GeometryPart>, ModelError> {
    compile_geometry_with_uvs(mesh, subset, coordinate, None, None)
}

#[cfg(test)]
pub(super) fn compile_geometry_with_uvs(
    mesh: &ModelMesh<'_>,
    subset: &DrawSubset,
    coordinate: u32,
    secondary: Option<u32>,
    third: Option<u32>,
) -> Result<Vec<GeometryPart>, ModelError> {
    compile_geometry_with_color(
        mesh,
        subset,
        coordinate,
        [secondary, third, None],
        |_, color| color,
    )
}

/// Apply instance-specific color interpretation while original vertex identity
/// is still available, before projection and complete-record interning.
pub(super) fn compile_geometry_with_color(
    mesh: &ModelMesh<'_>,
    subset: &DrawSubset,
    coordinate: u32,
    additional: [Option<u32>; 3],
    color: impl Fn(usize, Vec4) -> Vec4,
) -> Result<Vec<GeometryPart>, ModelError> {
    mesh.project_with_color(subset, coordinate, additional, color)
        .map_err(ModelError::from_source)?
        .into_iter()
        .map(|part| {
            let topology = match part.kind {
                PrimitiveKind::Points => PrimitiveTopology::PointList,
                PrimitiveKind::Lines => PrimitiveTopology::LineList,
                PrimitiveKind::Triangles => PrimitiveTopology::TriangleList,
            };
            let mut output = Mesh::new(topology, RenderAssetUsages::default());
            output.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions);
            output.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals);
            for (attribute, values) in [
                Mesh::ATTRIBUTE_UV_0,
                Mesh::ATTRIBUTE_UV_1,
                EXTRA_TEXCOORD,
                FOURTH_TEXCOORD,
            ]
            .into_iter()
            .zip(part.texture_coordinates)
            {
                if !values.is_empty() {
                    output.insert_attribute(attribute, values);
                }
            }
            for (attribute, values) in [Mesh::ATTRIBUTE_COLOR, SECOND_COLOR]
                .into_iter()
                .zip(part.colors)
            {
                if !values.is_empty() {
                    output.insert_attribute(attribute, values);
                }
            }
            if !part.tangents.is_empty() {
                output.insert_attribute(Mesh::ATTRIBUTE_TANGENT, part.tangents);
            } else if part.kind == PrimitiveKind::Triangles {
                let _ = output.generate_tangents();
            }
            Ok(GeometryPart {
                mesh: output,
                vertices: part.sources.into_iter().map(|c| c.vertex).collect(),
            })
        })
        .collect()
}

/// Compile joint slots without inventing a bind convention or truncating weights.
pub(super) fn apply_skin(
    part: &mut GeometryPart,
    source: &ModelMesh<'_>,
    joints: &[NodeId],
) -> Result<Vec<usize>, ModelError> {
    let resolved = skin_weights(part.vertices.iter().copied(), source, joints)
        .map_err(ModelError::from_source)?;
    if let Some(row) = resolved.rows.iter().find(|row| row.len() > 8) {
        return Err(ModelError::new(format!(
            "{} active influences exceed the Universal PBR limit of eight",
            row.len()
        )));
    }
    let palette = resolved.palette;
    let contributions = resolved.rows;
    if palette.len() > usize::from(u16::MAX) + 1 {
        return Err(ModelError::new("compiled skin palette exceeds u16"));
    }
    let mut indices = Vec::with_capacity(contributions.len());
    let mut weights = Vec::with_capacity(contributions.len());
    let mut extra_indices = Vec::with_capacity(contributions.len());
    let mut extra_weights = Vec::with_capacity(contributions.len());
    let mut has_extra = false;
    for active in contributions {
        let mut ids = [0u16; 4];
        let mut ws = [0f32; 4];
        let mut extra_ids = [0u16; 4];
        let mut extra_ws = [0f32; 4];
        for (slot, (joint, weight)) in active.into_iter().enumerate() {
            let id =
                u16::try_from(joint).map_err(|_| ModelError::new("skin palette exceeds u16"))?;
            if slot < 4 {
                ids[slot] = id;
                ws[slot] = weight;
            } else {
                extra_ids[slot - 4] = id;
                extra_ws[slot - 4] = weight;
                has_extra = true;
            }
        }
        indices.push(ids);
        weights.push(ws);
        extra_indices.push(extra_ids);
        extra_weights.push(extra_ws);
    }
    part.mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(indices),
    );
    part.mesh
        .insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights);
    if has_extra {
        part.mesh.insert_attribute(
            EXTRA_JOINT_INDEX,
            VertexAttributeValues::Uint16x4(extra_indices),
        );
        part.mesh
            .insert_attribute(EXTRA_JOINT_WEIGHT, extra_weights);
    }
    Ok(palette)
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
