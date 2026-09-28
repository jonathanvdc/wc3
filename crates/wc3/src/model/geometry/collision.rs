//! Box and sphere collision shapes in `CLID` chunks.
use crate::model::mdx;
use crate::model::Encoder;
use crate::model::KnownChunk;
use crate::model::ModelVersion;
use crate::model::Vec3;
use crate::model::WriteError;

use crate::model::CollisionShapesChunk;
use crate::model::Cursor;
use crate::model::{Model, Node, ReadError};

/// A collision primitive attached to a node.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
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

impl mdx::Read for CollisionGeometry {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let kind_offset = cursor.absolute_position();
        let kind = cursor.read::<u32>()?;
        match kind {
            0 => Ok(Self::Box([cursor.read()?, cursor.read()?])),
            1 => Ok(Self::Plane([cursor.read()?, cursor.read()?])),
            2 => Ok(Self::Sphere(cursor.read()?, cursor.read()?)),
            3 => Ok(Self::Cylinder(
                [cursor.read()?, cursor.read()?],
                cursor.read()?,
            )),
            _ => Err(ReadError::MalformedRecord {
                tag: CollisionShapesChunk::TAG,
                offset: kind_offset,
            }),
        }
    }
}

impl mdx::Write for CollisionGeometry {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        let kind = match self {
            Self::Box(_) => 0u32,
            Self::Plane(_) => 1,
            Self::Sphere(_, _) => 2,
            Self::Cylinder(_, _) => 3,
        };
        bytes.write(&kind)?;

        match self {
            Self::Box(points) | Self::Plane(points) => bytes.write(points)?,
            Self::Sphere(center, radius) => {
                bytes.write(center)?;
                bytes.write(radius)?;
            }
            Self::Cylinder(points, radius) => {
                bytes.write(points)?;
                bytes.write(radius)?;
            }
        }
        Ok(())
    }
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
