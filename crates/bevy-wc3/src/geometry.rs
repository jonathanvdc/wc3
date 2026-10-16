//! Direct mesh projection, preserving face and corner attribute correspondence.
use super::mesh_attributes::{
    EXTRA_JOINT_INDEX, EXTRA_JOINT_WEIGHT, EXTRA_TEXCOORD, FOURTH_TEXCOORD, SECOND_COLOR,
};
use crate::ModelError;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::{Mesh, Vec2, Vec3};
use wc3::model::{
    AttributeDomain, AttributeSemantic, AttributeValues, DrawSubset, GeometrySchema as _,
    Mesh as ModelMesh, NodeId, PrimitiveRange, PrimitiveSelection, Topology, VertexInfluences,
};

#[derive(Clone, Copy)]
struct Corner {
    vertex: usize,
    face: usize,
    corner: usize,
}

struct Primitive {
    corners: Vec<Corner>,
}

fn primitives(
    topology: &Topology<'_>,
    first: usize,
    count: usize,
) -> Result<Vec<Primitive>, ModelError> {
    let end = first
        .checked_add(count)
        .ok_or_else(|| ModelError("primitive range overflow".into()))?;
    if end > topology.primitive_count() {
        return Err(ModelError("draw range exceeds connectivity".into()));
    }
    fn visit(
        topology: &Topology<'_>,
        first: usize,
        end: usize,
        base_face: usize,
        base_corner: usize,
        output: &mut Vec<Primitive>,
    ) {
        let local_first = first.saturating_sub(base_face);
        let local_end = end
            .saturating_sub(base_face)
            .min(topology.primitive_count());
        if local_first >= local_end {
            return;
        }
        let mut push = |face: usize, corner: usize, vertices: Vec<u32>| {
            output.push(Primitive {
                corners: vertices
                    .into_iter()
                    .enumerate()
                    .map(|(i, vertex)| Corner {
                        vertex: vertex as usize,
                        face: base_face + face,
                        corner: base_corner + corner + i,
                    })
                    .collect(),
            });
        };
        match topology {
            Topology::Triangles(values) => {
                for i in local_first..local_end {
                    push(i, i * 3, values.get(i).unwrap().to_vec());
                }
            }
            Topology::Lines(values) => {
                for i in local_first..local_end {
                    push(i, i * 2, values.get(i).unwrap().to_vec());
                }
            }
            Topology::Points(values) => {
                for i in local_first..local_end {
                    push(i, i, vec![values.get(i).unwrap()]);
                }
            }
            Topology::TriangleStrip(values) => {
                for i in local_first..local_end {
                    let mut vertices = [
                        values.get(i).unwrap(),
                        values.get(i + 1).unwrap(),
                        values.get(i + 2).unwrap(),
                    ];
                    if i % 2 == 1 {
                        vertices.swap(0, 1);
                    }
                    push(i, i * 3, vertices.to_vec());
                }
            }
            Topology::Polygons { corners, offsets } => {
                for i in local_first..local_end {
                    let start = offsets.get(i).unwrap() as usize;
                    let end = offsets.get(i + 1).unwrap() as usize;
                    push(
                        i,
                        start,
                        (start..end).map(|j| corners.get(j).unwrap()).collect(),
                    );
                }
            }
            Topology::Mixed(groups) => {
                let mut face = base_face;
                let mut corner = base_corner;
                for group in groups {
                    if face >= end {
                        break;
                    }
                    visit(group, first, end, face, corner, output);
                    face += group.primitive_count();
                    corner += group.corner_count();
                }
            }
        }
    }
    let mut output = Vec::with_capacity(count);
    visit(topology, first, end, 0, 0, &mut output);
    Ok(output)
}

fn cross(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b - a).perp_dot(c - a)
}

