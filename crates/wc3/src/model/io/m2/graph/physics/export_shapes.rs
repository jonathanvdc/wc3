//! Rebuilds fixture tables and half-edge connectivity from logical geometry.
use super::*;
use glam::Mat4;
use glam::Vec3;
use native::companions::{
    PhysicsBox, PhysicsCapsule, PhysicsEdge, PhysicsShape, PhysicsShape2, PhysicsSphere, Polytope,
    PolytopeHeader,
};
use native::records::{PhysicsGameFlags, PhysicsShapeKind};
use native::ZeroVec;
use profile::{Shape, ShapeGeometry};
use std::collections::BTreeMap;
#[derive(Default)]
pub(super) struct Geometry {
    pub boxes: Vec<PhysicsBox>,
    pub capsules: Vec<PhysicsCapsule>,
    pub spheres: Vec<PhysicsSphere>,
    pub polytopes: Vec<Polytope<'static>>,
    pub shapes: Vec<PhysicsShape2>,
}
impl Geometry {
    pub(super) fn add(&mut self, shape: &Shape<'_>, context: &mut Context) -> Result<(), Error> {
        if shape.collision_mask.bits() != 0 {
            context.unsupported(
                "physics.shape.collision_mask",
                "M2 fixtures have no independent collision mask",
            )?;
        }
        let (kind, index) = match &shape.geometry {
            ShapeGeometry::Box(extents) => {
                let index = self.boxes.len();
                self.boxes.push(PhysicsBox {
                    frame: native_frame(profile::Frame {
                        axis_x: shape.transform.x_axis.truncate(),
                        axis_y: shape.transform.y_axis.truncate(),
                        axis_z: shape.transform.z_axis.truncate(),
                        origin: shape.transform.w_axis.truncate(),
                    }),
                    half_extents: (*extents).to_array(),
                });
                (PhysicsShapeKind::Box, index)
            }
            ShapeGeometry::Sphere(radius) => {
                let scale = uniform_scale(shape.transform.to_cols_array_2d())?;
                let index = self.spheres.len();
                self.spheres.push(PhysicsSphere {
                    position: shape.transform.w_axis.truncate().to_array(),
                    radius: radius * scale,
                });
                (PhysicsShapeKind::Sphere, index)
            }
            ShapeGeometry::Capsule { radius, endpoints } => {
                let scale = uniform_scale(shape.transform.to_cols_array_2d())?;
                let index = self.capsules.len();
                self.capsules.push(PhysicsCapsule {
                    position1: point(shape.transform.to_cols_array_2d(), endpoints[0].to_array()),
                    position2: point(shape.transform.to_cols_array_2d(), endpoints[1].to_array()),
                    radius: radius * scale,
                });
                (PhysicsShapeKind::Capsule, index)
            }
            ShapeGeometry::ConvexHull {
                vertices,
                planes,
                faces,
            } => {
                if shape.transform != Mat4::from_cols_array_2d(&identity()) {
                    return Err(Error::new(
                        "physics.polytope.transform",
                        "native polytope export requires geometry in fixture coordinates",
                    ));
                }
                let index = self.polytopes.len();
                let points = vertices.to_vec();
                let polygons = faces.to_vec();
                if points.len() > 256 || polygons.len() > 256 {
                    return Err(Error::new(
                        "physics.polytope",
                        "geometry exceeds byte-indexed topology",
                    ));
                }
                let mut edges: Vec<PhysicsEdge> = Vec::new();
                let mut directed = BTreeMap::new();
                let mut used = std::collections::BTreeSet::new();
                let mut first = Vec::new();
                for (face, polygon) in polygons.iter().enumerate() {
                    let mut boundary = Vec::new();
                    for (&from, &to) in polygon
                        .iter()
                        .zip(polygon.iter().cycle().skip(1))
                        .take(polygon.len())
                    {
                        if !used.insert((from, to)) {
                            return Err(Error::new(
                                "physics.polytope.faces",
                                "directed edge belongs to multiple faces",
                            ));
                        }
                        let edge = if let Some(index) = directed.get(&(from, to)) {
                            *index
                        } else {
                            let index = edges.len();
                            directed.insert((from, to), index);
                            directed.insert((to, from), index + 1);
                            edges.push(PhysicsEdge {
                                origin_vertex: u8::try_from(from).map_err(|_| {
                                    Error::new(
                                        "physics.polytope",
                                        "vertex index exceeds byte range",
                                    )
                                })?,
                                twin_offset: 1,
                                ..Default::default()
                            });
                            edges.push(PhysicsEdge {
                                origin_vertex: u8::try_from(to).map_err(|_| {
                                    Error::new(
                                        "physics.polytope",
                                        "vertex index exceeds byte range",
                                    )
                                })?,
                                twin_offset: -1,
                                ..Default::default()
                            });
                            index
                        };
                        boundary.push(edge);
                    }
                    if edges.len() > 256 {
                        return Err(Error::new(
                            "physics.polytope",
                            "edge count exceeds byte-indexed topology",
                        ));
                    }
                    first.push(
                        u8::try_from(*boundary.first().ok_or_else(|| {
                            Error::new("physics.polytope.faces", "empty boundary")
                        })?)
                        .map_err(|_| {
                            Error::new("physics.polytope", "edge index exceeds byte range")
                        })?,
                    );
                    for (&edge, &next) in boundary
                        .iter()
                        .zip(boundary.iter().cycle().skip(1))
                        .take(boundary.len())
                    {
                        edges[edge].face_index = face as u8;
                        edges[edge].next_edge = next as u8;
                    }
                }
                if directed.keys().any(|edge| !used.contains(edge)) {
                    return Err(Error::new(
                        "physics.polytope.faces",
                        "convex boundary is not closed",
                    ));
                }
                let measured = if let Some(mass) = shape.mass_properties {
                    mass
                } else {
                    let hull = hull::hull(points.iter().map(|v| v.to_array()).collect())?;
                    profile::MassProperties {
                        centroid: Vec3::from_array(hull.header.centroid),
                        volume: hull.header.volume,
                        surface_area: hull.header.surface_area,
                    }
                };
                self.polytopes.push(Polytope {
                    header: PolytopeHeader {
                        centroid: (measured.centroid).to_array(),
                        volume: measured.volume,
                        surface_area: measured.surface_area,
                        ..Default::default()
                    },
                    vertices: ZeroVec::alloc_from_slice(
                        &points.iter().map(|v| v.to_array()).collect::<Vec<_>>(),
                    ),
                    planes: ZeroVec::alloc_from_slice(
                        &planes
                            .to_vec()
                            .iter()
                            .map(|v| v.to_array())
                            .collect::<Vec<_>>(),
                    ),
                    face_first_edge: ZeroVec::alloc_from_slice(&first),
                    edges,
                });
                (PhysicsShapeKind::ConvexHull, index)
            }
        };
        self.shapes.push(PhysicsShape2 {
            shape: PhysicsShape {
                shape_type: kind,
                shape_index: i16::try_from(index).map_err(|_| {
                    Error::new("physics.shape", "geometry index exceeds native range")
                })?,
                game_flags: PhysicsGameFlags::from_bits_retain(shape.game_flags.bits()),
                friction: shape.friction,
                restitution: shape.restitution,
                density: shape.density,
                ..Default::default()
            },
            ..Default::default()
        });
        Ok(())
    }
}
fn point(matrix: [[f32; 4]; 4], point: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|axis| {
        matrix[3][axis] + (0..3).map(|i| matrix[i][axis] * point[i]).sum::<f32>()
    })
}
fn uniform_scale(matrix: [[f32; 4]; 4]) -> Result<f32, Error> {
    let lengths: [f32; 3] =
        std::array::from_fn(|row| matrix[row][..3].iter().map(|v| v * v).sum::<f32>().sqrt());
    let scale = lengths[0];
    let orthogonal = (0..3).all(|a| {
        (0..3).all(|b| {
            a == b
                || (0..3)
                    .map(|i| matrix[a][i] * matrix[b][i])
                    .sum::<f32>()
                    .abs()
                    <= 0.000001
        })
    });
    if !orthogonal
        || lengths
            .iter()
            .any(|length| (*length - scale).abs() > 0.000001)
    {
        return Err(Error::new(
            "physics.shape.transform",
            "sphere and capsule require a uniform orthogonal transform",
        ));
    }
    Ok(scale)
}
