//! Collision primitives attached to model nodes.
use crate::model::common::Ownership;
use crate::model::formats::mdlx::mdl::{Parser, Span, Writer};
use crate::model::formats::mdlx::scene::{set_node_kind, validate_node_kind};
use crate::model::formats::mdlx::CollisionShapesChunk;
use crate::model::formats::mdlx::Cursor;
use crate::model::formats::mdlx::Dialect;
use crate::model::formats::mdlx::Encoder;
use crate::model::formats::mdlx::IoError;
use crate::model::formats::mdlx::KnownChunk;
use crate::model::formats::mdlx::Vec3;
use crate::model::formats::mdlx::{mdl, mdx};
use crate::model::formats::mdlx::{Document, Node};
use std::io::Write as IoWrite;

/// A collision primitive attached to a node.
///
/// Use the shape constructors to initialize the dimensions and node flags.
/// Edit [`CollisionGeometry`] to change the shape directly. Sphere and cylinder
/// shapes have a radius; boxes and planes are described by two points.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, Ownership, mdx::ReadOwned)]
pub struct CollisionShape<'data> {
    /// Attached node.
    pub node: Node<'data>,
    /// Shape and dimensions in the node's local coordinate system.
    pub geometry: CollisionGeometry,
}

#[derive(Clone, Debug, PartialEq)]
/// The shape and dimensions of a collision primitive.
pub enum CollisionGeometry {
    /// An axis-aligned box described by two corners.
    Box([Vec3; 2]),
    /// A plane primitive described by two stored vertices.
    Plane([Vec3; 2]),
    /// A sphere described by its center and radius.
    Sphere(Vec3, f32),
    /// A cylinder described by two stored vertices and a radius.
    Cylinder([Vec3; 2], f32),
}

impl<'data> mdx::Read<'data> for CollisionGeometry {
    fn read_mdx(cursor: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
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
            _ => Err(mdx::ReadError::new(
                kind_offset,
                mdx::ReadErrorKind::UnknownEnumValue {
                    enum_name: "CollisionShape",
                    value: kind,
                },
            )
            .with_tag(CollisionShapesChunk::TAG)),
        }
    }
}

impl mdx::Write for CollisionGeometry {
    fn write_mdx(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
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

impl<'data> CollisionShape<'data> {
    /// Creates a box collision shape from two XYZ corners.
    pub fn new_box(node: Node<'data>, corners: [Vec3; 2]) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Box(corners),
        }
    }

    /// Creates a sphere collision shape from center and radius.
    pub fn new_sphere(node: Node<'data>, center: Vec3, radius: f32) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Sphere(center, radius),
        }
    }

    /// Creates a plane collision shape from two XYZ points.
    pub fn new_plane(node: Node<'data>, points: [Vec3; 2]) -> Self {
        Self {
            node,
            geometry: CollisionGeometry::Plane(points),
        }
    }

    /// Creates a cylinder collision shape from endpoints and radius.
    pub fn new_cylinder(node: Node<'data>, endpoints: [Vec3; 2], radius: f32) -> Self {
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

impl<'data, D: Dialect> Document<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn collision_shapes<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'s CollisionShape<'data>> + use<'data, 's, D> {
        self.records::<CollisionShapesChunk<'_>>()
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn collision_shapes_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut CollisionShape<'data>> + use<'data, 's, D> {
        self.records_mut::<CollisionShapesChunk<'_>>()
    }

    /// Replaces collision shapes with one `CLID` chunk, removing any duplicate chunks.
    pub fn set_collision_shapes(
        &mut self,
        shapes: impl IntoIterator<Item = CollisionShape<'data>>,
    ) {
        self.replace_chunk(CollisionShapesChunk::new(shapes.into_iter().collect()));
    }
}

#[derive(mdl::Read, mdl::Write)]
#[mdl(choice)]
enum CollisionKind {
    Box,
    Plane,
    Sphere,
    Cylinder,
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
struct CollisionMdl<'data> {
    #[mdl(flatten)]
    node: Node<'data>,
    #[mdl(flatten)]
    kind: CollisionKind,
    #[mdl(counted = "Vertices")]
    vertices: Vec<CollisionVertex>,
    #[mdl(property = "BoundsRadius", delegate)]
    radius: Option<f32>,
}
fn finish_collision<'data>(value: &mut CollisionMdl<'data>, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x2000);
    Ok(())
}
fn validate_collision_read<'data>(
    value: &CollisionMdl<'data>,
    span: Span,
) -> Result<(), mdl::ReadError> {
    let expected = if matches!(value.kind, CollisionKind::Sphere) {
        1
    } else {
        2
    };
    if value.vertices.len() != expected {
        return Err(mdl::ReadError::new(
            span,
            mdl::ReadErrorKind::CountMismatch {
                expected,
                actual: value.vertices.len(),
            },
        ));
    }
    if matches!(value.kind, CollisionKind::Sphere | CollisionKind::Cylinder) {
        if value.radius.is_none() {
            return Err(mdl::ReadError::new(
                span,
                mdl::ReadErrorKind::MissingField("BoundsRadius"),
            ));
        }
    } else if value.radius.is_some() {
        return Err(mdl::ReadError::new(
            span,
            mdl::ReadErrorKind::UnsupportedField,
        ));
    }
    Ok(())
}
fn validate_collision_write<'data>(value: &CollisionMdl<'data>) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x2000)
}
impl<'data> mdl::Read for CollisionShape<'data> {
    fn read_mdl(parser: &mut Parser<'_>) -> Result<Self, mdl::ReadError> {
        let value = parser.read::<CollisionMdl<'data>>()?;
        let first = value.vertices[0].0;
        let geometry = match value.kind {
            CollisionKind::Box => CollisionGeometry::Box([first, value.vertices[1].0]),
            CollisionKind::Plane => CollisionGeometry::Plane([first, value.vertices[1].0]),
            CollisionKind::Sphere => {
                CollisionGeometry::Sphere(first, value.radius.expect("validated radius"))
            }
            CollisionKind::Cylinder => CollisionGeometry::Cylinder(
                [first, value.vertices[1].0],
                value.radius.expect("validated radius"),
            ),
        };
        Ok(Self {
            node: value.node,
            geometry,
        })
    }
}
impl<'data> mdl::Write for CollisionShape<'data> {
    fn write_mdl<W: IoWrite>(
        &self,
        writer: &mut Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        let (kind, vertices, radius) = match self.geometry {
            CollisionGeometry::Box(points) => (
                CollisionKind::Box,
                points.into_iter().map(CollisionVertex).collect(),
                None,
            ),
            CollisionGeometry::Plane(points) => (
                CollisionKind::Plane,
                points.into_iter().map(CollisionVertex).collect(),
                None,
            ),
            CollisionGeometry::Sphere(center, radius) => (
                CollisionKind::Sphere,
                vec![CollisionVertex(center)],
                Some(radius),
            ),
            CollisionGeometry::Cylinder(points, radius) => (
                CollisionKind::Cylinder,
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
