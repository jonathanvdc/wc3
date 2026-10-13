//! Static collision and selection volumes as editable physics shapes.
use super::super::{append, metadata as root_metadata};
use super::{
    metadata, native, nr, read_error, revision, sem, wem, wr, Context, Envelope, Error, Ref,
    IDENTITY,
};
use std::collections::BTreeMap;
const STATIC: u32 = 0x8000_0000;
const TIGHT: u32 = 0x8000_0001;
const FUZZY: u32 = 0x9000_0000;
const VOLUME: u32 = 0xa000_0000;
fn points<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::Vector3>,
) -> Result<Vec<wr::Vector3>, Error> {
    source
        .resolve::<nr::Vector3>(reference.cast())
        .map_err(read_error)?
        .iter()
        .map(|r| {
            r.map(|r| wr::Vector3 { value: r.value })
                .map_err(read_error)
        })
        .collect()
}
fn faces<E: Envelope>(
    source: &native::Document<'_, E>,
    reference: Ref<nr::targets::U16>,
    vertices: usize,
) -> Result<Vec<wr::U32>, Error> {
    let rows = source
        .resolve::<nr::U16>(reference.cast())
        .map_err(read_error)?
        .iter()
        .map(|r| {
            r.map(|r| wr::U32 {
                value: u32::from(r.value),
            })
            .map_err(read_error)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !rows.len().is_multiple_of(3) || rows.iter().any(|r| r.value as usize >= vertices) {
        return Err(Error::new(
            "collision.faces",
            "invalid collision triangle index",
        ));
    }
    Ok(rows)
}
fn body(
    doc: &mut wem::RawDocument<'static>,
    id: u32,
    node: u32,
    shape: wr::PhysicsShape<wr::V2>,
) -> Result<wr::PhysicsBody<wr::V3>, Error> {
    Ok(wr::PhysicsBody {
        id,
        node,
        motion: wr::BodyMotion::from_raw(2),
        shapes: metadata::put(doc, &[shape])?.cast(),
        ..Default::default()
    })
}
fn shape<E: Envelope>(
    source: &native::Document<'_, E>,
    doc: &mut wem::RawDocument<'static>,
    r: nr::HitTestShape<nr::V1>,
    context: &mut Context,
) -> Result<wr::PhysicsShape<wr::V2>, Error> {
    let vertices = points(source, r.vertex_positions)?;
    let triangles = faces(source, r.face_indices, vertices.len())?;
    let kind = match r.shape_type {
        nr::VolumeShape::Box => 0,
        nr::VolumeShape::Sphere => 1,
        nr::VolumeShape::Capsule => 2,
        nr::VolumeShape::Cylinder => 3,
        nr::VolumeShape::Mesh => 5,
        nr::VolumeShape::Unknown(other) => {
            context.unsupported(
                "hit_test.kind",
                format!("uninterpreted native hit-test shape kind {other}"),
            )?;
            0
        }
    };
    Ok(wr::PhysicsShape {
        kind: wr::PhysicsShapeKind::from_raw(kind),
        transform: r.transform,
        half_extents: [r.size_x, r.size_y, r.size_z],
        radius: r.size_x,
        length: r.size_y,
        vertices: metadata::put(doc, &vertices)?.cast(),
        triangles: metadata::put(doc, &triangles)?.cast(),
        ..Default::default()
    })
}
#[derive(Clone, Copy, Debug, Default)]
/// Editable collision references assigned to a native model root.
pub struct Collision {
    /// Attachment addon values whose native interpretation is unresolved.
    pub attachment_addons: Ref<nr::targets::U16>,
    /// Camera addon values whose native interpretation is unresolved.
    pub camera_addons: Ref<nr::targets::U16>,
    /// First attachment-volume addon array.
    pub volume_addon0: Ref<nr::targets::U16>,
    /// Second attachment-volume addon array.
    pub volume_addon1: Ref<nr::targets::U16>,
    /// Bounding extent of static collision geometry.
    pub bounds: nr::Extent,
    /// Static collision vertex positions.
    pub vertices: Ref<nr::targets::Vector3>,
    /// Static triangle indices.
    pub faces: Ref<nr::targets::U16>,
    /// Authored collision normals.
    pub normals: Ref<nr::targets::Vector3>,
    /// Primary selection volume.
    pub tight: nr::HitTestShape<nr::V1>,
    /// Additional selection volumes.
    pub fuzzy: Ref<nr::targets::HitTestShape>,
    /// Attachment influence volumes.
    pub volumes: Ref<nr::targets::AttachmentVolume>,
}

mod export;
mod import;
pub(in crate::model::conversion::m3) use export::export;
pub(in crate::model::conversion::m3) use import::attach;
trait OptionalArray {
    fn array(&self) -> Ref<nr::targets::U16>;
}
impl OptionalArray for () {
    fn array(&self) -> Ref<nr::targets::U16> {
        Ref::default()
    }
}
impl OptionalArray for Ref<nr::targets::U16> {
    fn array(&self) -> Self {
        *self
    }
}
