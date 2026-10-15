//! Collider geometry uses checked scene meshes; cooking tables do not become graph addresses.
use super::super::{
    materials::{read_error, revision},
    slots::{Slot, Unresolved as _},
    GraphBuilder,
};
use crate::model::conversion::materialize;
use crate::model::conversion::{Context, Error};
use crate::model::m3::{records, Document, Envelope, Ref};
use crate::model::semantic::{
    profiles::m3::{values::PhysicsShapeKind, AuxiliaryGeometry, Shape, ShapeGeometry},
    M3CollisionGeometry, M3Geometry, Mesh,
};
use std::array::from_fn;
use zerovec::ZeroVec;

fn points<'view, E: Envelope + Send + Sync>(
    document: &'view Document<'_, E>,
    reference: Ref<records::targets::Vector3>,
) -> Result<ZeroVec<'view, [f32; 3]>, Error> {
    let rows = document
        .resolve::<records::Vector3>(reference.cast())
        .map_err(read_error)?;
    for row in rows.iter() {
        row.map_err(read_error)?;
    }
    Ok(materialize(rows.len(), move |index| {
        rows.get(index)
            .expect("checked point index")
            .expect("validated point")
            .value
    }))
}
fn planes<'view, E: Envelope + Send + Sync>(
    document: &'view Document<'_, E>,
    reference: Ref<records::targets::Vector4>,
) -> Result<ZeroVec<'view, [f32; 4]>, Error> {
    let rows = document
        .resolve::<records::Vector4>(reference.cast())
        .map_err(read_error)?;
    for row in rows.iter() {
        row.map_err(read_error)?;
    }
    Ok(materialize(rows.len(), move |index| {
        rows.get(index)
            .expect("checked plane index")
            .expect("validated plane")
            .value
    }))
}
struct Input {
    transform: [[f32; 4]; 4],
    kind: PhysicsShapeKind,
    dimensions: [f32; 3],
    margin: Option<f32>,
    source: Ref<records::targets::Vector3>,
    triangles: Ref<records::targets::U16>,
    planes: Ref<records::targets::Vector4>,
    hull: Ref<records::targets::Vector3>,
    hull_planes: Ref<records::targets::Vector4>,
    positions: Ref<records::targets::Vector3>,
    mesh_triangles: Ref<records::targets::PhysicsMeshTriangle>,
    modern_positions: Ref<records::targets::Vector4>,
    modern_faces: bool,
}
fn geometry<'view, E: Envelope + Send + Sync>(
    document: &'view Document<'_, E>,
    input: &Input,
    builder: &mut GraphBuilder<'view>,
    context: &mut Context,
) -> Result<Option<AuxiliaryGeometry<'view>>, Error> {
    let positions = if !input.source.is_empty() {
        points(document, input.source)?
    } else if !input.hull.is_empty() {
        points(document, input.hull)?
    } else if !input.positions.is_empty() {
        points(document, input.positions)?
    } else {
        let rows = document
            .resolve::<records::Vector4>(input.modern_positions.cast())
            .map_err(read_error)?;
        for row in rows.iter() {
            let row = row.map_err(read_error)?;
            if row.value[3] != 0.0 {
                context.unsupported(
                    "m3.physics.shape.positions.w",
                    "a cooked position has a nonzero fourth component whose meaning is unresolved",
                )?;
            }
        }
        materialize(rows.len(), move |index| {
            let p = rows
                .get(index)
                .expect("checked point index")
                .expect("validated point")
                .value;
            [p[0], p[1], p[2]]
        })
    };
    if positions.is_empty()
        && input.triangles.is_empty()
        && input.mesh_triangles.is_empty()
        && !input.modern_faces
        && input.planes.is_empty()
        && input.hull_planes.is_empty()
    {
        return Ok(None);
    }
    if input.modern_faces {
        context.unsupported(
            "m3.physics.shape.connectivity",
            "the seven cooked mesh lanes have no verified vertex-index interpretation",
        )?;
    }
    let mesh = if !input.triangles.is_empty() {
        let rows = document
            .resolve::<records::U16>(input.triangles.cast())
            .map_err(read_error)?;
        for row in rows.iter() {
            row.map_err(read_error)?;
        }
        if rows.len() % 3 != 0 {
            return Err(Error::new(
                "m3.physics.shape.triangles",
                "source connectivity must contain complete triangles",
            ));
        }
        let triangles = materialize(rows.len() / 3, move |index| {
            from_fn(|lane| {
                u32::from(
                    rows.get(index * 3 + lane)
                        .expect("checked corner")
                        .expect("validated corner")
                        .value,
                )
            })
        });
        M3CollisionGeometry::new(positions, triangles)?
    } else if !input.mesh_triangles.is_empty() {
        let rows = document
            .resolve::<records::PhysicsMeshTriangle>(input.mesh_triangles.cast())
            .map_err(read_error)?;
        for row in rows.iter() {
            row.map_err(read_error)?;
        }
        let triangles = materialize(rows.len(), move |index| {
            let row = rows
                .get(index)
                .expect("checked triangle")
                .expect("validated triangle");
            [row.vertex_index0, row.vertex_index1, row.vertex_index2]
        });
        let controls = document
            .resolve::<records::PhysicsMeshTriangle>(input.mesh_triangles.cast())
            .map_err(read_error)?;
        let controls = materialize(controls.len(), move |index| {
            controls
                .get(index)
                .expect("checked triangle control")
                .expect("validated triangle control")
                .unresolved_control
        });
        M3CollisionGeometry::new(positions, triangles)?.with_triangle_controls(controls)?
    } else {
        M3CollisionGeometry::points(positions)?
    };
    let mesh = builder.add_mesh(Mesh::new(M3Geometry::Collision(mesh)))?;
    let plane_ref = if input.planes.is_empty() {
        input.hull_planes
    } else {
        input.planes
    };
    Ok(Some(AuxiliaryGeometry {
        mesh,
        planes: planes(document, plane_ref)?,
    }))
}

