//! Fixtures retain their actual geometric and contact properties.
use super::*;
use crate::model::common::PackedArray;
use crate::model::error::Location;
use crate::model::io::materialize;
use crate::model::ValueGroups;
use glam::Mat4;
use glam::Vec3;
use native::records::PhysicsShapeKind;
use profile::{values::PhysicsGameFlags, MassProperties, Shape, ShapeGeometry};
pub(super) fn shapes<'a>(file: &'a PhysicsFile<'_>) -> Result<Vec<Shape<'a>>, Error> {
    let definitions: Vec<_> = file
        .chunks
        .iter()
        .flat_map(|chunk| match chunk {
            Pc::Shapes(rows) => rows.iter().collect::<Vec<_>>(),
            Pc::Shapes2(rows) => rows.iter().map(|row| &row.shape).collect(),
            _ => Vec::new(),
        })
        .collect();
    let mut out = Vec::new();
    for definition in definitions {
        let index = usize::try_from(definition.shape_index).map_err(|_| {
            Error::requirement(
                Location::new().field("physics").field("shape"),
                "negative geometry index",
            )
        })?;
        let mut transform = identity();
        let mut mass_properties = None;
        let geometry = match definition.shape_type {
            PhysicsShapeKind::Box => {
                let value = file
                    .chunks
                    .iter()
                    .filter_map(|chunk| {
                        if let Pc::Boxes(rows) = chunk {
                            Some(rows)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(index)
                    .ok_or_else(|| {
                        Error::requirement(
                            Location::new().field("physics").field("box"),
                            "missing geometry",
                        )
                    })?;
                let f = frame(&value.frame);
                transform = [
                    [f.axis_x[0], f.axis_x[1], f.axis_x[2], 0.],
                    [f.axis_y[0], f.axis_y[1], f.axis_y[2], 0.],
                    [f.axis_z[0], f.axis_z[1], f.axis_z[2], 0.],
                    [f.origin[0], f.origin[1], f.origin[2], 1.],
                ];
                ShapeGeometry::Box(Vec3::from_array(value.half_extents))
            }
            PhysicsShapeKind::Sphere => {
                let value = file
                    .chunks
                    .iter()
                    .filter_map(|chunk| {
                        if let Pc::Spheres(rows) = chunk {
                            Some(rows)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(index)
                    .ok_or_else(|| {
                        Error::requirement(
                            Location::new().field("physics").field("sphere"),
                            "missing geometry",
                        )
                    })?;
                transform[3] = [value.position[0], value.position[1], value.position[2], 1.];
                ShapeGeometry::Sphere(value.radius)
            }
            PhysicsShapeKind::Capsule => {
                let value = file
                    .chunks
                    .iter()
                    .filter_map(|chunk| {
                        if let Pc::Capsules(rows) = chunk {
                            Some(rows)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(index)
                    .ok_or_else(|| {
                        Error::requirement(
                            Location::new().field("physics").field("capsule"),
                            "missing geometry",
                        )
                    })?;
                ShapeGeometry::Capsule {
                    radius: value.radius,
                    endpoints: [
                        Vec3::from_array(value.position1),
                        Vec3::from_array(value.position2),
                    ],
                }
            }
            PhysicsShapeKind::ConvexHull => {
                let value = file
                    .chunks
                    .iter()
                    .filter_map(|chunk| {
                        if let Pc::Polytopes(rows) = chunk {
                            Some(rows)
                        } else {
                            None
                        }
                    })
                    .flatten()
                    .nth(index)
                    .ok_or_else(|| {
                        Error::requirement(
                            Location::new().field("physics").field("polytope"),
                            "missing geometry",
                        )
                    })?;
                let mut faces = Vec::new();
                for first in value.face_first_edge.iter() {
                    let first = first as usize;
                    let mut edge = first;
                    let mut face = Vec::new();
                    loop {
                        let row = value.edges.get(edge).ok_or_else(|| {
                            Error::requirement(
                                Location::new()
                                    .field("physics")
                                    .field("polytope")
                                    .field("faces"),
                                "edge is absent",
                            )
                        })?;
                        face.push(u32::from(row.origin_vertex));
                        edge = row.next_edge as usize;
                        if edge == first {
                            break;
                        }
                        if face.len() > value.edges.len() {
                            return Err(Error::requirement(
                                Location::new()
                                    .field("physics")
                                    .field("polytope")
                                    .field("faces"),
                                "boundary is not closed",
                            ));
                        }
                    }
                    faces.push(face);
                }
                mass_properties = Some(MassProperties {
                    centroid: Vec3::from_array(value.header.centroid),
                    volume: value.header.volume,
                    surface_area: value.header.surface_area,
                });
                ShapeGeometry::ConvexHull {
                    vertices: PackedArray::from_storage(
                        (materialize(value.vertices.len(), move |i| {
                            value.vertices.get(i).unwrap()
                        }))
                        .into_storage(),
                    ),
                    planes: PackedArray::from_storage(
                        (materialize(value.planes.len(), move |i| value.planes.get(i).unwrap()))
                            .into_storage(),
                    ),
                    faces: ValueGroups::from_rows(faces),
                }
            }
            PhysicsShapeKind::Unknown(value) => {
                return Err(Error::requirement(
                    Location::new().field("physics").field("shape"),
                    format!("unknown geometry selector {value}"),
                ));
            }
        };
        out.push(Shape {
            transform: Mat4::from_cols_array_2d(&transform),
            geometry,
            friction: definition.friction,
            restitution: definition.restitution,
            density: definition.density,
            mass_properties,
            game_flags: PhysicsGameFlags::from_bits_retain(definition.game_flags.bits()),
            collision_mask: Default::default(),
        });
    }
    Ok(out)
}
