//! Draw projection with explicit source correspondence, independent of a renderer.
use crate::model::error::{
    AttributeValueKind, GeometryError, Location, TopologyKind, TriangulationFailure,
};
use crate::model::ConnectivityId;
use crate::model::{
    AttributeDomain, AttributeSemantic, AttributeValues, DrawSubset, Error, Mesh, NodeId,
    PrimitiveRange, PrimitiveSelection, Topology, VertexInfluences,
};
use glam::Vec4;
use glam::{Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Original correspondence of one projected vertex.
pub struct SourceCorner {
    /// Original vertex index.
    pub vertex: usize,
    /// Original primitive index within the selected connectivity.
    pub face: usize,
    /// Original primitive-corner index within the selected connectivity.
    pub corner: usize,
}

struct PrimitiveCorners {
    corners: Vec<SourceCorner>,
    offsets: Vec<usize>,
}
impl PrimitiveCorners {
    fn push(&mut self, face: usize, corner: usize, vertices: impl IntoIterator<Item = u32>) {
        self.corners.extend(
            vertices
                .into_iter()
                .enumerate()
                .map(|(i, vertex)| SourceCorner {
                    vertex: vertex as usize,
                    face,
                    corner: corner + i,
                }),
        );
        self.offsets.push(self.corners.len());
    }

    fn iter(&self) -> impl Iterator<Item = &[SourceCorner]> {
        self.offsets
            .windows(2)
            .map(|range| &self.corners[range[0]..range[1]])
    }
}

fn primitives(
    topology: &Topology<'_>,
    connectivity: ConnectivityId,
    first: usize,
    count: usize,
) -> Result<PrimitiveCorners, Error> {
    let end = first.checked_add(count).ok_or_else(|| {
        Error::at(
            Location::new().field("range"),
            GeometryError::RangeOverflow {
                first: first as u32,
                count: count as u32,
            },
        )
    })?;
    if end > topology.primitive_count() {
        return Err(Error::at(
            Location::new().field("range"),
            GeometryError::SelectionRange {
                connectivity,
                range: PrimitiveRange::Primitives {
                    first: first as u32,
                    count: count as u32,
                },
                capacity: topology.primitive_count(),
            },
        ));
    }
    fn visit(
        topology: &Topology<'_>,
        first: usize,
        end: usize,
        base_face: usize,
        base_corner: usize,
        output: &mut PrimitiveCorners,
    ) {
        let local_first = first.saturating_sub(base_face);
        let local_end = end
            .saturating_sub(base_face)
            .min(topology.primitive_count());
        if local_first >= local_end {
            return;
        }
        match topology {
            Topology::Triangles(values) => {
                for i in local_first..local_end {
                    output.push(base_face + i, base_corner + i * 3, values.get(i).unwrap());
                }
            }
            Topology::Lines(values) => {
                for i in local_first..local_end {
                    output.push(base_face + i, base_corner + i * 2, values.get(i).unwrap());
                }
            }
            Topology::Points(values) => {
                for i in local_first..local_end {
                    output.push(base_face + i, base_corner + i, [values.get(i).unwrap()]);
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
                    output.push(base_face + i, base_corner + i * 3, vertices);
                }
            }
            Topology::Polygons { corners, offsets } => {
                for i in local_first..local_end {
                    let start = offsets.get(i).unwrap() as usize;
                    let end = offsets.get(i + 1).unwrap() as usize;
                    output.push(
                        base_face + i,
                        base_corner + start,
                        (start..end).map(|j| corners.get(j).unwrap()),
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
    let corner_capacity = if first == 0 && count == topology.primitive_count() {
        topology.corner_count()
    } else {
        count
    };
    let mut output = PrimitiveCorners {
        corners: Vec::with_capacity(corner_capacity),
        offsets: Vec::with_capacity(count + 1),
    };
    output.offsets.push(0);
    visit(topology, first, end, 0, 0, &mut output);
    Ok(output)
}

fn cross(a: Vec2, b: Vec2, c: Vec2) -> f32 {
    (b - a).perp_dot(c - a)
}

// Ear clipping retains concave polygon boundaries; a fan would fill exterior areas.
fn triangulate(
    mesh: &Mesh<'_>,
    polygon: &[SourceCorner],
    result: &mut Vec<SourceCorner>,
) -> Result<(), Error> {
    if polygon.len() == 3 {
        result.extend_from_slice(polygon);
        return Ok(());
    }
    let positions: Vec<_> = polygon
        .iter()
        .map(|c| mesh.positions().get(c.vertex).unwrap())
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
        return Err(Error::at(
            Location::new().field("topology").index(polygon[0].face),
            GeometryError::Triangulation {
                reason: TriangulationFailure::ZeroArea,
                corners: polygon.len(),
            },
        ));
    }
    let sign = area.signum();
    let mut remaining: Vec<_> = (0..polygon.len()).collect();
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
            .ok_or_else(|| {
                Error::at(
                    Location::new().field("topology").index(polygon[0].face),
                    GeometryError::Triangulation {
                        reason: TriangulationFailure::NoEar,
                        corners: polygon.len(),
                    },
                )
            })?;
        result.extend([
            polygon[remaining[(ear + remaining.len() - 1) % remaining.len()]],
            polygon[remaining[ear]],
            polygon[remaining[(ear + 1) % remaining.len()]],
        ]);
        remaining.remove(ear);
    }
    result.extend(remaining.into_iter().map(|i| polygon[i]));
    Ok(())
}

#[derive(Clone, Copy)]
struct ResolvedAttribute<'view, 'data> {
    values: &'view AttributeValues<'data>,
    index: usize,
    domain: AttributeDomain,
}

fn attribute<'m, 'a>(
    mesh: &'m Mesh<'a>,
    subset: &DrawSubset,
    semantic: &AttributeSemantic<'_>,
    c: SourceCorner,
) -> Result<Option<ResolvedAttribute<'m, 'a>>, Error> {
    if let Some(binding) = subset
        .tangents
        .filter(|_| matches!(semantic, AttributeSemantic::Tangent))
    {
        let values = &mesh
            .geometry
            .attributes
            .get(semantic, AttributeDomain::Pool)
            .ok_or_else(|| {
                Error::at(
                    Location::new().field("tangents"),
                    GeometryError::MissingTangentPool,
                )
            })?
            .values;
        let offset = c
            .vertex
            .checked_sub(binding.first_vertex as usize)
            .filter(|i| *i < binding.vertex_count as usize)
            .ok_or_else(|| {
                Error::at(
                    Location::new().field("tangents"),
                    GeometryError::TangentVertexWindow {
                        vertex: c.vertex,
                        first: binding.first_vertex,
                        count: binding.vertex_count,
                    },
                )
            })?;
        let index = binding.first_tangent as usize + offset;
        if index >= values.len() {
            return Err(Error::at(
                Location::new().field("tangents"),
                GeometryError::IndexOutOfBounds {
                    index,
                    len: values.len(),
                },
            ));
        }
        return Ok(Some(ResolvedAttribute {
            values,
            index,
            domain: AttributeDomain::Pool,
        }));
    }
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
                return Err(Error::at(
                    Location::new()
                        .attribute(semantic.clone(), domain)
                        .index(index),
                    GeometryError::IndexOutOfBounds {
                        index,
                        len: value.values.len(),
                    },
                ));
            }
            return Ok(Some(ResolvedAttribute {
                values: &value.values,
                index,
                domain,
            }));
        }
    }
    if mesh
        .geometry
        .attributes
        .get(semantic, AttributeDomain::Pool)
        .is_some()
    {
        return Err(Error::at(
            Location::new().attribute(semantic.clone(), AttributeDomain::Pool),
            GeometryError::MissingPoolBinding,
        ));
    }
    Ok(None)
}

/// Primitive family of a projected batch. Vertices are in primitive order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveKind {
    /// Independent points.
    Points,
    /// Independent line pairs.
    Lines,
    /// Independent triangle triples.
    Triangles,
}
/// Owned editable projected streams. Empty streams are absent.
/// `sources` retains the original vertex, face, and corner for application attributes.
#[derive(Clone, Debug)]
pub struct ProjectedGeometry {
    /// Primitive family of this batch.
    pub kind: PrimitiveKind,
    /// Editable authored-space positions.
    pub positions: Vec<Vec3>,
    /// Normals, generated for triangle batches when absent.
    pub normals: Vec<Vec3>,
    /// Authored tangent directions and handedness; empty when absent.
    pub tangents: Vec<Vec4>,
    /// Requested primary and additional UV streams in slot order.
    pub texture_coordinates: [Vec<Vec2>; 4],
    /// Primary and secondary color streams.
    pub colors: [Vec<Vec4>; 2],
    /// Original vertex, primitive, and corner identities for each output vertex.
    pub sources: Vec<SourceCorner>,
}
impl ProjectedGeometry {
    fn new(kind: PrimitiveKind) -> Self {
        Self {
            kind,
            positions: Vec::new(),
            normals: Vec::new(),
            tangents: Vec::new(),
            texture_coordinates: Default::default(),
            colors: Default::default(),
            sources: Vec::new(),
        }
    }
    /// Resolve and normalize source skin contributions without imposing GPU influence limits.
    /// Returns a compact palette into `joints`, and one variable-length row per projected vertex.
    pub fn skin_weights(&self, source: &Mesh<'_>, joints: &[NodeId]) -> Result<SkinWeights, Error> {
        skin_weights(self.sources.iter().map(|c| c.vertex), source, joints)
    }
}
/// Compact, normalized skin distribution. Rows reference `palette`, whose entries select source joints.
#[derive(Clone, Debug)]
pub struct SkinWeights {
    /// Source skin joint slots retained by positive contributions.
    pub palette: Vec<usize>,
    /// Normalized variable-length rows referencing the compact palette.
    pub rows: Vec<Vec<(usize, f32)>>,
}
/// Select the full primary topology without inventing a draw or subset identity.
fn whole_geometry(mesh: &Mesh<'_>) -> Result<DrawSubset, Error> {
    let connectivity = mesh.subsets.primary();
    let topology = mesh.connectivity(connectivity).ok_or_else(|| {
        Error::at(
            Location::new().field("connectivity"),
            GeometryError::MissingConnectivity { connectivity },
        )
    })?;
    let count = u32::try_from(topology.primitive_count()).map_err(|_| {
        Error::at(
            Location::new().field("topology"),
            GeometryError::CountOverflow {
                actual: topology.primitive_count(),
                maximum: u32::MAX as usize,
            },
        )
    })?;
    Ok(DrawSubset::new(PrimitiveSelection {
        connectivity,
        range: PrimitiveRange::Primitives { first: 0, count },
    }))
}