pub(super) fn read<'view, E: Envelope + Send + Sync>(
    document: &'view Document<'_, E>,
    reference: Ref<records::targets::PhysicsShape>,
    builder: &mut GraphBuilder<'view>,
    context: &mut Context,
) -> Result<Vec<Shape<'view>>, Error> {
    if reference.is_empty() {
        return Ok(Vec::new());
    }
    let mut output = Vec::new();
    macro_rules! read {
        ($version:ty) => {
            for row in document
                .resolve::<records::PhysicsShape<$version>>(reference.cast())
                .map_err(read_error)?
                .iter()
            {
                let r = row.map_err(read_error)?;
                for (field, populated) in [
                    ("legacy_reserved_0", r.legacy_reserved_0.populated()),
                    ("legacy_reserved_1", r.legacy_reserved_1.populated()),
                    ("reserved_0", r.reserved_0.populated()),
                    ("unused", r.unused.populated()),
                    ("cooked_controls", r.cooked_controls.populated()),
                    ("reserved_1", r.reserved_1.populated()),
                    ("reserved_2", r.reserved_2.populated()),
                    ("mesh_unknown1", r.mesh_unknown1.populated()),
                    ("mesh_reserved", r.mesh_reserved.populated()),
                ] {
                    if populated {
                        context.unsupported(
                            format!("m3.physics.shape.{field}"),
                            "this populated source field has no established interpretation",
                        )?;
                    }
                }
                let modern16 =
                    Slot::<Ref<records::targets::MeshTriangle16>>::slot(r.mesh_face_indices16)
                        .unwrap_or_default();
                let modern32 =
                    Slot::<Ref<records::targets::MeshTriangle32>>::slot(r.mesh_face_indices32)
                        .unwrap_or_default();
                let input = Input {
                    transform: r.transform,
                    kind: PhysicsShapeKind::from_raw(r.shape_type.raw()),
                    dimensions: r.shape_dimensions,
                    margin: Slot::<f32>::slot(r.mesh_collision_margin),
                    source: Slot::<Ref<records::targets::Vector3>>::slot(r.source_points)
                        .unwrap_or_default(),
                    triangles: Slot::<Ref<records::targets::U16>>::slot(r.source_triangles)
                        .unwrap_or_default(),
                    planes: Slot::<Ref<records::targets::Vector4>>::slot(r.planes)
                        .unwrap_or_default(),
                    hull: Slot::<Ref<records::targets::Vector3>>::slot(r.hull_vertices)
                        .unwrap_or_default(),
                    hull_planes: Slot::<Ref<records::targets::Vector4>>::slot(r.hull_planes)
                        .unwrap_or_default(),
                    positions: Slot::<Ref<records::targets::Vector3>>::slot(r.positions)
                        .unwrap_or_default(),
                    mesh_triangles: Slot::<Ref<records::targets::PhysicsMeshTriangle>>::slot(
                        r.triangles,
                    )
                    .unwrap_or_default(),
                    modern_positions: Slot::<Ref<records::targets::Vector4>>::slot(
                        r.mesh_vertex_positions,
                    )
                    .unwrap_or_default(),
                    modern_faces: !modern16.is_empty() || !modern32.is_empty(),
                };
                if input.modern_faces && input.kind == PhysicsShapeKind::TriangleMesh {
                    context.unsupported(
                        "m3.physics.shape.connectivity",
                        "the seven cooked mesh lanes have no verified vertex-index interpretation",
                    )?;
                    continue;
                }
                let auxiliary = geometry(document, &input, builder, context)?;
                let [x, y, z] = input.dimensions;
                let (geometry, auxiliary_geometry) = match input.kind {
                    PhysicsShapeKind::Box => (
                        ShapeGeometry::Box {
                            half_extents: [x, y, z],
                        },
                        auxiliary,
                    ),
                    PhysicsShapeKind::Sphere => (
                        ShapeGeometry::Sphere {
                            radius: x,
                            axis_controls: [y, z],
                        },
                        auxiliary,
                    ),
                    PhysicsShapeKind::Capsule => (
                        ShapeGeometry::Capsule {
                            radius: x,
                            length: y,
                            axis_control: z,
                        },
                        auxiliary,
                    ),
                    PhysicsShapeKind::Cylinder => (
                        ShapeGeometry::Cylinder {
                            radius: x,
                            length: y,
                            axis_control: z,
                        },
                        auxiliary,
                    ),
                    PhysicsShapeKind::ConvexHull => {
                        let auxiliary = auxiliary.ok_or_else(|| {
                            Error::new(
                                "m3.physics.shape.hull",
                                "convex geometry has neither input points nor planes",
                            )
                        })?;
                        (
                            ShapeGeometry::ConvexHull {
                                controls: input.dimensions,
                                mesh: auxiliary.mesh,
                                planes: auxiliary.planes,
                            },
                            None,
                        )
                    }
                    PhysicsShapeKind::TriangleMesh => {
                        if input.triangles.is_empty() && input.mesh_triangles.is_empty() {
                            return Err(Error::new(
                                "m3.physics.shape.triangles",
                                "triangle surface has no understood connectivity",
                            ));
                        }
                        let auxiliary = auxiliary.ok_or_else(|| {
                            Error::new(
                                "m3.physics.shape.mesh",
                                "triangle geometry has no positions",
                            )
                        })?;
                        (
                            ShapeGeometry::TriangleMesh {
                                controls: input.dimensions,
                                mesh: auxiliary.mesh,
                                planes: auxiliary.planes,
                            },
                            None,
                        )
                    }
                    kind => {
                        let (mesh, planes) = auxiliary.map_or((None, ZeroVec::new()), |value| {
                            (Some(value.mesh), value.planes)
                        });
                        (
                            ShapeGeometry::Unknown {
                                kind,
                                dimensions: [x, y, z],
                                mesh,
                                planes,
                            },
                            None,
                        )
                    }
                };
                output.push(Shape {
                    transform: input.transform,
                    geometry,
                    auxiliary_geometry,
                    margin: input.margin,
                });
            }
        };
    }
    match revision(document, reference).map_err(read_error)? {
        0 => read!(records::V0),
        1 => read!(records::V1),
        2 => read!(records::V2),
        3 => read!(records::V3),
        _ => {
            return Err(Error::new(
                "m3.physics.shape",
                "unsupported shape record revision",
            ));
        }
    }
    Ok(output)
}
