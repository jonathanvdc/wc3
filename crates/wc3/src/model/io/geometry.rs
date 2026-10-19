//! Native projection expands logical corner and face attributes at the output boundary.
use super::{Context, Error};
use crate::model::common::Borrowed as _;
use crate::model::common::PackedArray;
use crate::model::io::{materialize, owned};
use crate::model::profiles::Profile;
use crate::model::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, ConnectivityId, DrawId,
    GeometryParts, InfluenceRows, InstanceId, LookId, MaterialId, Mesh, Model, PrimitiveRange,
    SkinId, Topology, ValueGroups,
};
use crate::model::{GeometryInfluences, GeometrySchema};
use glam::{Vec2, Vec3, Vec4};
use std::collections::BTreeMap;
/// Geometry for one material draw after attribute seams and polygons are resolved.
#[derive(Clone, Debug)]
pub struct ProjectedDraw<'a> {
    /// Resolved material after applying the selected appearance.
    pub material: MaterialId,
    /// Original logical draw carrying the projected pass behavior.
    pub source_draw: DrawId,
    /// Geometry with seam vertices expanded and polygon connectivity resolved.
    pub mesh: Mesh<'a>,
    /// Original vertex ordinal for each projected vertex.
    pub source_vertices: PackedArray<'a, u32>,
    /// Joint palette retained by the projected placement.
    pub skin: Option<SkinId>,
}
#[derive(Clone, Copy, Debug)]
struct Corner {
    connectivity: ConnectivityId,
    vertex: u32,
    corner: usize,
    face: usize,
}
#[derive(Clone)]
enum Primitive {
    Point(Corner),
    Line([Corner; 2]),
    Polygon(Vec<Corner>),
}
fn primitives(
    topology: &Topology<'_>,
    connectivity: ConnectivityId,
    out: &mut Vec<Primitive>,
    corner: &mut usize,
    face: &mut usize,
) {
    match topology {
        Topology::TriangleStrip(indices) => {
            for index in 0..indices.len().saturating_sub(2) {
                let mut vertices = [
                    indices.get(index).unwrap(),
                    indices.get(index + 1).unwrap(),
                    indices.get(index + 2).unwrap(),
                ];
                if index % 2 != 0 {
                    vertices.swap(0, 1);
                }
                let points = vertices.map(|vertex| {
                    let point = Corner {
                        connectivity,
                        vertex,
                        corner: *corner,
                        face: *face,
                    };
                    *corner += 1;
                    point
                });
                out.push(Primitive::Polygon(points.to_vec()));
                *face += 1;
            }
        }
        Topology::Mixed(groups) => {
            for group in groups {
                primitives(group, connectivity, out, corner, face);
            }
        }
        Topology::Points(values) => {
            for vertex in values.iter() {
                out.push(Primitive::Point(Corner {
                    connectivity,
                    vertex,
                    corner: *corner,
                    face: *face,
                }));
                *corner += 1;
                *face += 1;
            }
        }
        Topology::Lines(values) => {
            for vertices in values.iter() {
                let points = vertices.map(|vertex| {
                    let point = Corner {
                        connectivity,
                        vertex,
                        corner: *corner,
                        face: *face,
                    };
                    *corner += 1;
                    point
                });
                out.push(Primitive::Line(points));
                *face += 1;
            }
        }
        Topology::Triangles(values) => {
            for vertices in values.iter() {
                let points = vertices.map(|vertex| {
                    let point = Corner {
                        connectivity,
                        vertex,
                        corner: *corner,
                        face: *face,
                    };
                    *corner += 1;
                    point
                });
                out.push(Primitive::Polygon(points.to_vec()));
                *face += 1;
            }
        }
        Topology::Polygons { corners, offsets } => {
            let offsets = offsets.to_vec();
            for range in offsets.windows(2) {
                let points = (range[0] as usize..range[1] as usize)
                    .map(|index| {
                        let point = Corner {
                            connectivity,
                            vertex: corners.get(index).expect("validated index"),
                            corner: *corner,
                            face: *face,
                        };
                        *corner += 1;
                        point
                    })
                    .collect();
                out.push(Primitive::Polygon(points));
                *face += 1;
            }
        }
    }
}
fn index(attribute: &Attribute<'_>, point: Corner) -> usize {
    match attribute.domain {
        AttributeDomain::Vertex => point.vertex as usize,
        AttributeDomain::Corner | AttributeDomain::ConnectivityCorner(_) => point.corner,
        AttributeDomain::Face | AttributeDomain::ConnectivityFace(_) => point.face,
        AttributeDomain::Mesh => 0,
        AttributeDomain::Pool => {
            unreachable!("indexed pool projection requires explicit translation")
        }
    }
}
fn applies(attribute: &Attribute<'_>, primary: ConnectivityId, selected: ConnectivityId) -> bool {
    match attribute.domain {
        AttributeDomain::Face | AttributeDomain::Corner => selected == primary,
        AttributeDomain::ConnectivityFace(id) | AttributeDomain::ConnectivityCorner(id) => {
            id == selected
        }
        _ => true,
    }
}
fn lookup<'a, 'data: 'a, P: Profile>(
    mesh: &'a Mesh<'data, P>,
    semantic: &AttributeSemantic<'_>,
    connectivity: ConnectivityId,
) -> Option<&'a Attribute<'data>> {
    mesh.attributes()
        .iter()
        .filter(|attribute| {
            &attribute.semantic == semantic
                && applies(attribute, mesh.subsets.primary(), connectivity)
        })
        .min_by_key(|attribute| match attribute.domain {
            AttributeDomain::Corner | AttributeDomain::ConnectivityCorner(_) => 0,
            AttributeDomain::Face | AttributeDomain::ConnectivityFace(_) => 1,
            AttributeDomain::Vertex => 2,
            AttributeDomain::Mesh => 3,
            AttributeDomain::Pool => 4,
        })
}
/// Projects each draw, encoding expanded attributes once at the output boundary.
pub fn project<'a, 'data: 'a, P: Profile>(
    model: &'a Model<'data, P>,
    instance: InstanceId,
    look: Option<LookId>,
    _context: &mut Context,
) -> Result<Vec<ProjectedDraw<'a>>, Error> {
    let instance_id = instance;
    let instance = model
        .instance(instance)
        .ok_or_else(|| Error::new("geometry.instance", "placement is absent"))?;
    let source = model
        .mesh(instance.mesh)
        .ok_or_else(|| Error::new("geometry.mesh", "geometry is absent"))?;
    if source.attributes().iter().any(|attribute| {
        attribute.domain == AttributeDomain::Pool
            || attribute.semantic == AttributeSemantic::SurfaceControls
    }) || source
        .geometry
        .surface_triangles()
        .is_some_and(|controls| !controls.is_empty())
        || source
            .geometry
            .tangent_pool()
            .is_some_and(|values| !values.is_empty())
    {
        return Err(Error::new(
            "geometry.domains",
            "surface controls and indexed draw pools require an explicit domain translation",
        ));
    }
    if instance.draws.iter().any(|draw| {
        source.subset(draw.subset).is_some_and(|subset| {
            subset.strip.is_some() || subset.tangents.is_some() || subset.vertex_window.is_some()
        })
    }) {
        return Err(Error::new(
            "geometry.subset.bindings",
            "supplemental selections and pool associations require explicit domain translation",
        ));
    }
    if matches!(source.influences(), InfluenceRows::Nodes(_)) {
        return Err(Error::new(
            "geometry.influences",
            "direct-node rig contributions require an explicit binding translation",
        ));
    }
    if let [draw] = instance.draws.as_slice() {
        if let Ok(draw) = source.resolve_draw(draw) {
            if draw.first == 0
                && draw.count as usize == source.topology().primitive_count()
                && source.attributes().is_empty()
                && matches!(
                    source.topology(),
                    Topology::Triangles(_) | Topology::Lines(_) | Topology::Points(_)
                )
            {
                let length = source.positions().len();
                let rows = source.influences();
                let influences = if rows.is_empty() {
                    ValueGroups::default()
                } else {
                    ValueGroups::from_rows((0..rows.len()).map(|index| {
                        rows.joint_weights(index)
                            .expect("checked joint-slot influence index")
                    }))
                };
                return Ok(vec![ProjectedDraw {
                    material: model.material_for_draw(instance_id, draw.id, look)?,
                    source_draw: draw.id,
                    skin: instance.skin.or_else(|| source.skin()),
                    source_vertices: materialize(length, |index| index as u32),
                    mesh: Mesh {
                        subsets: Default::default(),
                        name: source.name.clone(),
                        controls: None,
                        bounds: source.bounds,
                        geometry: GeometryParts {
                            strip_set: false,
                            surface_triangles: PackedArray::new(),
                            pools: Default::default(),
                            kind: source.geometry.kind(),
                            positions: source.positions().as_borrowed(),
                            normals: source
                                .normals()
                                .map(PackedArray::as_borrowed)
                                .unwrap_or_default(),
                            tangents: source
                                .tangents()
                                .map(PackedArray::as_borrowed)
                                .unwrap_or_default(),
                            texture_coordinates: source
                                .texture_coordinates()
                                .iter()
                                .map(PackedArray::as_borrowed)
                                .collect(),
                            colors: source
                                .colors()
                                .map(PackedArray::as_borrowed)
                                .unwrap_or_default(),
                            influences: GeometryInfluences::Variable(influences),
                            topology: source.topology().clone(),
                            attributes: Vec::new(),
                        }
                        .into_data()?,
                    },
                }]);
            }
        }
    }
    let mut out = Vec::new();
    for draw in &instance.draws {
        let subset = source
            .subset(draw.subset)
            .ok_or_else(|| Error::new("geometry.subset", "draw subset is absent"))?;
        let topology = source
            .connectivity(subset.selection.connectivity)
            .ok_or_else(|| {
                Error::new("geometry.connectivity", "selected connectivity is absent")
            })?;
        let mut all = Vec::new();
        let (first, count) = match subset.selection.range {
            PrimitiveRange::Primitives { first, count } => {
                primitives(
                    &topology,
                    subset.selection.connectivity,
                    &mut all,
                    &mut 0,
                    &mut 0,
                );
                (first as usize, count as usize)
            }
            PrimitiveRange::StripIndices { first, count } => {
                let Topology::TriangleStrip(indices) = topology else {
                    return Err(Error::new(
                        "geometry.selection",
                        "strip selection requires strip connectivity",
                    ));
                };
                let selected = materialize(count as usize, |index| {
                    indices
                        .get(first as usize + index)
                        .expect("validated strip window")
                });
                primitives(
                    &Topology::TriangleStrip(selected),
                    subset.selection.connectivity,
                    &mut all,
                    &mut (first as usize * 3),
                    &mut (first as usize),
                );
                if first % 2 != 0 {
                    for primitive in &mut all {
                        if let Primitive::Polygon(points) = primitive {
                            let corner = points[0].corner;
                            points[0].corner = points[1].corner;
                            points[1].corner = corner;
                        }
                    }
                }
                (0, all.len())
            }
        };
        let selected = all
            .get(first..first + count)
            .ok_or_else(|| Error::new("geometry.draw", "primitive range is absent"))?;
        let material = model.material_for_draw(instance_id, draw.id(), look)?;
        let mut points = Vec::new();
        let mut groups = Vec::new();
        let domain_seams = source.attributes().iter().any(|attribute| {
            applies(
                attribute,
                source.subsets.primary(),
                subset.selection.connectivity,
            ) && matches!(
                attribute.domain,
                AttributeDomain::Face
                    | AttributeDomain::Corner
                    | AttributeDomain::ConnectivityFace(_)
                    | AttributeDomain::ConnectivityCorner(_)
            )
        });
        let mut identities = BTreeMap::new();
        let mut vertex = |point: Corner| {
            let key = if domain_seams {
                (point.vertex, point.corner, point.face)
            } else {
                (point.vertex, 0, 0)
            };
            *identities.entry(key).or_insert_with(|| {
                let id = points.len() as u32;
                points.push(point);
                id
            })
        };
        for primitive in selected {
            match primitive {
                Primitive::Point(point) => {
                    groups.push(Topology::Points(owned(vec![vertex(*point)])))
                }
                Primitive::Line(pair) => {
                    groups.push(Topology::Lines(owned(vec![pair.map(&mut vertex)])))
                }
                Primitive::Polygon(polygon) => {
                    let triangles: Vec<[u32; 3]> = triangulate(source, polygon)?
                        .into_iter()
                        .map(|triangle| triangle.map(&mut vertex))
                        .collect();
                    groups.push(Topology::Triangles(owned(triangles)));
                }
            }
        }
        let length = points.len();
        let positions = {
            materialize(length, |i| {
                source
                    .positions()
                    .get(points[i].vertex as usize)
                    .expect("validated vertex")
            })
        };
        let normals = vector3(source, AttributeSemantic::Normal, &points, source.normals())?;
        let tangents = vector4(
            source,
            AttributeSemantic::Tangent,
            &points,
            source.tangents(),
        )?;
        let colors = vector4(
            source,
            AttributeSemantic::Color(0),
            &points,
            source.colors(),
        )?;
        let max_uv = source
            .attributes()
            .iter()
            .filter_map(|attribute| {
                if let AttributeSemantic::TexCoord(slot) = attribute.semantic {
                    Some(slot as usize + 1)
                } else {
                    None
                }
            })
            .max()
            .unwrap_or(0)
            .max(source.texture_coordinates().len());
        let mut uv_sets = Vec::new();
        for slot in 0..max_uv {
            let fallback = source.texture_coordinates().get(slot);
            uv_sets.push(vector2(
                source,
                AttributeSemantic::TexCoord(slot as u32),
                &points,
                fallback,
            )?);
        }
        let influences = if source.influences().is_empty() {
            ValueGroups::default()
        } else {
            ValueGroups::from_rows((0..length).map(|i| {
                source
                    .influences()
                    .joint_weights(points[i].vertex as usize)
                    .expect("validated influence")
            }))
        };

        let topology = if groups
            .iter()
            .all(|group| matches!(group, Topology::Triangles(_)))
        {
            Topology::Triangles(owned(
                groups
                    .into_iter()
                    .flat_map(|group| match group {
                        Topology::Triangles(values) => values.to_vec(),
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>(),
            ))
        } else if groups
            .iter()
            .all(|group| matches!(group, Topology::Lines(_)))
        {
            Topology::Lines(owned(
                groups
                    .into_iter()
                    .flat_map(|group| match group {
                        Topology::Lines(values) => values.to_vec(),
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>(),
            ))
        } else if groups
            .iter()
            .all(|group| matches!(group, Topology::Points(_)))
        {
            Topology::Points(owned(
                groups
                    .into_iter()
                    .flat_map(|group| match group {
                        Topology::Points(values) => values.to_vec(),
                        _ => unreachable!(),
                    })
                    .collect::<Vec<_>>(),
            ))
        } else {
            Topology::Mixed(groups)
        };
        let source_vertices = { materialize(length, |i| points[i].vertex) };
        let attributes = source
            .attributes()
            .iter()
            .filter(|attribute| {
                applies(
                    attribute,
                    source.subsets.primary(),
                    subset.selection.connectivity,
                ) && !matches!(
                    attribute.semantic,
                    AttributeSemantic::Normal
                        | AttributeSemantic::Tangent
                        | AttributeSemantic::Color(0)
                        | AttributeSemantic::TexCoord(_)
                )
            })
            .map(|attribute| project_attribute(attribute, &points))
            .collect();
        out.push(ProjectedDraw {
            material,
            source_draw: draw.id(),
            skin: instance.skin.or_else(|| source.skin()),
            source_vertices,
            mesh: Mesh {
                subsets: Default::default(),
                controls: None,
                name: source.name.clone(),
                bounds: source.bounds,
                geometry: GeometryParts {
                    strip_set: false,
                    surface_triangles: PackedArray::new(),
                    pools: Default::default(),
                    kind: source.geometry.kind(),
                    positions,
                    normals,
                    tangents,
                    colors,
                    texture_coordinates: uv_sets,
                    influences: GeometryInfluences::Variable(influences),
                    topology,
                    attributes,
                }
                .into_data()?,
            },
        });
    }
    Ok(out)
}

fn vector3<'a, 'data: 'a, P: Profile>(
    mesh: &'a Mesh<'data, P>,
    semantic: AttributeSemantic<'_>,
    points: &[Corner],
    fallback: Option<&'a PackedArray<'data, Vec3>>,
) -> Result<PackedArray<'a, Vec3>, Error> {
    if let Some(attribute) = lookup(
        mesh,
        &semantic,
        points
            .first()
            .map(|point| point.connectivity)
            .unwrap_or_else(|| mesh.subsets.primary()),
    ) {
        let AttributeValues::Vector3(values) = &attribute.values else {
            return Err(Error::new(
                "geometry.attribute",
                "expected a three-component vector",
            ));
        };
        Ok(materialize(points.len(), |i| {
            values
                .get(index(attribute, points[i]))
                .expect("validated attribute")
        }))
    } else if let Some(fallback) = fallback.filter(|values| !values.is_empty()) {
        Ok(materialize(points.len(), |i| {
            fallback
                .get(points[i].vertex as usize)
                .expect("validated attribute")
        }))
    } else {
        Ok(PackedArray::new())
    }
}
fn vector4<'a, 'data: 'a, P: Profile>(
    mesh: &'a Mesh<'data, P>,
    semantic: AttributeSemantic<'_>,
    points: &[Corner],
    fallback: Option<&'a PackedArray<'data, Vec4>>,
) -> Result<PackedArray<'a, Vec4>, Error> {
    if let Some(attribute) = lookup(
        mesh,
        &semantic,
        points
            .first()
            .map(|point| point.connectivity)
            .unwrap_or_else(|| mesh.subsets.primary()),
    ) {
        let AttributeValues::Vector4(values) = &attribute.values else {
            return Err(Error::new(
                "geometry.attribute",
                "expected a four-component vector",
            ));
        };
        Ok(materialize(points.len(), |i| {
            values
                .get(index(attribute, points[i]))
                .expect("validated attribute")
        }))
    } else if let Some(fallback) = fallback.filter(|values| !values.is_empty()) {
        Ok(materialize(points.len(), |i| {
            fallback
                .get(points[i].vertex as usize)
                .expect("validated attribute")
        }))
    } else {
        Ok(PackedArray::new())
    }
}
fn vector2<'a, 'data: 'a, P: Profile>(
    mesh: &'a Mesh<'data, P>,
    semantic: AttributeSemantic<'_>,
    points: &[Corner],
    fallback: Option<&'a PackedArray<'data, Vec2>>,
) -> Result<PackedArray<'a, Vec2>, Error> {
    if let Some(attribute) = lookup(
        mesh,
        &semantic,
        points
            .first()
            .map(|point| point.connectivity)
            .unwrap_or_else(|| mesh.subsets.primary()),
    ) {
        let AttributeValues::Vector2(values) = &attribute.values else {
            return Err(Error::new(
                "geometry.attribute",
                "expected a two-component vector",
            ));
        };
        Ok(materialize(points.len(), |i| {
            values
                .get(index(attribute, points[i]))
                .expect("validated attribute")
        }))
    } else if let Some(fallback) = fallback.filter(|values| !values.is_empty()) {
        Ok(materialize(points.len(), |i| {
            fallback
                .get(points[i].vertex as usize)
                .expect("validated attribute")
        }))
    } else {
        Ok(PackedArray::new())
    }
}
// Ear clipping preserves authored winding and supports concave simple polygons.
fn triangulate<P: Profile>(
    mesh: &Mesh<'_, P>,
    polygon: &[Corner],
) -> Result<Vec<[Corner; 3]>, Error> {
    if polygon.len() == 3 {
        return Ok(vec![[polygon[0], polygon[1], polygon[2]]]);
    }
    let positions: Vec<_> = polygon
        .iter()
        .map(|point| {
            mesh.positions()
                .get(point.vertex as usize)
                .expect("validated vertex")
                .to_array()
                .map(f64::from)
        })
        .collect();
    let mut normal = [0.0f64; 3];
    for i in 0..positions.len() {
        let a = positions[i];
        let b = positions[(i + 1) % positions.len()];
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
    }
    let axis = (0..3)
        .max_by(|a, b| normal[*a].abs().total_cmp(&normal[*b].abs()))
        .unwrap();
    let xy: Vec<_> = positions
        .iter()
        .map(|p| match axis {
            0 => [p[1], p[2]],
            1 => [p[2], p[0]],
            _ => [p[0], p[1]],
        })
        .collect();
    let cross = |a: usize, b: usize, c: usize| {
        (xy[b][0] - xy[a][0]) * (xy[c][1] - xy[a][1])
            - (xy[b][1] - xy[a][1]) * (xy[c][0] - xy[a][0])
    };
    for a in 0..xy.len() {
        let b = (a + 1) % xy.len();
        for c in a + 1..xy.len() {
            let d = (c + 1) % xy.len();
            if a == c || a == d || b == c || b == d {
                continue;
            }
            let ab_c = cross(a, b, c);
            let ab_d = cross(a, b, d);
            let cd_a = cross(c, d, a);
            let cd_b = cross(c, d, b);
            let on = |p: usize, u: usize, v: usize| {
                xy[p][0] >= xy[u][0].min(xy[v][0])
                    && xy[p][0] <= xy[u][0].max(xy[v][0])
                    && xy[p][1] >= xy[u][1].min(xy[v][1])
                    && xy[p][1] <= xy[u][1].max(xy[v][1])
            };
            if (ab_c * ab_d < 0.0 && cd_a * cd_b < 0.0)
                || (ab_c == 0.0 && on(c, a, b))
                || (ab_d == 0.0 && on(d, a, b))
                || (cd_a == 0.0 && on(a, c, d))
                || (cd_b == 0.0 && on(b, c, d))
            {
                return Err(Error::new(
                    "geometry.polygon",
                    "polygon boundary intersects itself",
                ));
            }
        }
    }
    let area: f64 = (0..xy.len())
        .map(|i| {
            let j = (i + 1) % xy.len();
            xy[i][0] * xy[j][1] - xy[j][0] * xy[i][1]
        })
        .sum();
    if area == 0.0 {
        return Err(Error::new(
            "geometry.polygon",
            "polygon has zero projected area",
        ));
    }
    let winding = area.signum();
    let mut remaining: Vec<_> = (0..polygon.len()).collect();
    let mut output = Vec::new();
    while remaining.len() > 3 {
        let ear = (0..remaining.len())
            .find(|&i| {
                let a = remaining[(i + remaining.len() - 1) % remaining.len()];
                let b = remaining[i];
                let c = remaining[(i + 1) % remaining.len()];
                if cross(a, b, c) * winding <= 0.0 {
                    return false;
                }
                !remaining.iter().any(|&p| {
                    p != a
                        && p != b
                        && p != c
                        && cross(a, b, p) * winding >= 0.0
                        && cross(b, c, p) * winding >= 0.0
                        && cross(c, a, p) * winding >= 0.0
                })
            })
            .ok_or_else(|| {
                Error::new("geometry.polygon", "polygon is degenerate or is not simple")
            })?;
        let a = remaining[(ear + remaining.len() - 1) % remaining.len()];
        let b = remaining[ear];
        let c = remaining[(ear + 1) % remaining.len()];
        output.push([polygon[a], polygon[b], polygon[c]]);
        remaining.remove(ear);
    }
    output.push([
        polygon[remaining[0]],
        polygon[remaining[1]],
        polygon[remaining[2]],
    ]);
    Ok(output)
}

