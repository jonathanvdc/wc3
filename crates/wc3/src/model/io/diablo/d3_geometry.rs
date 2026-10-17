//! Materialized logical arrays from native vertex records and explicit quantization on output.
use super::{bounds, i32_count, name, native, native_bounds, parent, text_owned};
use crate::model::conversion::{Context, Error};
use crate::model::io::materialize;
use crate::model::profiles::diablo::d3::{Geometry, MeshControls, VertexExtras, VertexStreams, D3};
use crate::model::Topology;
use crate::model::{Influence, MaterialSlotId, Mesh, NodeId};
use std::{array::from_fn, collections::BTreeMap};
use zerovec::ZeroVec;
pub(super) fn read<'a>(
    sub: &'a native::SubObject<'_>,
    bones: &[NodeId],
    slot: Option<MaterialSlotId>,
) -> Result<Mesh<'a, D3>, Error> {
    if !sub.indices.len().is_multiple_of(3) {
        return Err(Error::new(
            "d3.indices",
            "index count is not a triangle multiple",
        ));
    }
    if !sub.vertex_influences.is_empty() && sub.vertex_influences.len() != sub.vertices.len() {
        return Err(Error::new(
            "d3.influences",
            "influence count differs from vertex count",
        ));
    }
    for row in sub.vertex_influences.iter() {
        for influence in [row.influence0, row.influence1, row.influence2] {
            if influence.bone_index < 0 || influence.bone_index as usize >= bones.len() {
                return Err(Error::new(
                    "d3.influences",
                    "influence joint is outside skeleton",
                ));
            }
        }
    }
    let vertices = &sub.vertices;
    let streams = VertexStreams {
        positions: materialize(vertices.len(), move |i| vertices.get(i).unwrap().position),
        normals: materialize(vertices.len(), move |i| {
            vertices.get(i).unwrap().normal_vector()
        }),
        colors: materialize(vertices.len(), move |i| {
            vertices.get(i).unwrap().rgba().map(|v| f32::from(v) / 255.)
        }),
        texture_coordinates: [
            materialize(vertices.len(), move |i| vertices.get(i).unwrap().uv0()),
            materialize(vertices.len(), move |i| vertices.get(i).unwrap().uv1()),
        ],
        tangents: materialize(vertices.len(), move |i| {
            vertices.get(i).unwrap().tangent_vector()
        }),
        extras: VertexExtras {
            direction_auxiliary: materialize(vertices.len(), move |i| {
                let vertex = vertices.get(i).unwrap();
                [vertex.normal, vertex.tangent, vertex.binormal].map(|word| (word >> 24) as u8)
            }),
            auxiliary_colors: materialize(vertices.len(), move |i| {
                vertices
                    .get(i)
                    .unwrap()
                    .aux_color
                    .to_le_bytes()
                    .map(|v| f32::from(v) / 255.)
            }),
            binormals: materialize(vertices.len(), move |i| {
                vertices.get(i).unwrap().binormal_vector()
            }),
            cloth_vertices: materialize(vertices.len(), move |i| {
                vertices.get(i).unwrap().cloth_vertex_index
            }),
        },
    };
    let indices = &sub.indices;
    let triangles = materialize(indices.len() / 3, move |i| {
        from_fn(|j| u32::from(indices.get(i * 3 + j).unwrap()))
    });
    let influences = &sub.vertex_influences;
    let rows = materialize(influences.len(), move |i| {
        let r = influences.get(i).unwrap();
        [r.influence0, r.influence1, r.influence2].map(|v| Influence {
            joint: v.bone_index as u32,
            weight: v.weight,
        })
    });
    let mut mesh =
        Mesh::new(Geometry::new(streams, triangles, rows)?).with_controls(MeshControls {
            vertex_flags: sub.vertex_format,
            material_slot: slot,
            shape_name: text_owned(&sub.material_name)?,
            surface: sub.surface,
            bone: parent(sub.bone_index, bones)?,
            unclassified_control: sub.unknown58,
        });
    mesh.bounds = bounds(sub.bounds);
    Ok(mesh)
}
fn quantize(
    value: f32,
    scale: f32,
    bias: f32,
    max: u32,
    context: &mut Context,
    path: &str,
) -> Result<u32, Error> {
    let scaled = (value + bias) * scale;
    if !scaled.is_finite() || scaled < 0. || scaled > max as f32 {
        return Err(Error::new(path, "value is outside native packed range"));
    }
    let packed = scaled.round() as u32;
    let decoded = packed as f32 / scale - bias;
    context.approximate(
        path,
        (value - decoded).abs(),
        "native packed component requires quantization",
    )?;
    Ok(packed)
}
fn vector(value: [f32; 3], auxiliary: u8, context: &mut Context) -> Result<u32, Error> {
    let mut out = u32::from(auxiliary) << 24;
    for (i, v) in value.into_iter().enumerate() {
        let scaled = ((v + 1.) * (255. / 2.)).round();
        if !scaled.is_finite() || !(0. ..=255.).contains(&scaled) {
            return Err(Error::new(
                "d3.vector",
                "vector component is outside native range",
            ));
        }
        let p = scaled as u32;
        let decoded = p as f32 / 127.5 - 1.;
        context.approximate(
            "d3.vector",
            (v - decoded).abs(),
            "native vector component requires quantization",
        )?;
        out |= p << (i * 8);
    }
    Ok(out)
}

