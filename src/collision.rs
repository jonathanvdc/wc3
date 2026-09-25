//! Box and sphere collision shapes in `CLID` chunks.

use crate::Record;
use crate::{ChunkRecord, Error, Model, Node};

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
    Box([[f32; 3]; 2]),
    Plane([[f32; 3]; 2]),
    Sphere([f32; 3], f32),
    Cylinder([[f32; 3]; 2], f32),
}

impl CollisionShape {
    /// Creates a box collision shape from two XYZ corners.
    pub fn new_box(node: Node, corners: [[f32; 3]; 2]) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Box(corners),
        }
    }

    /// Creates a sphere collision shape from center and radius.
    pub fn new_sphere(node: Node, center: [f32; 3], radius: f32) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Sphere(center, radius),
        }
    }

    /// Creates a plane collision shape from two XYZ points.
    pub fn new_plane(node: Node, points: [[f32; 3]; 2]) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Plane(points),
        }
    }

    /// Creates a cylinder collision shape from endpoints and radius.
    pub fn new_cylinder(node: Node, endpoints: [[f32; 3]; 2], radius: f32) -> Self {
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
    pub fn box_corners(&self) -> Option<[[f32; 3]; 2]> {
        match self.geometry {
            CollisionGeometry::Box(points) => Some(points),
            _ => None,
        }
    }

    /// Returns the two points of a box, plane, or cylinder.
    pub fn points(&self) -> Option<[[f32; 3]; 2]> {
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
    pub fn sphere(&self) -> Option<([f32; 3], f32)> {
        match self.geometry {
            CollisionGeometry::Sphere(center, radius) => Some((center, radius)),
            _ => None,
        }
    }
}

fn record_end(data: &[u8], offset: usize) -> Result<usize, Error> {
    let size_bytes = data
        .get(offset..offset.saturating_add(4))
        .ok_or(Error::MalformedRecord {
            tag: CollisionShape::TAG,
            offset,
        })?;
    let node_size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
    let node_end = offset
        .checked_add(node_size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord {
            tag: CollisionShape::TAG,
            offset,
        })?;
    Node::decode(&data[offset..node_end], 0)?;
    let kind_bytes =
        data.get(node_end..node_end.saturating_add(4))
            .ok_or(Error::MalformedRecord {
                tag: CollisionShape::TAG,
                offset: node_end,
            })?;
    let kind = u32::from_le_bytes(kind_bytes.try_into().expect("four-byte kind"));
    let data_size = match kind {
        0 | 1 => 24,
        2 => 16,
        3 => 28,
        _ => {
            return Err(Error::MalformedRecord {
                tag: CollisionShape::TAG,
                offset: node_end,
            })
        }
    };
    node_end
        .checked_add(4 + data_size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord {
            tag: CollisionShape::TAG,
            offset: node_end,
        })
}

impl Model {
    /// Decodes all collision shapes in `CLID` chunks.
    pub fn collision_shapes(&self) -> Result<Vec<CollisionShape>, Error> {
        let mut shapes = Vec::new();
        for chunk in self
            .chunks()
            .iter()
            .filter(|chunk| chunk.tag == CollisionShape::TAG)
        {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let end = record_end(&chunk.data, offset)?;
                shapes.push(CollisionShape::decode(&chunk.data[offset..end], 0)?);
                offset = end;
            }
        }
        Ok(shapes)
    }

    /// Replaces collision shapes in the first `CLID` chunk.
    pub fn set_collision_shapes(&mut self, shapes: &[CollisionShape]) -> Result<(), Error> {
        let size = shapes.iter().try_fold(0usize, |sum, shape| {
            sum.checked_add(shape.encode()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: CollisionShape::TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for shape in shapes {
            data.extend_from_slice(&shape.encode()?);
        }
        self.replace_chunks(CollisionShape::TAG, data);
        Ok(())
    }
}

impl Record for CollisionShape {
    fn decode(bytes: &[u8], _version: u32) -> Result<Self, Error> {
        if record_end(bytes, 0)? != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: CollisionShape::TAG,
                offset: 0,
            });
        }
        let node_size =
            u32::from_le_bytes(bytes[..4].try_into().expect("validated node size")) as usize;
        let node = Node::decode(&bytes[..node_size], 0)?;
        let kind = u32::from_le_bytes(
            bytes[node_size..node_size + 4]
                .try_into()
                .expect("validated kind"),
        );
        let start = node_size + 4;
        let vec3 = |offset| {
            std::array::from_fn(|axis| {
                f32::from_le_bytes(
                    bytes[offset + axis * 4..offset + axis * 4 + 4]
                        .try_into()
                        .expect("validated coordinate"),
                )
            })
        };
        let scalar = |offset| {
            f32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .expect("validated radius"),
            )
        };
        let geometry = match kind {
            0 => CollisionGeometry::Box([vec3(start), vec3(start + 12)]),
            1 => CollisionGeometry::Plane([vec3(start), vec3(start + 12)]),
            2 => CollisionGeometry::Sphere(vec3(start), scalar(start + 12)),
            3 => CollisionGeometry::Cylinder([vec3(start), vec3(start + 12)], scalar(start + 24)),
            _ => unreachable!("validated kind"),
        };
        Ok(Self { node, geometry })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok({
            let mut bytes = self.node.encode()?;
            let kind = match self.geometry {
                CollisionGeometry::Box(_) => 0u32,
                CollisionGeometry::Plane(_) => 1,
                CollisionGeometry::Sphere(_, _) => 2,
                CollisionGeometry::Cylinder(_, _) => 3,
            };
            bytes.extend_from_slice(&kind.to_le_bytes());
            let mut push_vec3 = |values: [f32; 3]| {
                for value in values {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
            };
            match self.geometry {
                CollisionGeometry::Box(points) | CollisionGeometry::Plane(points) => {
                    for point in points {
                        push_vec3(point);
                    }
                }
                CollisionGeometry::Sphere(center, radius) => {
                    push_vec3(center);
                    bytes.extend_from_slice(&radius.to_le_bytes());
                }
                CollisionGeometry::Cylinder(points, radius) => {
                    for point in points {
                        push_vec3(point);
                    }
                    bytes.extend_from_slice(&radius.to_le_bytes());
                }
            }
            bytes
        })
    }
}

impl ChunkRecord for CollisionShape {
    const TAG: [u8; 4] = *b"CLID";
}