fn project_attribute<'a, 'data: 'a>(
    attribute: &'a Attribute<'data>,
    points: &[Corner],
) -> Attribute<'a> {
    macro_rules! values {
        ($values:expr, $variant:ident) => {{
            let source = $values;
            AttributeValues::$variant(materialize(points.len(), |i| {
                source
                    .get(index(attribute, points[i]))
                    .expect("validated attribute")
            }))
        }};
    }
    let values = match &attribute.values {
        AttributeValues::Boolean(source) => values!(source, Boolean),
        AttributeValues::Byte3(source) => values!(source, Byte3),
        AttributeValues::Byte4(source) => values!(source, Byte4),
        AttributeValues::U16(source) => values!(source, U16),
        AttributeValues::Unsigned4(source) => values!(source, Unsigned4),
        AttributeValues::Quaternion(source) => values!(source, Quaternion),
        AttributeValues::Scalar(source) => values!(source, Scalar),
        AttributeValues::Vector2(source) => values!(source, Vector2),
        AttributeValues::Vector3(source) => values!(source, Vector3),
        AttributeValues::Vector4(source) => values!(source, Vector4),
        AttributeValues::Unsigned(source) => values!(source, Unsigned),
        AttributeValues::Signed(source) => values!(source, Signed),
        AttributeValues::SurfaceControls(source) => values!(source, SurfaceControls),
    };
    Attribute {
        semantic: attribute.semantic.clone(),
        domain: AttributeDomain::Vertex,
        values,
    }
}