fn color(value: [f32; 4], context: &mut Context) -> Result<u32, Error> {
    let mut out = 0;
    for (i, v) in value.into_iter().enumerate() {
        out |= quantize(v, 255., 0., 255, context, "d3.color")? << (i * 8);
    }
    Ok(out)
}
fn uv(value: [f32; 2], context: &mut Context) -> Result<u32, Error> {
    Ok(quantize(value[0], 512., 64., 65535, context, "d3.uv")?
        | (quantize(value[1], 512., 64., 65535, context, "d3.uv")? << 16))
}
pub(super) fn write(
    mesh: &Mesh<'_, D3>,
    slot_name: &str,
    palette: &[u32],
    bones: &BTreeMap<NodeId, i32>,
    context: &mut Context,
) -> Result<native::SubObject<'static>, Error> {
    if mesh.bounds.sphere_radius != 0. {
        context.unsupported(
            "d3.mesh.bounds.sphere_radius",
            "native subobject AABB has no independent sphere radius",
        )?;
    }
    let streams = mesh.geometry.streams();
    let count = streams.positions.len();
    let mut vertices = Vec::with_capacity(count);
    for i in 0..count {
        let auxiliary = streams.extras.direction_auxiliary.get(i).unwrap_or([0; 3]);
        vertices.push(native::FatVertex {
            position: streams.positions.get(i).unwrap(),
            normal: vector(streams.normals.get(i).unwrap(), auxiliary[0], context)?,
            color: color(streams.colors.get(i).unwrap(), context)?,
            aux_color: color(streams.extras.auxiliary_colors.get(i).unwrap(), context)?,
            tex_coord0: uv(streams.texture_coordinates[0].get(i).unwrap(), context)?,
            tex_coord1: uv(streams.texture_coordinates[1].get(i).unwrap(), context)?,
            tangent: vector(streams.tangents.get(i).unwrap(), auxiliary[1], context)?,
            binormal: vector(
                streams.extras.binormals.get(i).unwrap(),
                auxiliary[2],
                context,
            )?,
            cloth_vertex_index: streams.extras.cloth_vertices.get(i).unwrap(),
        });
    }
    let Topology::Triangles(triangles) = mesh.topology() else {
        return Err(Error::new(
            "d3.topology",
            "native subobjects require triangles",
        ));
    };
    let mut indices = Vec::new();
    for triangle in triangles.iter() {
        for index in triangle {
            indices.push(
                u16::try_from(index)
                    .map_err(|_| Error::new("d3.indices", "vertex index exceeds u16"))?,
            );
        }
    }
    let mut influences = Vec::new();
    for row in mesh.geometry.influence_rows().iter() {
        let mut slots = [native::Influence::default(); 3];
        for (i, v) in row.into_iter().enumerate() {
            let joint = *palette
                .get(v.joint as usize)
                .ok_or_else(|| Error::new("d3.skin", "joint index is outside placement skin"))?;
            slots[i] = native::Influence {
                bone_index: joint as i32,
                weight: v.weight,
            };
        }
        influences.push(native::VertexInfluences {
            influence0: slots[0],
            influence1: slots[1],
            influence2: slots[2],
        });
    }
    if !mesh.name.is_empty() {
        context.unsupported("d3.mesh.name","native subobject names are material-slot references, not independent geometry display labels")?;
    }
    Ok(native::SubObject {
        vertex_format: mesh.controls.vertex_flags,
        vertex_count: i32_count(vertices.len())?,
        vertices: ZeroVec::alloc_from_slice(&vertices),
        vertex_influences: ZeroVec::alloc_from_slice(&influences),
        index_count: i32_count(indices.len())?,
        indices: ZeroVec::alloc_from_slice(&indices),
        surface: mesh.controls.surface,
        bone_index: match mesh.controls.bone {
            Some(id) => *bones
                .get(&id)
                .ok_or_else(|| Error::new("d3.mesh.bone", "placement bone is absent"))?,
            None => -1,
        },
        unknown58: mesh.controls.unclassified_control,
        name: name(slot_name)?,
        material_name: name(&mesh.controls.shape_name)?,
        bounds: native_bounds(mesh.bounds),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {

    use super::{vector, Context};
    use crate::model::conversion::ExportOptions;
    use crate::model::formats::diablo::d3::records::FatVertex;

    #[test]
    fn every_native_direction_byte_is_exactly_exportable() {
        let mut context = Context::export(&ExportOptions::default());
        for channel in u8::MIN..=u8::MAX {
            let packed = u32::from_le_bytes([channel, 255 - channel, channel / 2, 0xa5]);
            let native = FatVertex {
                normal: packed,
                ..Default::default()
            };
            assert_eq!(
                vector(native.normal_vector(), 0xa5, &mut context).unwrap(),
                packed
            );
        }
    }
}
