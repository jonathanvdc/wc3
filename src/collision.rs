//! Box and sphere collision shapes in `CLID` chunks.

use crate::{Error, Model, Node};

const TAG: [u8; 4] = *b"CLID";

/// Warcraft III collision primitive type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollisionKind {
    /// Two XYZ corners.
    Box,
    /// Center XYZ and radius.
    Sphere,
}

/// A collision primitive attached to a node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollisionShape {
    bytes: Vec<u8>,
}

impl CollisionShape {
    /// Creates a box collision shape from two XYZ corners.
    pub fn new_box(node: Node, corners: [[f32; 3]; 2]) -> Self {
        let mut bytes = node.as_bytes().to_vec();
        bytes.extend_from_slice(&0u32.to_le_bytes());
        for corner in corners {
            for coordinate in corner {
                bytes.extend_from_slice(&coordinate.to_le_bytes());
            }
        }
        Self { bytes }
    }

    /// Creates a sphere collision shape from center and radius.
    pub fn new_sphere(node: Node, center: [f32; 3], radius: f32) -> Self {
        let mut bytes = node.as_bytes().to_vec();
        bytes.extend_from_slice(&2u32.to_le_bytes());
        for coordinate in center {
            bytes.extend_from_slice(&coordinate.to_le_bytes());
        }
        bytes.extend_from_slice(&radius.to_le_bytes());
        Self { bytes }
    }

    /// Wraps one complete shape record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if record_end(bytes, 0)? != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the attached node.
    pub fn node(&self) -> Node {
        let size =
            u32::from_le_bytes(self.bytes[..4].try_into().expect("validated node size")) as usize;
        Node::from_bytes(&self.bytes[..size]).expect("validated node")
    }

    /// Returns the collision primitive kind.
    pub fn kind(&self) -> CollisionKind {
        let offset = self.node_size();
        match u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("validated kind"),
        ) {
            0 => CollisionKind::Box,
            2 => CollisionKind::Sphere,
            _ => unreachable!("validated kind"),
        }
    }

    /// Returns the two box corners, or `None` for a sphere.
    pub fn box_corners(&self) -> Option<[[f32; 3]; 2]> {
        if self.kind() != CollisionKind::Box {
            return None;
        }
        let start = self.node_size() + 4;
        Some(std::array::from_fn(|corner| {
            std::array::from_fn(|axis| self.f32_at(start + (corner * 3 + axis) * 4))
        }))
    }

    /// Returns sphere center and radius, or `None` for a box.
    pub fn sphere(&self) -> Option<([f32; 3], f32)> {
        if self.kind() != CollisionKind::Sphere {
            return None;
        }
        let start = self.node_size() + 4;
        let center = std::array::from_fn(|axis| self.f32_at(start + axis * 4));
        Some((center, self.f32_at(start + 12)))
    }

    fn node_size(&self) -> usize {
        u32::from_le_bytes(self.bytes[..4].try_into().expect("validated node size")) as usize
    }

    fn f32_at(&self, offset: usize) -> f32 {
        f32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("validated coordinate"),
        )
    }
}

fn record_end(data: &[u8], offset: usize) -> Result<usize, Error> {
    let size_bytes = data
        .get(offset..offset.saturating_add(4))
        .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
    let node_size = u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
    let node_end = offset
        .checked_add(node_size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
    Node::from_bytes(&data[offset..node_end])?;
    let kind_bytes =
        data.get(node_end..node_end.saturating_add(4))
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: node_end,
            })?;
    let kind = u32::from_le_bytes(kind_bytes.try_into().expect("four-byte kind"));
    let data_size = match kind {
        0 => 24,
        2 => 16,
        _ => {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: node_end,
            })
        }
    };
    node_end
        .checked_add(4 + data_size)
        .filter(|&end| end <= data.len())
        .ok_or(Error::MalformedRecord {
            tag: TAG,
            offset: node_end,
        })
}

impl Model {
    /// Decodes all collision shapes in `CLID` chunks.
    pub fn collision_shapes(&self) -> Result<Vec<CollisionShape>, Error> {
        let mut shapes = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let end = record_end(&chunk.data, offset)?;
                shapes.push(CollisionShape::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(shapes)
    }

    /// Replaces collision shapes in the first `CLID` chunk.
    pub fn set_collision_shapes(&mut self, shapes: &[CollisionShape]) -> Result<(), Error> {
        let size = shapes.iter().try_fold(0usize, |sum, shape| {
            sum.checked_add(shape.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for shape in shapes {
            data.extend_from_slice(shape.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