/// Apply instance-specific color interpretation while original vertex identity
/// is still available, before projection and complete-record interning.
fn project_geometry_with_color(
    mesh: &Mesh<'_>,
    subset: &DrawSubset,
    coordinate: u32,
    additional: [Option<u32>; 3],
    color: impl Fn(usize, Vec4) -> Vec4,
) -> Result<Vec<ProjectedGeometry>, Error> {
    let [secondary, third, fourth] = additional;
    let topology = mesh
        .connectivity(subset.selection.connectivity)
        .ok_or_else(|| {
            Error::at(
                Location::new().field("connectivity"),
                GeometryError::MissingConnectivity {
                    connectivity: subset.selection.connectivity,
                },
            )
        })?;
    let (first, count, restart) = match subset.selection.range {
        PrimitiveRange::Primitives { first, count } => (first as usize, count as usize, false),
        PrimitiveRange::StripIndices { first, count } => {
            let Topology::TriangleStrip(values) = &topology else {
                return Err(Error::at(
                    Location::new().field("range"),
                    GeometryError::SelectionDomain {
                        topology: TopologyKind::from(&topology),
                        range: subset.selection.range,
                    },
                ));
            };
            if first as usize + count as usize > values.len() {
                return Err(Error::at(
                    Location::new().field("range"),
                    GeometryError::SelectionRange {
                        connectivity: subset.selection.connectivity,
                        range: subset.selection.range,
                        capacity: values.len(),
                    },
                ));
            }
            if count < 3 {
                return Ok(Vec::new());
            }
            (first as usize, count as usize - 2, first % 2 == 1)
        }
    };
    let mut selected = primitives(&topology, subset.selection.connectivity, first, count)?;
    if restart {
        for corners in selected.corners.as_chunks_mut::<3>().0 {
            corners.swap(0, 1);
        }
    }
    if let Some(corner) = selected
        .corners
        .iter()
        .find(|corner| corner.vertex >= mesh.positions().len())
    {
        return Err(Error::at(
            Location::for_owner(subset.selection.connectivity)
                .field("corners")
                .index(corner.corner),
            GeometryError::IndexOutOfBounds {
                index: corner.vertex,
                len: mesh.positions().len(),
            },
        ));
    }
    let mut batches: Vec<(usize, Vec<SourceCorner>)> = Vec::new();
    for primitive in selected.iter() {
        let size = primitive.len().min(3);
        if batches.last().is_none_or(|(previous, _)| *previous != size) {
            batches.push((size, Vec::new()));
        }
        let (_, corners) = batches.last_mut().expect("primitive batch");
        if size == 3 {
            triangulate(mesh, primitive, corners)?;
        } else {
            corners.extend_from_slice(primitive);
        }
    }
    batches
        .into_iter()
        .map(|(size, corners)| {
            let topology = match size {
                1 => PrimitiveKind::Points,
                2 => PrimitiveKind::Lines,
                _ => PrimitiveKind::Triangles,
            };
            let mut output = ProjectedGeometry::new(topology);
            let positions: Vec<_> = corners
                .iter()
                .map(|c| mesh.positions().get(c.vertex).unwrap())
                .collect();
            output.positions = positions;
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            let mut secondary_uvs = Vec::new();
            let mut third_uvs = Vec::new();
            let mut fourth_uvs = Vec::new();
            let mut colors = Vec::new();
            let mut second_colors = Vec::new();
            let mut tangents = Vec::new();
            for &c in &corners {
                if let Some(ResolvedAttribute {
                    values: AttributeValues::Vector3(values),
                    index,
                    ..
                }) = attribute(mesh, subset, &AttributeSemantic::Normal, c)?
                {
                    normals.push(values.get(index).unwrap());
                }
                if let Some(ResolvedAttribute {
                    values: AttributeValues::Vector2(values),
                    index,
                    ..
                }) = attribute(mesh, subset, &AttributeSemantic::TexCoord(coordinate), c)?
                {
                    uvs.push(values.get(index).unwrap());
                }
                for (coordinate, target) in [
                    (secondary, &mut secondary_uvs),
                    (third, &mut third_uvs),
                    (fourth, &mut fourth_uvs),
                ] {
                    if let Some(coordinate) = coordinate {
                        if let Some(ResolvedAttribute {
                            values: AttributeValues::Vector2(values),
                            index,
                            ..
                        }) =
                            attribute(mesh, subset, &AttributeSemantic::TexCoord(coordinate), c)?
                        {
                            target.push(values.get(index).unwrap());
                        } else {
                            return Err(Error::at(
                                Location::new()
                                    .field("texture_coordinates")
                                    .index(coordinate as usize),
                                GeometryError::MissingTextureCoordinates { slot: coordinate },
                            ));
                        }
                    }
                }
                if let Some(ResolvedAttribute {
                    values: AttributeValues::Vector4(values),
                    index,
                    ..
                }) = attribute(mesh, subset, &AttributeSemantic::Color(0), c)?
                {
                    colors.push(color(c.vertex, values.get(index).unwrap()));
                }
                if let Some(ResolvedAttribute {
                    values: AttributeValues::Vector4(values),
                    index,
                    ..
                }) = attribute(mesh, subset, &AttributeSemantic::Color(1), c)?
                {
                    second_colors.push(values.get(index).unwrap());
                }
                if coordinate == 0 {
                    if let Some(ResolvedAttribute {
                        values: AttributeValues::Vector4(values),
                        index,
                        ..
                    }) = attribute(mesh, subset, &AttributeSemantic::Tangent, c)?
                    {
                        tangents.push(values.get(index).unwrap());
                    }
                }
            }
            if !normals.is_empty() {
                output.normals = normals;
            } else if size == 3 {
                output.normals = output
                    .positions
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .flat_map(|t| {
                        let a = t[0];
                        let normal = (t[1] - a).cross(t[2] - a).normalize_or_zero();
                        [normal; 3]
                    })
                    .collect();
            } else {
                output.normals = vec![Vec3::from_array([0., 0., 1.]); corners.len()];
            }
            if !colors.is_empty() {
                output.colors[0] = colors;
            }
            if !second_colors.is_empty() {
                output.colors[1] = second_colors;
            }
            if secondary.is_some() {
                output.texture_coordinates[1] = secondary_uvs;
            }
            if third.is_some() {
                output.texture_coordinates[2] = third_uvs;
            }
            if fourth.is_some() {
                output.texture_coordinates[3] = fourth_uvs;
            }
            if !uvs.is_empty() {
                output.texture_coordinates[0] = uvs;
            } else if coordinate != 0 {
                return Err(Error::at(
                    Location::new()
                        .field("texture_coordinates")
                        .index(coordinate as usize),
                    GeometryError::MissingTextureCoordinates { slot: coordinate },
                ));
            } else {
                output.texture_coordinates[0] = vec![Vec2::from_array([0.; 2]); corners.len()];
            }
            if !tangents.is_empty() {
                output.tangents = tangents;
            }
            output.sources = corners;
            Ok(output)
        })
        .collect()
}

