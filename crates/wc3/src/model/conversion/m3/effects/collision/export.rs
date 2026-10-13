//! Relocates editable collision surfaces and selection volumes.
use super::*;
fn native_shape<E: Envelope>(
    source: &wem::RawDocument<'_>,
    doc: &mut native::Document<'static, E>,
    body: &wr::PhysicsBody<wr::V3>,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<nr::HitTestShape<nr::V1>, Error> {
    if body.shapes.entries != 1 {
        return Err(Error::new(
            "collision.shapes",
            "a native selection volume has exactly one shape",
        ));
    }
    let shape = source
        .resolve::<wr::PhysicsShape<wr::V2>>(body.shapes.cast())?
        .get(0)
        .ok_or_else(|| Error::new("collision.shape", "missing shape"))??;
    let points = source
        .resolve::<wr::Vector3>(shape.vertices.cast())?
        .iter()
        .map(|r| r.map(|r| nr::Vector3 { value: r.value }))
        .collect::<Result<Vec<_>, _>>()?;
    let faces = source
        .resolve::<wr::U32>(shape.triangles.cast())?
        .iter()
        .map(|r| {
            let r = r?;
            Ok(nr::U16 {
                value: u16::try_from(r.value)
                    .map_err(|_| Error::new("collision.faces", "collision index exceeds u16"))?,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    if !faces.len().is_multiple_of(3) || faces.iter().any(|r| r.value as usize >= points.len()) {
        return Err(Error::new("collision.faces", "invalid collision triangle"));
    }
    let kind = match shape.kind.0 {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        5 => 4,
        other => {
            context.unsupported(
                "collision.kind",
                format!("selection volume shape {other} has no established native encoding"),
            )?;
            0
        }
    };
    Ok(nr::HitTestShape {
        shape_type: kind,
        bone_index: if body.node == u32::MAX {
            u16::MAX
        } else {
            u16::try_from(
                *bone_map
                    .get(body.node as usize)
                    .ok_or_else(|| Error::new("collision.node", "missing shape anchor"))?,
            )
            .map_err(|_| Error::new("collision.node", "bone exceeds volume width"))?
        },
        transform: shape.transform,
        vertex_positions: append(doc, &points)?.cast(),
        face_indices: append(doc, &faces)?.cast(),
        size_x: if kind == 0 {
            shape.half_extents[0]
        } else {
            shape.radius
        },
        size_y: if kind == 0 {
            shape.half_extents[1]
        } else {
            shape.length
        },
        size_z: shape.half_extents[2],
        ..Default::default()
    })
}
pub(in crate::model::conversion::m3) fn export<E: Envelope>(
    model: &sem::Model<'_>,
    doc: &mut native::Document<'static, E>,
    bone_map: &[u32],
    context: &mut Context,
) -> Result<Collision, Error> {
    let mut out = Collision::default();
    let Some(native) = &model.native else {
        return Ok(out);
    };
    let source = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = source
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let values = root_metadata::values(model)?;
    out.bounds = nr::Extent {
        minimum: [0, 1, 2].map(|i| {
            f32::from_bits(
                values
                    .get(&format!("m3.collision.bounds[{i}]"))
                    .copied()
                    .unwrap_or(0) as u32,
            )
        }),
        maximum: [3, 4, 5].map(|i| {
            f32::from_bits(
                values
                    .get(&format!("m3.collision.bounds[{i}]"))
                    .copied()
                    .unwrap_or(0) as u32,
            )
        }),
        radius: f32::from_bits(values.get("m3.collision.bounds[6]").copied().unwrap_or(0) as u32),
    };
    let sets = source
        .resolve::<wr::ProfileMaterialSet>(record.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    if let Some(set) = sets.iter().find(|r| r.profile == model.profile) {
        for (key, destination) in [
            ("attachment_addons", &mut out.attachment_addons),
            ("camera_addons", &mut out.camera_addons),
            ("volume_addon0", &mut out.volume_addon0),
            ("volume_addon1", &mut out.volume_addon1),
        ] {
            if let Some(text) = metadata::get_text(source, &set.native, &format!("m3.{key}"))? {
                let rows = text
                    .split_whitespace()
                    .map(|word| {
                        word.parse::<u16>()
                            .map(|value| nr::U16 { value })
                            .map_err(|_| {
                                Error::new(
                                    format!("m3.{key}"),
                                    "addon value exceeds native u16 width",
                                )
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                *destination = append(doc, &rows)?.cast();
            }
        }
    }
    let mut fuzzy = Vec::new();
    let mut volumes = Vec::new();
    for body in source
        .resolve::<wr::PhysicsBody<wr::V3>>(record.physics.bodies.cast())?
        .iter()
    {
        let body = body?;
        let role = values
            .get(&format!("m3.collision.body[{}].role", body.id))
            .copied();
        if role.is_none() && body.node != u32::MAX {
            continue;
        }
        let mut hit = native_shape(source, doc, &body, bone_map, context)?;
        if body.id == TIGHT {
            hit.padding = values.get("m3.hit.tight.padding").copied().unwrap_or(0) as u16;
            out.tight = hit;
            continue;
        }
        match role {
            None | Some(8) => {
                if out.vertices.entries != 0 {
                    return Err(Error::new(
                        "collision",
                        "multiple static collision bodies require geometry merging",
                    ));
                }
                out.vertices = hit.vertex_positions;
                out.faces = hit.face_indices;
                let min = [0, 1, 2].map(|i| {
                    f32::from_bits(
                        values
                            .get(&format!("m3.collision.bounds[{i}]"))
                            .copied()
                            .unwrap_or(0) as u32,
                    )
                });
                let max = [3, 4, 5].map(|i| {
                    f32::from_bits(
                        values
                            .get(&format!("m3.collision.bounds[{i}]"))
                            .copied()
                            .unwrap_or(0) as u32,
                    )
                });
                out.bounds = nr::Extent {
                    minimum: min,
                    maximum: max,
                    radius: f32::from_bits(
                        values.get("m3.collision.bounds[6]").copied().unwrap_or(0) as u32,
                    ),
                };
                let sets = source
                    .resolve::<wr::ProfileMaterialSet>(record.profile_sets.cast())?
                    .iter()
                    .collect::<Result<Vec<_>, _>>()?;
                let bag = sets
                    .iter()
                    .find(|r| r.profile == model.profile)
                    .map(|r| &r.native);
                let mut normals = Vec::new();
                if let Some(bag) = bag {
                    if let Some(text) = metadata::get_text(source, bag, "m3.collision.normals")? {
                        let values = text
                            .split_whitespace()
                            .map(|word| {
                                word.parse::<f32>().map_err(|_| {
                                    Error::new("collision.normals", "invalid normal component")
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        if !values.len().is_multiple_of(3) || values.iter().any(|v| !v.is_finite())
                        {
                            return Err(Error::new(
                                "collision.normals",
                                "normals need finite xyz triples",
                            ));
                        }
                        normals = values
                            .as_chunks::<3>()
                            .0
                            .iter()
                            .map(|v| nr::Vector3 {
                                value: [v[0], v[1], v[2]],
                            })
                            .collect();
                    }
                }
                out.normals = append(doc, &normals)?.cast();
                if body.id == TIGHT {
                    out.tight = hit;
                }
            }
            Some(9) => {
                let mut hit = hit;
                let i = body.id - FUZZY;
                hit.padding = values
                    .get(&format!("m3.hit.fuzzy[{i}].padding"))
                    .copied()
                    .unwrap_or(0) as u16;
                fuzzy.push(hit);
            }
            Some(10) => {
                let i = body.id - VOLUME;
                let remap = |key: &str| -> Result<u32, Error> {
                    let value = values
                        .get(&format!("m3.attachment_volume[{i}].{key}"))
                        .copied()
                        .unwrap_or(0) as u32;
                    if value == u32::MAX {
                        return Ok(value);
                    }
                    bone_map.get(value as usize).copied().ok_or_else(|| {
                        Error::new("attachment_volume.node", "missing secondary anchor")
                    })
                };
                volumes.push(nr::AttachmentVolume {
                    bone1: remap("bone1")?,
                    bone2: remap("bone2")?,
                    shape_type: hit.shape_type,
                    bone_index: if hit.bone_index == u16::MAX {
                        u32::MAX
                    } else {
                        u32::from(hit.bone_index)
                    },
                    transform: hit.transform,
                    vertex_positions: hit.vertex_positions,
                    face_indices: hit.face_indices,
                    size_x: hit.size_x,
                    size_y: hit.size_y,
                    size_z: hit.size_z,
                });
            }
            Some(other) => {
                return Err(Error::new(
                    "collision.role",
                    format!("unknown collision role {other}"),
                ));
            }
        }
    }
    out.fuzzy = append(doc, &fuzzy)?.cast();
    out.volumes = append(doc, &volumes)?.cast();
    Ok(out)
}
