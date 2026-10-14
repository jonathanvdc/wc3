//! Native projection expands logical corner and face attributes at the output boundary.
use super::{Context, Error};
use crate::model::semantic::profiles::Profile;
use crate::model::semantic::{
    Attribute, AttributeDomain, AttributeSemantic, AttributeValues, GeometryData, InfluenceRows,
    InstanceId, LookId, MaterialId, Mesh, Model, SkinId, Topology, UniversalGeometry, Values,
};
use std::collections::BTreeMap;
use std::sync::Arc;
/// Geometry for one material draw after attribute seams and polygons are resolved.
#[derive(Clone, Debug)]
pub struct ProjectedDraw<'a> {
    /// Resolved material after applying the selected appearance.
    pub material: MaterialId,
    /// Original logical draw carrying the projected pass behavior.
    pub source_draw: u32,
    /// Geometry with seam vertices expanded and polygon connectivity resolved.
    pub mesh: Mesh<'a>,
    /// Original vertex ordinal for each projected vertex.
    pub source_vertices: Values<'a, u32>,
    /// Joint palette retained by the projected placement.
    pub skin: Option<SkinId>,
}
#[derive(Clone, Copy, Debug)]
struct Corner {
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
    out: &mut Vec<Primitive>,
    corner: &mut usize,
    face: &mut usize,
) {
    match topology {
        Topology::Mixed(groups) => {
            for group in groups {
                primitives(group, out, corner, face);
            }
        }
        Topology::Points(values) => {
            for vertex in values.iter() {
                out.push(Primitive::Point(Corner {
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
        AttributeDomain::Corner => point.corner,
        AttributeDomain::Face => point.face,
        AttributeDomain::Mesh => 0,
    }
}
fn lookup<'a, 'data: 'a, P: Profile>(
    mesh: &'a Mesh<'data, P>,
    semantic: &AttributeSemantic<'_>,
) -> Option<&'a Attribute<'data>> {
    mesh.attributes()
        .iter()
        .find(|attribute| &attribute.semantic == semantic)
}
/// Projects each draw while borrowing its logical values until native packing is required.
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
    if matches!(source.influences(), InfluenceRows::Nodes(_)) {
        return Err(Error::new(
            "geometry.influences",
            "direct-node rig contributions require an explicit binding translation",
        ));
    }
    if let [draw] = instance.draws.as_slice() {
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
                Values::default()
            } else {
                Values::view(rows.len(), move |index| {
                    rows.joint_weights(index)
                        .expect("checked joint-slot influence index")
                })
            };
            return Ok(vec![ProjectedDraw {
                material: model.material_for_draw(instance_id, 0, look)?,
                source_draw: 0,
                skin: instance.skin.or_else(|| source.skin()),
                source_vertices: Values::view(length, |index| index as u32),
                mesh: Mesh {
                    name: source.name.clone(),
                    controls: None,
                    bounds: source.bounds,
                    geometry: UniversalGeometry::Flexible(GeometryData {
                        positions: source.positions().clone(),
                        normals: source.normals().cloned().unwrap_or_default(),
                        tangents: source.tangents().cloned().unwrap_or_default(),
                        texture_coordinates: source.texture_coordinates().to_vec(),
                        colors: source.colors().cloned().unwrap_or_default(),
                        influences,
                        topology: source.topology().clone(),
                        attributes: Vec::new(),
                    }),
                },
            }]);
        }
    }
    let mut all = Vec::new();
    primitives(source.topology(), &mut all, &mut 0, &mut 0);
    let mut out = Vec::new();
    for (draw_index, draw) in instance.draws.iter().enumerate() {
        let end = draw.first as usize + draw.count as usize;
        let selected = all
            .get(draw.first as usize..end)
            .ok_or_else(|| Error::new("geometry.draw", "primitive range is absent"))?;
        let base_material = model.material_for_draw(instance_id, draw_index as u32, look)?;
        let authored_override = look
            .or(model.default_look())
            .and_then(|id| model.look(id))
            .is_some_and(|look| {
                look.bindings.iter().any(|binding| {
                    binding.instance == instance_id && binding.draw == draw_index as u32
                })
            });
        let material_attribute = lookup(source, &AttributeSemantic::Material);
        let mut segments: Vec<(MaterialId, Vec<&Primitive>)> = Vec::new();
        for primitive in selected {
            let point = match primitive {
                Primitive::Point(point) => *point,
                Primitive::Line(pair) => pair[0],
                Primitive::Polygon(points) => points[0],
            };
            let material = if authored_override {
                base_material
            } else if let Some(attribute) = material_attribute {
                let AttributeValues::Materials(values) = &attribute.values else {
                    return Err(Error::new(
                        "geometry.material",
                        "material attribute must contain material IDs",
                    ));
                };
                if !matches!(
                    attribute.domain,
                    AttributeDomain::Face | AttributeDomain::Mesh
                ) {
                    return Err(Error::new(
                        "geometry.material",
                        "material assignments must be per face or per mesh",
                    ));
                }
                values
                    .get(index(attribute, point))
                    .expect("validated material attribute")
            } else {
                base_material
            };
            if let Some((_, primitives)) = segments.last_mut().filter(|(id, _)| *id == material) {
                primitives.push(primitive);
            } else {
                segments.push((material, vec![primitive]));
            }
        }
        for (material, selected) in segments {
            let mut points = Vec::new();
            let mut groups = Vec::new();
            let domain_seams = source.attributes().iter().any(|attribute| {
                matches!(
                    attribute.domain,
                    AttributeDomain::Face | AttributeDomain::Corner
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
                        groups.push(Topology::Points(Values::owned(vec![vertex(*point)])))
                    }
                    Primitive::Line(pair) => {
                        groups.push(Topology::Lines(Values::owned(vec![pair.map(&mut vertex)])))
                    }
                    Primitive::Polygon(polygon) => {
                        let triangles: Vec<[u32; 3]> = triangulate(source, polygon)?
                            .into_iter()
                            .map(|triangle| triangle.map(&mut vertex))
                            .collect();
                        groups.push(Topology::Triangles(Values::owned(triangles)));
                    }
                }
            }
            let points: Arc<[Corner]> = points.into();
            let length = points.len();
            let positions = {
                let points = points.clone();
                Values::view(length, move |i| {
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
                Values::default()
            } else {
                let points = points.clone();
                Values::view(length, move |i| {
                    source
                        .influences()
                        .joint_weights(points[i].vertex as usize)
                        .expect("validated influence")
                })
            };

            let topology = if groups
                .iter()
                .all(|group| matches!(group, Topology::Triangles(_)))
            {
                Topology::Triangles(Values::owned(
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
                Topology::Lines(Values::owned(
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
                Topology::Points(Values::owned(
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
            let source_vertices = {
                let points = points.clone();
                Values::view(length, move |i| points[i].vertex)
            };
            let attributes = source
                .attributes()
                .iter()
                .filter(|attribute| {
                    !matches!(
                        attribute.semantic,
                        AttributeSemantic::Normal
                            | AttributeSemantic::Tangent
                            | AttributeSemantic::Color(0)
                            | AttributeSemantic::TexCoord(_)
                            | AttributeSemantic::Material
                    )
                })
                .map(|attribute| project_attribute(attribute, &points))
                .collect();
            out.push(ProjectedDraw {
                material,
                source_draw: draw_index as u32,
                skin: instance.skin.or_else(|| source.skin()),
                source_vertices,
                mesh: Mesh {
                    controls: None,
                    name: source.name.clone(),
                    bounds: source.bounds,
                    geometry: UniversalGeometry::Flexible(GeometryData {
                        positions,
                        normals,
                        tangents,
                        colors,
                        texture_coordinates: uv_sets,
                        influences,
                        topology,
                        attributes,
                    }),
                },
            });
        }
    }
    Ok(out)
}

fn vector3<'a, 'data: 'a, P: Profile>(
    mesh: &'a Mesh<'data, P>,
    semantic: AttributeSemantic<'_>,
    points: &Arc<[Corner]>,
    fallback: Option<&'a Values<'data, [f32; 3]>>,
) -> Result<Values<'a, [f32; 3]>, Error> {
    if let Some(attribute) = lookup(mesh, &semantic) {
        let AttributeValues::Vector3(values) = &attribute.values else {
            return Err(Error::new(
                "geometry.attribute",
                "expected a three-component vector",
            ));
        };
        let points = points.clone();
        Ok(Values::view(points.len(), move |i| {
            values
                .get(index(attribute, points[i]))
                .expect("validated attribute")
        }))
    } else if let Some(fallback) = fallback.filter(|values| !values.is_empty()) {
        let points = points.clone();
        Ok(Values::view(points.len(), move |i| {
            fallback
                .get(points[i].vertex as usize)
                .expect("validated attribute")
        }))
    } else {
        Ok(Values::default())
    }
}
fn vector4<'a, 'data: 'a, P: Profile>(
    mesh: &'a Mesh<'data, P>,
    semantic: AttributeSemantic<'_>,
    points: &Arc<[Corner]>,
    fallback: Option<&'a Values<'data, [f32; 4]>>,
) -> Result<Values<'a, [f32; 4]>, Error> {
    if let Some(attribute) = lookup(mesh, &semantic) {
        let AttributeValues::Vector4(values) = &attribute.values else {
            return Err(Error::new(
                "geometry.attribute",
                "expected a four-component vector",
            ));
        };
        let points = points.clone();
        Ok(Values::view(points.len(), move |i| {
            values
                .get(index(attribute, points[i]))
                .expect("validated attribute")
        }))
    } else if let Some(fallback) = fallback.filter(|values| !values.is_empty()) {
        let points = points.clone();
        Ok(Values::view(points.len(), move |i| {
            fallback
                .get(points[i].vertex as usize)
                .expect("validated attribute")
        }))
    } else {
        Ok(Values::default())
    }
}
fn vector2<'a, 'data: 'a, P: Profile>(
    mesh: &'a Mesh<'data, P>,
    semantic: AttributeSemantic<'_>,
    points: &Arc<[Corner]>,
    fallback: Option<&'a Values<'data, [f32; 2]>>,
) -> Result<Values<'a, [f32; 2]>, Error> {
    if let Some(attribute) = lookup(mesh, &semantic) {
        let AttributeValues::Vector2(values) = &attribute.values else {
            return Err(Error::new(
                "geometry.attribute",
                "expected a two-component vector",
            ));
        };
        let points = points.clone();
        Ok(Values::view(points.len(), move |i| {
            values
                .get(index(attribute, points[i]))
                .expect("validated attribute")
        }))
    } else if let Some(fallback) = fallback.filter(|values| !values.is_empty()) {
        let points = points.clone();
        Ok(Values::view(points.len(), move |i| {
            fallback
                .get(points[i].vertex as usize)
                .expect("validated attribute")
        }))
    } else {
        Ok(Values::default())
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
    points: &Arc<[Corner]>,
) -> Attribute<'a> {
    macro_rules! values {
        ($values:expr, $variant:ident) => {{
            let source = $values;
            let points = points.clone();
            AttributeValues::$variant(Values::view(points.len(), move |i| {
                source
                    .get(index(attribute, points[i]))
                    .expect("validated attribute")
            }))
        }};
    }
    let values = match &attribute.values {
        AttributeValues::Boolean(source) => values!(source, Boolean),
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
        AttributeValues::Materials(source) => values!(source, Materials),
    };
    Attribute {
        semantic: attribute.semantic.clone(),
        domain: AttributeDomain::Vertex,
        values,
    }
}