/// Resolve source contributions for any application-owned vertex mapping.
pub fn skin_weights(
    vertices: impl Iterator<Item = usize>,
    source: &Mesh<'_>,
    joints: &[NodeId],
) -> Result<SkinWeights, Error> {
    let rows = source.influences();
    let mut contributions = Vec::with_capacity(vertices.size_hint().0);
    let mut palette = Vec::new();
    for vertex in vertices {
        let values = match rows.get(vertex) {
            None => Vec::new(),
            Some(VertexInfluences::Joints(values)) => values
                .into_iter()
                .map(|v| (v.joint as usize, v.weight))
                .collect(),
            Some(VertexInfluences::Nodes(values)) => values
                .iter()
                .map(|v| {
                    joints
                        .iter()
                        .position(|id| *id == v.node)
                        .map(|i| (i, v.weight))
                        .ok_or_else(|| {
                            Error::at(
                                Location::new()
                                    .field("influences")
                                    .index(vertex)
                                    .field("node"),
                                GeometryError::UnboundNode { node: v.node },
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        for (index, &(_, weight)) in values.iter().enumerate() {
            if !weight.is_finite() || weight < 0. {
                return Err(Error::at(
                    Location::new()
                        .field("influences")
                        .index(vertex)
                        .index(index)
                        .field("weight"),
                    GeometryError::InvalidWeight { weight },
                ));
            }
        }
        let active: Vec<_> = values.into_iter().filter(|(_, w)| *w > 0.).collect();
        let sum: f32 = active.iter().map(|(_, w)| w).sum();
        if !sum.is_finite() {
            return Err(Error::at(
                Location::new().field("influences").index(vertex),
                GeometryError::NonFiniteWeightSum { sum },
            ));
        }
        if sum <= 0. {
            return Err(Error::at(
                Location::new().field("influences").index(vertex),
                GeometryError::NoPositiveWeights { sum },
            ));
        }
        for &(joint, _) in &active {
            if joint >= joints.len() {
                return Err(Error::at(
                    Location::new()
                        .field("influences")
                        .index(vertex)
                        .field("joint"),
                    GeometryError::IndexOutOfBounds {
                        index: joint,
                        len: joints.len(),
                    },
                ));
            }
            palette.push(joint);
        }
        contributions.push((active, sum));
    }
    palette.sort_unstable();
    palette.dedup();
    let rows = contributions
        .into_iter()
        .map(|(active, sum)| {
            active
                .into_iter()
                .map(|(joint, weight)| {
                    (
                        palette.binary_search(&joint).expect("collected joint"),
                        weight / sum,
                    )
                })
                .collect()
        })
        .collect();
    Ok(SkinWeights { palette, rows })
}
impl Mesh<'_> {
    /// Projects selected geometry while applying instance-specific vertex color interpretation.
    /// The callback runs before complete-record interning by a renderer.
    pub fn project_with_color(
        &self,
        subset: &DrawSubset,
        coordinate: u32,
        additional: [Option<u32>; 3],
        color: impl Fn(usize, Vec4) -> Vec4,
    ) -> Result<Vec<ProjectedGeometry>, Error> {
        project_geometry_with_color(self, subset, coordinate, additional, color)
    }

    /// Projects a draw into ordered primitive batches. UV slots select authored sets;
    /// source mappings allow applications to retain additional attribute streams.
    pub fn project(
        &self,
        subset: &DrawSubset,
        coordinate: u32,
        additional: [Option<u32>; 3],
    ) -> Result<Vec<ProjectedGeometry>, Error> {
        project_geometry_with_color(self, subset, coordinate, additional, |_, color| color)
    }
    /// Selects all primary geometry without allocating a draw identity.
    pub fn whole_geometry(&self) -> Result<DrawSubset, Error> {
        whole_geometry(self)
    }
}

impl ProjectedGeometry {
    /// Projects an additional typed attribute using this batch's complete source mapping.
    /// This preserves application fields without making the renderer copy unused streams.
    /// Tangent requests honor the subset's explicit tangent-pool binding, even when
    /// the batch was projected with a different UV slot and has no tangent stream.
    /// Returns `None` when the attribute is absent; partial correspondence is an error.
    pub fn attribute(
        &self,
        mesh: &Mesh<'_>,
        subset: &DrawSubset,
        semantic: &AttributeSemantic<'_>,
    ) -> Result<Option<AttributeValues<'static>>, Error> {
        let Some(first) = self.sources.first() else {
            return Ok(None);
        };
        let Some(ResolvedAttribute { values: value, .. }) =
            attribute(mesh, subset, semantic, *first)?
        else {
            return Ok(None);
        };
        macro_rules! gather { ($($variant:ident),*) => { match value { $(AttributeValues::$variant(_) => {
            let mut output = Vec::with_capacity(self.sources.len());
            for (source_index, &corner) in self.sources.iter().enumerate() {
                let actual = attribute(mesh, subset, semantic, corner)?;
                let Some(ResolvedAttribute { values: AttributeValues::$variant(values), index, domain }) = actual else {
                    let location = actual.map_or_else(|| Location::new().field("sources").index(source_index), |resolved| Location::new().attribute(semantic.clone(), resolved.domain).index(resolved.index));
                    return Err(Error::at(location, GeometryError::InconsistentAttribute { semantic: semantic.clone().into_owned(), expected: AttributeValueKind::from(value), actual: actual.map(|resolved| AttributeValueKind::from(resolved.values)) }));
                };
                output.push(values.get(index).ok_or_else(|| Error::at(Location::new().attribute(semantic.clone(), domain).index(index), GeometryError::IndexOutOfBounds { index, len: values.len() }))?);
            }
            Ok(Some(AttributeValues::$variant(output.into_iter().collect())))
        },)* } }; }
        gather!(
            TriangleProperties,
            Boolean,
            Byte3,
            Byte4,
            U16,
            Unsigned4,
            Quaternion,
            Scalar,
            Vector2,
            Vector3,
            Vector4,
            Unsigned,
            Signed
        )
    }
}
