//! Box and sphere collision shapes in `CLID` chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::ModelVersion;
use crate::Vec3;

use crate::CollisionShapesChunk;
use crate::Cursor;
use crate::{DecodeError, Model, Node};
use crate::{Encodable, Readable};

/// Warcraft III collision primitive type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollisionKind {
    /// Two XYZ corners.
    Box,
    /// Two XYZ points describing a plane.
    Plane,
    /// Center XYZ and radius.
    Sphere,
    /// Two XYZ endpoints and a radius.
    Cylinder,
}

/// A collision primitive attached to a node.
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionShape {
    node: Node,
    geometry: CollisionGeometry,
}

#[derive(Clone, Debug, PartialEq)]
enum CollisionGeometry {
    Box([Vec3; 2]),
    Plane([Vec3; 2]),
    Sphere(Vec3, f32),
    Cylinder([Vec3; 2], f32),
}

impl CollisionShape {
    /// Creates a box collision shape from two XYZ corners.
    pub fn new_box(node: Node, corners: [Vec3; 2]) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Box(corners),
        }
    }

    /// Creates a sphere collision shape from center and radius.
    pub fn new_sphere(node: Node, center: Vec3, radius: f32) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Sphere(center, radius),
        }
    }

    /// Creates a plane collision shape from two XYZ points.
    pub fn new_plane(node: Node, points: [Vec3; 2]) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Plane(points),
        }
    }

    /// Creates a cylinder collision shape from endpoints and radius.
    pub fn new_cylinder(node: Node, endpoints: [Vec3; 2], radius: f32) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Cylinder(endpoints, radius),
        }
    }

    /// Borrows the attached node.
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Borrows the attached node for editing.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    /// Returns the collision primitive kind.
    pub fn kind(&self) -> CollisionKind {
        match self.geometry {
            CollisionGeometry::Box(_) => CollisionKind::Box,
            CollisionGeometry::Plane(_) => CollisionKind::Plane,
            CollisionGeometry::Sphere(_, _) => CollisionKind::Sphere,
            CollisionGeometry::Cylinder(_, _) => CollisionKind::Cylinder,
        }
    }

    /// Returns the two box corners, or `None` for other shapes.
    pub fn box_corners(&self) -> Option<[Vec3; 2]> {
        match self.geometry {
            CollisionGeometry::Box(points) => Some(points),
            _ => None,
        }
    }

    /// Returns the two points of a box, plane, or cylinder.
    pub fn points(&self) -> Option<[Vec3; 2]> {
        match self.geometry {
            CollisionGeometry::Box(points)
            | CollisionGeometry::Plane(points)
            | CollisionGeometry::Cylinder(points, _) => Some(points),
            _ => None,
        }
    }

    /// Returns the radius of a sphere or cylinder.
    pub fn radius(&self) -> Option<f32> {
        match self.geometry {
            CollisionGeometry::Sphere(_, radius) | CollisionGeometry::Cylinder(_, radius) => {
                Some(radius)
            }
            _ => None,
        }
    }

    /// Returns sphere center and radius, or `None` for other shapes.
    pub fn sphere(&self) -> Option<(Vec3, f32)> {
        match self.geometry {
            CollisionGeometry::Sphere(center, radius) => Some((center, radius)),
            _ => None,
        }
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all collision shapes in `CLID` chunks.
    pub fn collision_shapes(&self) -> Vec<CollisionShape> {
        self.collect_chunk_records::<CollisionShapesChunk>()
    }

    /// Replaces collision shapes in the first `CLID` chunk.
    pub fn set_collision_shapes(&mut self, shapes: &[CollisionShape]) {
        self.replace_chunk(CollisionShapesChunk::new(shapes.to_vec()));
    }
}

impl Readable for CollisionShape {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let node = cursor.read()?;
        let kind_offset = cursor.absolute_position();
        let kind = cursor.read::<u32>()?;
        let geometry = match kind {
            0 => CollisionGeometry::Box([cursor.read()?, cursor.read()?]),
            1 => CollisionGeometry::Plane([cursor.read()?, cursor.read()?]),
            2 => CollisionGeometry::Sphere(cursor.read()?, cursor.read()?),
            3 => CollisionGeometry::Cylinder([cursor.read()?, cursor.read()?], cursor.read()?),
            _ => {
                return Err(DecodeError::MalformedRecord {
                    tag: CollisionShapesChunk::TAG,
                    offset: kind_offset,
                })
            }
        };
        Ok(Self { node, geometry })
    }
}

impl Encodable for CollisionShape {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        self.node.encode_to(bytes)?;
        let kind = match self.geometry {
            CollisionGeometry::Box(_) => 0u32,
            CollisionGeometry::Plane(_) => 1,
            CollisionGeometry::Sphere(_, _) => 2,
            CollisionGeometry::Cylinder(_, _) => 3,
        };
        bytes.write(kind);

        match self.geometry {
            CollisionGeometry::Box(points) | CollisionGeometry::Plane(points) => {
                bytes.write(points);
            }
            CollisionGeometry::Sphere(center, radius) => {
                bytes.write(center);
                bytes.write(radius);
            }
            CollisionGeometry::Cylinder(points, radius) => {
                bytes.write(points);
                bytes.write(radius);
            }
        }
        Ok(())
    }
}
