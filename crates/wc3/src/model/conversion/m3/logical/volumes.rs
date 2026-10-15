//! Selection and attachment volumes refer to checked collision meshes and actual bones.
use super::{
    materials::{read_error, revision},
    write::append,
    Graph, GraphBuilder,
};
use crate::model::conversion::materialize;
use crate::model::conversion::{Context, Error};
use crate::model::m3::{self as m3, records, Document, Envelope, Ref};
use crate::model::semantic::{
    profiles::m3::{values::VolumeShape, AttachmentVolume, Feature, HitTestRole, HitTestShape},
    GeometrySchema as _, M3CollisionGeometry, M3Geometry, Mesh, MeshId, NodeId, Topology,
};
use std::array::from_fn;
use std::collections::BTreeMap;

fn node(index: u32, nodes: &[NodeId]) -> Result<NodeId, Error> {
    nodes
        .get(index as usize)
        .copied()
        .ok_or_else(|| Error::new("m3.volume.bone", "volume references an undeclared bone"))
}
fn mesh<'a, E: Envelope + Send + Sync>(
    document: &'a Document<'_, E>,
    positions: Ref<records::targets::Vector3>,
    faces: Ref<records::targets::U16>,
    builder: &mut GraphBuilder<'a>,
) -> Result<Option<MeshId>, Error> {
    if positions.is_empty() && faces.is_empty() {
        return Ok(None);
    }
    let points = document
        .resolve::<records::Vector3>(positions.cast())
        .map_err(read_error)?;
    for row in points.iter() {
        row.map_err(read_error)?;
    }
    let indices = document
        .resolve::<records::U16>(faces.cast())
        .map_err(read_error)?;
    for row in indices.iter() {
        row.map_err(read_error)?;
    }
    if indices.len() % 3 != 0 {
        return Err(Error::new(
            "m3.volume.faces",
            "volume connectivity must contain complete triangles",
        ));
    }
    let points = materialize(points.len(), move |i| {
        points
            .get(i)
            .expect("checked volume point")
            .expect("validated volume point")
            .value
    });
    let geometry = if indices.is_empty() {
        M3CollisionGeometry::points(points)?
    } else {
        let triangles = materialize(indices.len() / 3, move |i| {
            from_fn(|lane| {
                u32::from(
                    indices
                        .get(i * 3 + lane)
                        .expect("checked corner")
                        .expect("validated corner")
                        .value,
                )
            })
        });
        M3CollisionGeometry::new(points, triangles)?
    };
    Ok(Some(
        builder.add_mesh(Mesh::new(M3Geometry::Collision(geometry)))?,
    ))
}
fn hit<'a, E: Envelope + Send + Sync>(
    document: &'a Document<'_, E>,
    row: records::HitTestShape<records::V1>,
    role: HitTestRole,
    nodes: &[NodeId],
    builder: &mut GraphBuilder<'a>,
    context: &mut Context,
) -> Result<(), Error> {
    if row.padding != 0 {
        context.unsupported(
            "m3.hit_shape.padding",
            "nonzero alignment bytes have no established authored meaning",
        )?;
    }
    let geometry = mesh(document, row.vertex_positions, row.face_indices, builder)?;
    builder.add_feature(
        node(u32::from(row.bone_index), nodes)?,
        Feature::HitTest(
            (HitTestShape {
                role,
                shape_type: VolumeShape::from_raw(row.shape_type.raw()),
                transform: row.transform,
                mesh: geometry,
                size_x: row.size_x,
                size_y: row.size_y,
                size_z: row.size_z,
            })
            .into(),
        ),
    )?;
    Ok(())
}
pub(super) fn read<'a, E: Envelope + Send + Sync>(
    source: &'a m3::Model<'_, E>,
    nodes: &[NodeId],
    builder: &mut GraphBuilder<'a>,
    context: &mut Context,
) -> Result<(), Error> {
    let root = source.root_record().map_err(read_error)?;
    let (tight, fuzzy, attachments) = m3::visit_m3_root!(root, |r| (
        r.tight_hit_test_object,
        r.fuzzy_hit_test_objects,
        r.attachment_volumes
    ));
    let document = source.document();
    if tight != Default::default() {
        hit(document, tight, HitTestRole::Tight, nodes, builder, context)?;
    }
    if !fuzzy.is_empty() {
        match revision(document, fuzzy).map_err(read_error)? {
            0 => {
                for row in document
                    .resolve::<records::HitTestShape<records::V0>>(fuzzy.cast())
                    .map_err(read_error)?
                    .iter()
                {
                    let row = row.map_err(read_error)?;
                    hit(
                        document,
                        records::HitTestShape {
                            shape_type: row.shape_type,
                            bone_index: row.bone_index,
                            padding: row.padding,
                            transform: row.transform,
                            vertex_positions: row.vertex_positions,
                            face_indices: row.face_indices,
                            size_x: row.size_x,
                            size_y: row.size_y,
                            size_z: row.size_z,
                        },
                        HitTestRole::Fuzzy,
                        nodes,
                        builder,
                        context,
                    )?;
                }
            }
            1 => {
                for row in document
                    .resolve::<records::HitTestShape<records::V1>>(fuzzy.cast())
                    .map_err(read_error)?
                    .iter()
                {
                    hit(
                        document,
                        row.map_err(read_error)?,
                        HitTestRole::Fuzzy,
                        nodes,
                        builder,
                        context,
                    )?;
                }
            }
            _ => {
                return Err(Error::new(
                    "m3.hit_shapes",
                    "unknown selection volume revision",
                ));
            }
        }
    }
    for row in document
        .resolve::<records::AttachmentVolume>(attachments.cast())
        .map_err(read_error)?
        .iter()
    {
        let row = row.map_err(read_error)?;
        let geometry = mesh(document, row.vertex_positions, row.face_indices, builder)?;
        builder.add_feature(
            node(row.bone_index, nodes)?,
            Feature::AttachmentVolume(
                (AttachmentVolume {
                    bone1: node(row.bone1, nodes)?,
                    bone2: node(row.bone2, nodes)?,
                    shape_type: VolumeShape::from_raw(row.shape_type.raw()),
                    transform: row.transform,
                    mesh: geometry,
                    size_x: row.size_x,
                    size_y: row.size_y,
                    size_z: row.size_z,
                })
                .into(),
            ),
        )?;
    }
    Ok(())
}
fn bone(id: NodeId, nodes: &BTreeMap<NodeId, u32>) -> Result<u32, Error> {
    nodes
        .get(&id)
        .copied()
        .ok_or_else(|| Error::new("m3.volume.bone", "volume anchor has no native bone"))
}
fn geometry<E: Envelope>(
    graph: &Graph<'_>,
    id: Option<MeshId>,
    document: &mut Document<'_, E>,
) -> Result<(Ref<records::targets::Vector3>, Ref<records::targets::U16>), Error> {
    let Some(id) = id else {
        return Ok((Ref::default(), Ref::default()));
    };
    let mesh = graph
        .mesh(id)
        .ok_or_else(|| Error::new("m3.volume.mesh", "volume mesh does not exist"))?;
    let M3Geometry::Collision(geometry) = &mesh.geometry else {
        return Err(Error::new(
            "m3.volume.mesh",
            "volume geometry must be collision geometry",
        ));
    };
    let positions: Vec<_> = geometry
        .positions()
        .iter()
        .map(|value| records::Vector3 { value })
        .collect();
    let indices = match geometry.topology() {
        Topology::Triangles(triangles) => triangles
            .iter()
            .flatten()
            .map(|index| {
                u16::try_from(index)
                    .map(|value| records::U16 { value })
                    .map_err(|_| {
                        Error::new(
                            "m3.volume.faces",
                            "volume vertex index exceeds 16-bit capacity",
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?,
        Topology::Points(_) => Vec::new(),
        _ => {
            return Err(Error::new(
                "m3.volume.faces",
                "selection volumes require triangle connectivity",
            ));
        }
    };
    if geometry.triangle_controls().is_some() || geometry.face_normals().is_some() {
        return Err(Error::new(
            "m3.volume.mesh",
            "selection volume records cannot encode triangle controls or face normals",
        ));
    }
    Ok((
        append(document, &positions)?.cast(),
        append(document, &indices)?.cast(),
    ))
}
#[derive(Default)]
pub(super) struct Output {
    pub tight: records::HitTestShape<records::V1>,
    pub fuzzy: Ref<records::targets::HitTestShape>,
    pub attachments: Ref<records::targets::AttachmentVolume>,
}
pub(super) fn write<E: Envelope>(
    graph: &Graph<'_>,
    document: &mut Document<'_, E>,
    nodes: &BTreeMap<NodeId, u32>,
) -> Result<Output, Error> {
    let mut result = Output::default();
    let mut fuzzy = Vec::new();
    let mut attachments = Vec::new();
    let mut tight = false;
    for (_, feature) in graph.features() {
        match &feature.payload {
            Feature::HitTest(value) => {
                let (positions, indices) = geometry(graph, value.mesh, document)?;
                let row = records::HitTestShape::<records::V1> {
                    shape_type: records::VolumeShape::from_raw(value.shape_type.raw()),
                    bone_index: u16::try_from(bone(feature.node, nodes)?).map_err(|_| {
                        Error::new("m3.hit_shape.bone", "bone index exceeds 16-bit capacity")
                    })?,
                    padding: 0,
                    transform: value.transform,
                    vertex_positions: positions,
                    face_indices: indices,
                    size_x: value.size_x,
                    size_y: value.size_y,
                    size_z: value.size_z,
                };
                match value.role {
                    HitTestRole::Fuzzy => fuzzy.push(row),
                    HitTestRole::Tight => {
                        if tight {
                            return Err(Error::new(
                                "m3.hit_shape.tight",
                                "native M3 permits one precise volume",
                            ));
                        }
                        tight = true;
                        result.tight = row;
                    }
                }
            }
            Feature::AttachmentVolume(value) => {
                let (positions, indices) = geometry(graph, value.mesh, document)?;
                attachments.push(records::AttachmentVolume {
                    bone1: bone(value.bone1, nodes)?,
                    bone2: bone(value.bone2, nodes)?,
                    bone_index: bone(feature.node, nodes)?,
                    shape_type: records::VolumeShape::from_raw(value.shape_type.raw()),
                    transform: value.transform,
                    vertex_positions: positions,
                    face_indices: indices,
                    size_x: value.size_x,
                    size_y: value.size_y,
                    size_z: value.size_z,
                });
            }
            _ => {}
        }
    }
    result.fuzzy = append(document, &fuzzy)?.cast();
    result.attachments = append(document, &attachments)?.cast();
    Ok(result)
}
