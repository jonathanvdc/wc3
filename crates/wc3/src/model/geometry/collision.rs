//! Box and sphere collision shapes in `CLID` chunks.
use crate::model::mdl::{MdlWriter, Parser, ReadErrorKind, Span};
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::Encoder;
use crate::model::KnownChunk;
use crate::model::ModelVersion;
use crate::model::Vec3;
use crate::model::WriteError;
use crate::model::{mdl, mdx};
use std::io::Write as IoWrite;

use crate::model::CollisionShapesChunk;
use crate::model::Cursor;
use crate::model::{Model, Node, ReadError};

/// A collision primitive attached to a node.
///
/// MDL reading reconstructs the collision object-kind bit; writing requires
/// matching node bits. Box and Plane carry two vertices, Sphere one, and
/// Cylinder two. Only Sphere and Cylinder have a BoundsRadius property.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
pub struct CollisionShape {
    /// Attached node.
    pub node: Node,
    pub geometry: CollisionGeometry,
}

#[derive(Clone, Debug, PartialEq)]
/// The shape and dimensions of a collision primitive.
pub enum CollisionGeometry {
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

#[derive(mdl::Read, mdl::Write)]
#[mdl(entry)]
struct CollisionVertex(Vec3);

#[derive(mdl::Read, mdl::Write)]
#[mdl(
    block = "CollisionShape",
    after_read = "finish_collision",
    validate_read = "validate_collision_read",
    validate_write = "validate_collision_write"
)]
struct CollisionMdl {
    #[mdl(flatten)]
    node: Node,
    #[mdl(flags(Box = 1, Plane = 2, Sphere = 4, Cylinder = 8))]
    kind: u32,
    #[mdl(counted = "Vertices")]
    vertices: Vec<CollisionVertex>,
    #[mdl(property = "BoundsRadius", delegate)]
    radius: Option<f32>,
}
fn finish_collision(value: &mut CollisionMdl, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x2000);
    Ok(())
}
fn validate_collision_read(value: &CollisionMdl, span: Span) -> Result<(), mdl::ReadError> {
    if !value.kind.is_power_of_two() {
        return Err(mdl::ReadError::new(
            span,
            ReadErrorKind::Expected("one collision-type flag"),
        ));
    }
    let expected = if value.kind == 4 { 1 } else { 2 };
    if value.vertices.len() != expected {
        return Err(mdl::ReadError::new(
            span,
            ReadErrorKind::CountMismatch {
                expected,
                actual: value.vertices.len(),
            },
        ));
    }
    if matches!(value.kind, 4 | 8) {
        if value.radius.is_none() {
            return Err(mdl::ReadError::new(
                span,
                ReadErrorKind::MissingField("BoundsRadius"),
            ));
        }
    } else if value.radius.is_some() {
        return Err(mdl::ReadError::new(span, ReadErrorKind::UnsupportedField));
    }
    Ok(())
}
fn validate_collision_write(value: &CollisionMdl) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x2000)
}
impl mdl::Read for CollisionShape {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let value = parser.read::<CollisionMdl>()?;
        let first = value.vertices[0].0;
        let geometry = match value.kind {
            1 => CollisionGeometry::Box([first, value.vertices[1].0]),
            2 => CollisionGeometry::Plane([first, value.vertices[1].0]),
            4 => CollisionGeometry::Sphere(first, value.radius.expect("validated radius")),
            8 => CollisionGeometry::Cylinder(
                [first, value.vertices[1].0],
                value.radius.expect("validated radius"),
            ),
            _ => unreachable!("validated collision type"),
        };
        Ok(Self {
            node: value.node,
            geometry,
        })
    }
}
impl mdl::Write for CollisionShape {
    fn write_mdl<W: IoWrite>(&self, writer: &mut MdlWriter<W>) -> Result<(), mdl::WriteError> {
        let (kind, vertices, radius) = match self.geometry {
            CollisionGeometry::Box(points) => {
                (1, points.into_iter().map(CollisionVertex).collect(), None)
            }
            CollisionGeometry::Plane(points) => {
                (2, points.into_iter().map(CollisionVertex).collect(), None)
            }
            CollisionGeometry::Sphere(center, radius) => {
                (4, vec![CollisionVertex(center)], Some(radius))
            }
            CollisionGeometry::Cylinder(points, radius) => (
                8,
                points.into_iter().map(CollisionVertex).collect(),
                Some(radius),
            ),
        };
        writer.write(&CollisionMdl {
            node: self.node.clone(),
            kind,
            vertices,
            radius,
        })
    }
}
