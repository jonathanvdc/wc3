//! Geometry is rebuilt from scene values, with fresh collider references.
use super::super::{write::append, Graph};
use crate::model::conversion::{Context, Error, IssueKind};
use crate::model::m3::{records, Document, Envelope, Ref};
use crate::model::semantic::{
    profiles::m3::{values::PhysicsShapeKind, Shape, ShapeGeometry},
    M3Geometry, MeshId, Topology,
};
use zerovec::ZeroVec;

struct Geometry {
    points: Vec<records::Vector3>,
    triangles: Vec<[u32; 3]>,
    planes: Vec<records::Vector4>,
    controls: Vec<f32>,
}
fn mesh(graph: &Graph<'_>, id: MeshId, planes: &ZeroVec<'_, [f32; 4]>) -> Result<Geometry, Error> {
    let mesh = graph
        .mesh(id)
        .ok_or_else(|| Error::new("m3.physics.geometry", "missing collision mesh"))?;
    let M3Geometry::Collision(collision) = &mesh.geometry else {
        return Err(Error::new(
            "m3.physics.geometry",
            "collider requires collision geometry",
        ));
    };
    let triangles = match mesh.topology() {
        Topology::Triangles(rows) => rows.iter().collect(),
        Topology::Points(_) => Vec::new(),
        Topology::Polygons { .. } => Vec::new(),
        _ => {
            return Err(Error::new(
                "m3.physics.geometry",
                "unsupported collider connectivity",
            ));
        }
    };
    Ok(Geometry {
        points: mesh
            .positions()
            .iter()
            .map(|value| records::Vector3 { value })
            .collect(),
        triangles,
        planes: planes
            .iter()
            .map(|value| records::Vector4 { value })
            .collect(),
        controls: collision
            .triangle_controls()
            .map_or(Vec::new(), |values| values.iter().collect()),
    })
}
fn geometry(
    graph: &Graph<'_>,
    shape: &Shape<'_>,
) -> Result<(PhysicsShapeKind, [f32; 3], Option<Geometry>), Error> {
    let (kind, dimensions, primary) = match &shape.geometry {
        ShapeGeometry::Box { half_extents } => (PhysicsShapeKind::Box, *half_extents, None),
        ShapeGeometry::Sphere {
            radius,
            axis_controls,
        } => (
            PhysicsShapeKind::Sphere,
            [*radius, axis_controls[0], axis_controls[1]],
            None,
        ),
        ShapeGeometry::Capsule {
            radius,
            length,
            axis_control,
        } => (
            PhysicsShapeKind::Capsule,
            [*radius, *length, *axis_control],
            None,
        ),
        ShapeGeometry::Cylinder {
            radius,
            length,
            axis_control,
        } => (
            PhysicsShapeKind::Cylinder,
            [*radius, *length, *axis_control],
            None,
        ),
        ShapeGeometry::ConvexHull {
            mesh: id,
            planes,
            controls,
        } => (
            PhysicsShapeKind::ConvexHull,
            *controls,
            Some(mesh(graph, *id, planes)?),
        ),
        ShapeGeometry::TriangleMesh {
            mesh: id,
            planes,
            controls,
        } => (
            PhysicsShapeKind::TriangleMesh,
            *controls,
            Some(mesh(graph, *id, planes)?),
        ),
        ShapeGeometry::Unknown {
            kind,
            dimensions,
            mesh: id,
            planes,
        } => (
            *kind,
            *dimensions,
            id.map(|id| mesh(graph, id, planes)).transpose()?,
        ),
    };
    let auxiliary = shape
        .auxiliary_geometry
        .as_ref()
        .map(|aux| mesh(graph, aux.mesh, &aux.planes))
        .transpose()?;
    if primary.is_some() && auxiliary.is_some() {
        return Err(Error::new(
            "m3.physics.geometry",
            "one collider cannot declare two independent geometry inputs",
        ));
    }
    Ok((kind, dimensions, primary.or(auxiliary)))
}
pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Document<'_, E>,
    shapes: &[Shape<'_>],
    context: &mut Context,
) -> Result<Ref<records::targets::PhysicsShape>, Error> {
    let inputs = shapes
        .iter()
        .map(|shape| geometry(graph, shape))
        .collect::<Result<Vec<_>, _>>()?;
    let cooked = shapes.iter().any(|shape| shape.margin.is_some())
        || inputs.iter().any(|(_, _, geometry)| {
            geometry.as_ref().is_some_and(|g| {
                !g.controls.is_empty()
                    || g.triangles
                        .iter()
                        .flatten()
                        .any(|index| *index > u16::MAX.into())
            })
        });
    if shapes.iter().any(|shape| shape.margin.is_some()) {
        let mut rows = Vec::with_capacity(shapes.len());
        for (shape, (kind, dimensions, geometry)) in shapes.iter().zip(inputs) {
            if geometry
                .as_ref()
                .is_some_and(|geometry| !geometry.triangles.is_empty())
            {
                context.unsupported("m3.physics.shape.connectivity", "contact margins require the cooked mesh schema whose seven connectivity lanes remain unresolved")?;
                continue;
            }
            let mut row = records::PhysicsShape::<records::V3> {
                transform: shape.transform,
                shape_type: records::PhysicsShapeKind::from_raw(kind.raw()),
                shape_dimensions: dimensions,
                mesh_collision_margin: shape.margin.unwrap_or_default(),
                ..Default::default()
            };
            if let Some(geometry) = geometry {
                row.hull_vertices = append(document, &geometry.points)?.cast();
                row.hull_planes = append(document, &geometry.planes)?.cast();
                row.hull_vertex_count = geometry.points.len() as u32;
                row.hull_face_count = geometry.planes.len() as u32;
            }
            rows.push(row);
        }
        return append(document, &rows).map(Ref::cast);
    }
    if cooked {
        let mut rows = Vec::with_capacity(shapes.len());
        for (shape, (kind, dimensions, geometry)) in shapes.iter().zip(inputs) {
            let mut row = records::PhysicsShape::<records::V2> {
                transform: shape.transform,
                shape_type: records::PhysicsShapeKind::from_raw(kind.raw()),
                shape_dimensions: dimensions,
                ..Default::default()
            };
            if let Some(geometry) = geometry {
                if kind == PhysicsShapeKind::ConvexHull {
                    row.hull_vertices = append(document, &geometry.points)?.cast();
                    row.hull_planes = append(document, &geometry.planes)?.cast();
                    row.hull_vertex_count = u32::try_from(geometry.points.len()).map_err(|_| {
                        Error::new("m3.physics.geometry", "vertex capacity exceeded")
                    })?;
                    row.hull_face_count = geometry.planes.len() as u32;
                } else {
                    row.positions = append(document, &geometry.points)?.cast();
                    row.mesh_vertex_count = geometry.points.len() as u32;
                    let triangles = geometry
                        .triangles
                        .iter()
                        .enumerate()
                        .map(|(index, indices)| records::PhysicsMeshTriangle {
                            vertex_index0: indices[0],
                            vertex_index1: indices[1],
                            vertex_index2: indices[2],
                            unresolved_control: geometry
                                .controls
                                .get(index)
                                .copied()
                                .unwrap_or_default(),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>();
                    row.triangles = append(document, &triangles)?.cast();
                    row.mesh_face_index32_count = triangles.len() as u32;
                    row.hull_planes = append(document, &geometry.planes)?.cast();
                }
            }
            rows.push(row);
        }
        context.issue(
            "m3.physics.cooking",
            IssueKind::Normalized,
            "derived cooking storage is omitted from the relocated geometric definition",
        );
        append(document, &rows).map(Ref::cast)
    } else {
        let mut rows = Vec::with_capacity(shapes.len());
        for (shape, (kind, dimensions, geometry)) in shapes.iter().zip(inputs) {
            let mut row = records::PhysicsShape::<records::V1> {
                transform: shape.transform,
                shape_type: records::PhysicsShapeKind::from_raw(kind.raw()),
                shape_dimensions: dimensions,
                ..Default::default()
            };
            if let Some(geometry) = geometry {
                row.source_points = append(document, &geometry.points)?.cast();
                row.planes = append(document, &geometry.planes)?.cast();
                let triangles = geometry
                    .triangles
                    .iter()
                    .flatten()
                    .map(|index| {
                        Ok(records::U16 {
                            value: u16::try_from(*index).map_err(|_| {
                                Error::new("m3.physics.geometry", "triangle capacity exceeded")
                            })?,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                row.source_triangles = append(document, &triangles)?.cast();
            }
            rows.push(row);
        }
        append(document, &rows).map(Ref::cast)
    }
}