// Ear clipping retains concave polygon boundaries; a fan would fill exterior areas.
fn triangulate(mesh: &ModelMesh<'_>, polygon: &[Corner]) -> Result<Vec<Corner>, ModelError> {
    if polygon.len() == 3 {
        return Ok(polygon.to_vec());
    }
    let positions: Vec<_> = polygon
        .iter()
        .map(|c| Vec3::from_array(mesh.positions().get(c.vertex).unwrap()))
        .collect();
    let mut normal = Vec3::ZERO;
    for index in 0..positions.len() {
        normal += positions[index].cross(positions[(index + 1) % positions.len()]);
    }
    let axis = normal.abs().max_element();
    let project = |p: Vec3| {
        if normal.x.abs() == axis {
            Vec2::new(p.y, p.z)
        } else if normal.y.abs() == axis {
            Vec2::new(p.x, p.z)
        } else {
            Vec2::new(p.x, p.y)
        }
    };
    let projected: Vec<_> = positions.into_iter().map(project).collect();
    let area: f32 = (0..projected.len())
        .map(|i| projected[i].perp_dot(projected[(i + 1) % projected.len()]))
        .sum();
    if area == 0. {
        return Err(ModelError("degenerate polygon has no triangulation".into()));
    }
    let sign = area.signum();
    let mut remaining: Vec<_> = (0..polygon.len()).collect();
    let mut result = Vec::new();
    while remaining.len() > 3 {
        let ear = (0..remaining.len())
            .find(|&i| {
                let a = remaining[(i + remaining.len() - 1) % remaining.len()];
                let b = remaining[i];
                let c = remaining[(i + 1) % remaining.len()];
                if cross(projected[a], projected[b], projected[c]) * sign <= 0. {
                    return false;
                }
                !remaining.iter().any(|&p| {
                    p != a
                        && p != b
                        && p != c
                        && cross(projected[a], projected[b], projected[p]) * sign >= 0.
                        && cross(projected[b], projected[c], projected[p]) * sign >= 0.
                        && cross(projected[c], projected[a], projected[p]) * sign >= 0.
                })
            })
            .ok_or_else(|| ModelError("polygon is degenerate or self-intersecting".into()))?;
        result.extend([
            polygon[remaining[(ear + remaining.len() - 1) % remaining.len()]],
            polygon[remaining[ear]],
            polygon[remaining[(ear + 1) % remaining.len()]],
        ]);
        remaining.remove(ear);
    }
    result.extend(remaining.into_iter().map(|i| polygon[i]));
    Ok(result)
}

fn attribute<'m, 'a>(
    mesh: &'m ModelMesh<'a>,
    subset: &DrawSubset,
    semantic: &AttributeSemantic<'_>,
    c: Corner,
) -> Result<Option<(&'m AttributeValues<'a>, usize)>, ModelError> {
    // Most specific authored correspondence wins over a vertex stream.
    for (domain, index) in [
        (
            AttributeDomain::ConnectivityCorner(subset.selection.connectivity),
            c.corner,
        ),
        (
            AttributeDomain::ConnectivityFace(subset.selection.connectivity),
            c.face,
        ),
        (AttributeDomain::Corner, c.corner),
        (AttributeDomain::Face, c.face),
        (AttributeDomain::Vertex, c.vertex),
        (AttributeDomain::Mesh, 0),
    ] {
        if let Some(value) = mesh.geometry.attributes.get(semantic, domain) {
            if matches!(domain, AttributeDomain::Corner | AttributeDomain::Face)
                && subset.selection.connectivity != mesh.subsets.primary()
            {
                continue;
            }
            if value.values.is_empty() {
                continue;
            }
            if index >= value.values.len() {
                return Err(ModelError(format!(
                    "{semantic:?} correspondence exceeds attribute values"
                )));
            }
            return Ok(Some((&value.values, index)));
        }
    }
    if mesh
        .geometry
        .attributes
        .get(semantic, AttributeDomain::Pool)
        .is_some()
    {
        return Err(ModelError(format!(
            "{semantic:?} requires an explicit attribute-pool binding"
        )));
    }
    Ok(None)
}

pub(super) struct GeometryPart {
    pub mesh: Mesh,
    pub vertices: Vec<usize>,
}

/// Select the full primary topology without inventing a draw or subset identity.
pub(super) fn whole_geometry(mesh: &ModelMesh<'_>) -> Result<DrawSubset, ModelError> {
    let connectivity = mesh.subsets.primary();
    let topology = mesh
        .connectivity(connectivity)
        .ok_or_else(|| ModelError("primary connectivity is absent".into()))?;
    let count = u32::try_from(topology.primitive_count())
        .map_err(|_| ModelError("whole geometry exceeds u32 primitive range".into()))?;
    Ok(DrawSubset::new(PrimitiveSelection {
        connectivity,
        range: PrimitiveRange::Primitives { first: 0, count },
    }))
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
    color: impl Fn(usize, [f32; 4]) -> [f32; 4],
) -> Result<Vec<GeometryPart>, ModelError> {
    let [secondary, third, fourth] = additional;
    let topology = mesh
        .connectivity(subset.selection.connectivity)
        .ok_or_else(|| ModelError("draw connectivity is absent".into()))?;
    let (first, count, restart) = match subset.selection.range {
        PrimitiveRange::Primitives { first, count } => (first as usize, count as usize, false),
        PrimitiveRange::StripIndices { first, count } => {
            let Topology::TriangleStrip(values) = &topology else {
                return Err(ModelError("strip selection requires strip topology".into()));
            };
            if first as usize + count as usize > values.len() {
                return Err(ModelError("strip range exceeds connectivity".into()));
            }
            if count < 3 {
                return Ok(Vec::new());
            }
            (first as usize, count as usize - 2, first % 2 == 1)
        }
    };
    let mut selected = primitives(&topology, first, count)?;
    if restart {
        for primitive in &mut selected {
            primitive.corners.swap(0, 1);
        }
    }
    let mut batches: Vec<(usize, Vec<Corner>)> = Vec::new();
    for primitive in &selected {
        let size = primitive.corners.len().min(3);
        let values = if size == 3 {
            triangulate(mesh, &primitive.corners)?
        } else {
            primitive.corners.clone()
        };
        if let Some((previous, corners)) =
            batches.last_mut().filter(|(previous, _)| *previous == size)
        {
            let _ = previous;
            corners.extend(values);
        } else {
            batches.push((size, values));
        }
    }
    batches
        .into_iter()
        .map(|(size, corners)| {
            let topology = match size {
                1 => PrimitiveTopology::PointList,
                2 => PrimitiveTopology::LineList,
                _ => PrimitiveTopology::TriangleList,
            };
            let mut output = Mesh::new(topology, RenderAssetUsages::default());
            let positions: Vec<_> = corners
                .iter()
                .map(|c| mesh.positions().get(c.vertex).unwrap())
                .collect();
            output.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            let mut secondary_uvs = Vec::new();
            let mut third_uvs = Vec::new();
            let mut fourth_uvs = Vec::new();
            let mut colors = Vec::new();
            let mut second_colors = Vec::new();
            let mut tangents = Vec::new();
            for &c in &corners {
                if let Some((AttributeValues::Vector3(values), index)) =
                    attribute(mesh, subset, &AttributeSemantic::Normal, c)?
                {
                    normals.push(values.get(index).unwrap());
                }
                if let Some((AttributeValues::Vector2(values), index)) =
                    attribute(mesh, subset, &AttributeSemantic::TexCoord(coordinate), c)?
                {
                    uvs.push(values.get(index).unwrap());
                }
                for (coordinate, target) in [
                    (secondary, &mut secondary_uvs),
                    (third, &mut third_uvs),
                    (fourth, &mut fourth_uvs),
                ] {
                    if let Some(coordinate) = coordinate {
                        if let Some((AttributeValues::Vector2(values), index)) =
                            attribute(mesh, subset, &AttributeSemantic::TexCoord(coordinate), c)?
                        {
                            target.push(values.get(index).unwrap());
                        } else {
                            return Err(ModelError(format!(
                                "texture coordinate set {coordinate} is absent"
                            )));
                        }
                    }
                }
                if let Some((AttributeValues::Vector4(values), index)) =
                    attribute(mesh, subset, &AttributeSemantic::Color(0), c)?
                {
                    colors.push(color(c.vertex, values.get(index).unwrap()));
                }
                if let Some((AttributeValues::Vector4(values), index)) =
                    attribute(mesh, subset, &AttributeSemantic::Color(1), c)?
                {
                    second_colors.push(values.get(index).unwrap());
                }
                if coordinate == 0 {
                    if let Some(binding) = subset.tangents {
                        let pool = mesh
                            .geometry
                            .tangent_pool()
                            .ok_or_else(|| ModelError("tangent pool is absent".into()))?;
                        let index = c
                            .vertex
                            .checked_sub(binding.first_vertex as usize)
                            .filter(|i| *i < binding.vertex_count as usize)
                            .ok_or_else(|| ModelError("vertex outside tangent binding".into()))?;
                        tangents.push(
                            pool.get(binding.first_tangent as usize + index)
                                .ok_or_else(|| ModelError("tangent binding exceeds pool".into()))?,
                        );
                    } else if let Some((AttributeValues::Vector4(values), index)) =
                        attribute(mesh, subset, &AttributeSemantic::Tangent, c)?
                    {
                        tangents.push(values.get(index).unwrap());
                    }
                }
            }
            if !normals.is_empty() {
                output.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            } else if size == 3 {
                output.compute_flat_normals();
            } else {
                output.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; corners.len()]);
            }
            if !colors.is_empty() {
                output.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            }
            if !second_colors.is_empty() {
                output.insert_attribute(SECOND_COLOR, second_colors);
            }
            if secondary.is_some() {
                output.insert_attribute(Mesh::ATTRIBUTE_UV_1, secondary_uvs);
            }
            if third.is_some() {
                output.insert_attribute(EXTRA_TEXCOORD, third_uvs);
            }
            if fourth.is_some() {
                output.insert_attribute(FOURTH_TEXCOORD, fourth_uvs);
            }
            if !uvs.is_empty() {
                output.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            } else if coordinate != 0 {
                return Err(ModelError(format!(
                    "texture coordinate set {coordinate} is absent"
                )));
            } else {
                output.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.; 2]; corners.len()]);
            }
            if !tangents.is_empty() {
                output.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
            } else if size == 3 {
                let _ = output.generate_tangents();
            }
            Ok(GeometryPart {
                mesh: output,
                vertices: corners.iter().map(|c| c.vertex).collect(),
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
    let rows = source.influences();
    let mut contributions = Vec::with_capacity(part.vertices.len());
    let mut palette = Vec::new();
    for &vertex in &part.vertices {
        let values = match rows.get(vertex) {
            None => Vec::new(),
            Some(VertexInfluences::Joints(values)) => values
                .into_iter()
                .map(|v| (v.joint as usize, v.weight))
                .collect(),
            Some(VertexInfluences::Nodes(values)) => values
                .into_iter()
                .map(|v| {
                    joints
                        .iter()
                        .position(|id| *id == v.node)
                        .map(|i| (i, v.weight))
                        .ok_or_else(|| ModelError("influencing node is absent from skin".into()))
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        let active: Vec<_> = values.into_iter().filter(|(_, w)| *w > 0.).collect();
        if active.len() > 8 {
            return Err(ModelError(format!(
                "{} active influences exceed the Universal PBR limit of eight",
                active.len()
            )));
        }
        let sum: f32 = active.iter().map(|(_, w)| w).sum();
        if sum <= 0. {
            return Err(ModelError(
                "skinned vertex has no positive joint contribution".into(),
            ));
        }
        for &(joint, _) in &active {
            if joint >= joints.len() {
                return Err(ModelError("joint contribution exceeds skin palette".into()));
            }
            palette.push(joint);
        }
        contributions.push((active, sum));
    }
    palette.sort_unstable();
    palette.dedup();
    if palette.len() > usize::from(u16::MAX) + 1 {
        return Err(ModelError("compiled skin palette exceeds u16".into()));
    }
    let mut indices = Vec::with_capacity(contributions.len());
    let mut weights = Vec::with_capacity(contributions.len());
    let mut extra_indices = Vec::with_capacity(contributions.len());
    let mut extra_weights = Vec::with_capacity(contributions.len());
    let mut has_extra = false;
    for (active, sum) in contributions {
        let mut ids = [0u16; 4];
        let mut ws = [0f32; 4];
        let mut extra_ids = [0u16; 4];
        let mut extra_ws = [0f32; 4];
        for (slot, (joint, weight)) in active.into_iter().enumerate() {
            let joint = palette.binary_search(&joint).expect("collected skin joint");
            let id =
                u16::try_from(joint).map_err(|_| ModelError("skin palette exceeds u16".into()))?;
            if slot < 4 {
                ids[slot] = id;
                ws[slot] = weight / sum;
            } else {
                extra_ids[slot - 4] = id;
                extra_ws[slot - 4] = weight / sum;
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
