use super::vertices::{self, ElementSemantic, VertexLayout};
use super::{count, nonnegative};
use crate::model::common::IntoZeroVec;
use crate::model::conversion::{Context, Error};
use crate::model::formats::diablo::d4::records as native;
use crate::model::profiles::diablo::d4::{Geometry, Segment};
use crate::model::GeometryInfluences;
use crate::model::{GeometryData, Influence, NodeId, Topology};
use std::collections::BTreeMap;
use zerovec::ZeroVec;

pub(super) fn read(
    sub: &native::SubObject<'_>,
    chunk: &native::GeoChunk<'_>,
    layout: &VertexLayout,
    bones: &[NodeId],
    context: &mut Context,
) -> Result<(Geometry<'static>, Vec<Segment>), Error> {
    let vb = chunk
        .chunk_vertex_buffers
        .get(nonnegative(sub.vert_buffer_index, "vertex buffer")?)
        .ok_or_else(|| error("vertex buffer index exceeds chunk"))?;
    let ib = chunk
        .chunk_index_buffers
        .get(nonnegative(sub.index_buffer_index, "index buffer")?)
        .ok_or_else(|| error("index buffer index exceeds chunk"))?;
    let available = layout.validate_buffer(vb)?;
    let bytes = vb.chunk_vertices.as_bytes();
    let mut data = GeometryData::default();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut tangents = Vec::new();
    let mut colors = Vec::new();
    let uv_count = layout
        .elements
        .iter()
        .filter(|e| matches!(e.semantic, ElementSemantic::TexCoord(_)))
        .count();
    let mut uvs = vec![Vec::new(); uv_count];
    let mut triangles = Vec::new();
    let mut rows = Vec::new();
    let mut segments = Vec::new();
    let mut gaps = false;
    for source in &sub.segments {
        let vertex_bytes = nonnegative(source.vert_offset, "segment vertex offset")?;
        let vertex_count = nonnegative(source.vert_count, "segment vertex count")?;
        let index_bytes = nonnegative(source.index_offset, "segment index offset")?;
        let index_count = nonnegative(source.index_count, "segment index count")?;
        if vertex_bytes % layout.stride as usize != 0
            || index_bytes % 2 != 0
            || index_count % 3 != 0
        {
            return Err(error(
                "segment offsets are unaligned or connectivity is not triangles",
            ));
        }
        let first = vertex_bytes / layout.stride as usize;
        if first
            .checked_add(vertex_count)
            .is_none_or(|end| end > available)
        {
            return Err(error("segment vertex window exceeds selected buffer"));
        }
        let index_first = index_bytes / 2;
        if index_first
            .checked_add(index_count)
            .is_none_or(|end| end > ib.pdw_chunk_indices.len())
        {
            return Err(error("segment index window exceeds selected buffer"));
        }
        let palette: Vec<_> = source
            .bone_ids
            .iter()
            .map(|i| {
                bones
                    .get(nonnegative(i, "palette bone")?)
                    .copied()
                    .ok_or_else(|| error("segment palette bone exceeds skeleton"))
            })
            .collect::<Result<_, _>>()?;
        let logical_first = positions.len();
        let triangle_first = triangles.len();
        for vertex in first..first + vertex_count {
            let mut joint_indices = None;
            let mut joint_weights = None;
            for &element in &layout.elements {
                let v = vertices::read(bytes, vertex, layout.stride, element)?;
                match element.semantic {
                    ElementSemantic::Position => positions.push([v[0], v[1], v[2]]),
                    ElementSemantic::Normal => normals.push([v[0], v[1], v[2]]),
                    ElementSemantic::Tangent => tangents.push(v),
                    ElementSemantic::Color => colors.push(v),
                    ElementSemantic::TexCoord(slot) => uvs[slot as usize].push([v[0], v[1]]),
                    ElementSemantic::JointIndices => joint_indices = Some(v),
                    ElementSemantic::JointWeights => joint_weights = Some(v),
                }
            }
            if let (Some(indices), Some(weights)) = (joint_indices, joint_weights) {
                let mut row = [Influence {
                    joint: 0,
                    weight: 0.,
                }; 4];
                for slot in 0..4 {
                    let palette_index = indices[slot] as usize;
                    let node = palette
                        .get(palette_index)
                        .ok_or_else(|| error("vertex palette index exceeds segment palette"))?;
                    let joint = bones
                        .iter()
                        .position(|id| id == node)
                        .ok_or_else(|| error("palette bone is absent from skeleton"))?;
                    row[slot] = Influence {
                        joint: count(joint)?,
                        weight: weights[slot],
                    };
                }
                rows.push(row);
            }
            let start = vertex * layout.stride as usize;
            for byte in 0..layout.stride as usize {
                if !layout.elements.iter().any(|e| {
                    byte >= e.offset as usize && byte < (e.offset + e.encoding.width()) as usize
                }) && bytes[start + byte] != 0
                {
                    gaps = true;
                }
            }
        }
        for triangle in 0..index_count / 3 {
            let mut values = [0; 3];
            for (corner, value) in values.iter_mut().enumerate() {
                let local = usize::from(
                    ib.pdw_chunk_indices
                        .get(index_first + triangle * 3 + corner)
                        .expect("checked range"),
                );
                if local >= vertex_count {
                    return Err(error("segment triangle exceeds its local vertex window"));
                }
                *value = count(logical_first + local)?;
            }
            triangles.push(values);
        }
        segments.push(Segment {
            first_vertex: count(logical_first)?,
            vertex_count: count(vertex_count)?,
            first_triangle: count(triangle_first)?,
            triangle_count: count(index_count / 3)?,
            bones: palette,
        });
    }
    if gaps {
        context.unsupported("d4.geometry.vertex_padding","nonzero bytes outside explicitly interpreted vertex elements cannot define semantic attributes")?;
    }
    data.positions = positions.into_zero_vec();
    data.set_normals(normals.into_zero_vec());
    data.set_tangents(tangents.into_zero_vec());
    data.set_colors(colors.into_zero_vec());
    data.set_texture_coordinates(uvs.into_iter().map(IntoZeroVec::into_zero_vec).collect());
    data.influences = if layout
        .elements
        .iter()
        .any(|element| element.semantic == ElementSemantic::JointWeights)
    {
        GeometryInfluences::Four(rows.into_zero_vec())
    } else {
        GeometryInfluences::None
    };
    data.connectivity.primary = Topology::Triangles(triangles.into_zero_vec());
    Ok((Geometry::new(data)?, segments))
}

pub(super) struct EncodedGeometry {
    pub vertex_buffer: native::GeoChunkVertexBuffer<'static>,
    pub index_buffer: native::GeoChunkIndexBuffer<'static>,
    pub segments: Vec<native::SubObjectSegment<'static>>,
}
pub(super) fn write(
    geometry: &Geometry<'_>,
    segments: &[Segment],
    layout: &VertexLayout,
    bones: &BTreeMap<NodeId, u32>,
    palette: &[NodeId],
    context: &mut Context,
) -> Result<EncodedGeometry, Error> {
    layout.validate()?;
    let data = geometry.data().clone().into_parts();
    let Topology::Triangles(triangles) = &data.topology else {
        return Err(error("native subobjects require triangle connectivity"));
    };
    if !data.attributes.is_empty() {
        context.unsupported(
            "d4.geometry.attributes",
            "additional authored attributes have no declared native vertex element",
        )?;
    }
    for (present, semantic) in [
        (!data.normals.is_empty(), ElementSemantic::Normal),
        (!data.tangents.is_empty(), ElementSemantic::Tangent),
        (!data.colors.is_empty(), ElementSemantic::Color),
        (!data.influences.is_empty(), ElementSemantic::JointWeights),
    ] {
        if present && layout.element(semantic).is_none() {
            context.unsupported(
                "d4.geometry.stream",
                "authored vertex stream is absent from the selected output layout",
            )?;
        }
    }
    if data.texture_coordinates.len()
        > layout
            .elements
            .iter()
            .filter(|e| matches!(e.semantic, ElementSemantic::TexCoord(_)))
            .count()
    {
        context.unsupported(
            "d4.geometry.texture_coordinates",
            "output layout has fewer texture coordinate sets than authored geometry",
        )?;
    }
    let count_vertices = data.positions.len();
    let total = count_vertices
        .checked_mul(layout.stride as usize)
        .ok_or_else(|| error("output vertex bytes overflow"))?;
    let mut bytes = vec![0; total];
    let mut indices = Vec::new();
    let mut native_segments = Vec::new();
    let fallback;
    let segments = if segments.is_empty() {
        fallback = vec![Segment {
            first_vertex: 0,
            vertex_count: count(count_vertices)?,
            first_triangle: 0,
            triangle_count: count(triangles.len())?,
            bones: palette.to_vec(),
        }];
        fallback.as_slice()
    } else {
        segments
    };
    let mut next_vertex = 0;
    let mut next_triangle = 0;
    for segment in segments {
        let first = segment.first_vertex as usize;
        let len = segment.vertex_count as usize;
        let first_triangle = segment.first_triangle as usize;
        let triangles_len = segment.triangle_count as usize;
        if first != next_vertex
            || first_triangle != next_triangle
            || first
                .checked_add(len)
                .is_none_or(|end| end > count_vertices)
            || first_triangle
                .checked_add(triangles_len)
                .is_none_or(|end| end > triangles.len())
        {
            return Err(error(
                "semantic segments must partition vertices and triangles in order",
            ));
        }
        next_vertex = first + len;
        next_triangle = first_triangle + triangles_len;
        let native_bones: Vec<_> = segment
            .bones
            .iter()
            .map(|id| {
                bones
                    .get(id)
                    .copied()
                    .and_then(|v| i32::try_from(v).ok())
                    .ok_or_else(|| error("segment palette selects a non-bone node"))
            })
            .collect::<Result<_, _>>()?;
        for vertex in first..first + len {
            for &element in &layout.elements {
                let value = match element.semantic {
                    ElementSemantic::Position => {
                        let v = data.positions.get(vertex).expect("validated stream");
                        [v[0], v[1], v[2], 0.]
                    }
                    ElementSemantic::Normal => {
                        let v = data
                            .normals
                            .get(vertex)
                            .ok_or_else(|| error("output layout requires missing normals"))?;
                        [v[0], v[1], v[2], 0.]
                    }
                    ElementSemantic::Tangent => data
                        .tangents
                        .get(vertex)
                        .ok_or_else(|| error("output layout requires missing tangents"))?,
                    ElementSemantic::Color => data
                        .colors
                        .get(vertex)
                        .ok_or_else(|| error("output layout requires missing colors"))?,
                    ElementSemantic::TexCoord(slot) => {
                        let v = data
                            .texture_coordinates
                            .get(slot as usize)
                            .and_then(|v| v.get(vertex))
                            .ok_or_else(|| {
                                error("output layout requires missing texture coordinates")
                            })?;
                        [v[0], v[1], 0., 0.]
                    }
                    ElementSemantic::JointIndices | ElementSemantic::JointWeights => {
                        let row =
                            data.influences
                                .rows()
                                .joint_weights(vertex)
                                .ok_or_else(|| {
                                    error("output layout requires missing skin distributions")
                                })?;
                        if row.len() != 4 {
                            return Err(error(
                                "declared output skin encoding requires four ordered influences",
                            ));
                        }
                        let mut out = [0.; 4];
                        for (slot, influence) in row.iter().enumerate() {
                            let node = palette
                                .get(influence.joint as usize)
                                .ok_or_else(|| error("skin joint exceeds placement palette"))?;
                            let local = segment.bones.iter().position(|id| id == node).ok_or_else(
                                || error("skin joint is not selected by its segment palette"),
                            )?;
                            out[slot] = if element.semantic == ElementSemantic::JointIndices {
                                local as f32
                            } else {
                                influence.weight
                            };
                        }
                        out
                    }
                };
                vertices::write(&mut bytes, vertex, layout.stride, element, value, context)?;
            }
        }
        let index_offset = indices
            .len()
            .checked_mul(2)
            .ok_or_else(|| error("index offset overflows"))?;
        for triangle in first_triangle..first_triangle + triangles_len {
            for index in triangles.get(triangle).expect("checked range") {
                if index < segment.first_vertex
                    || index >= segment.first_vertex + segment.vertex_count
                {
                    return Err(error("triangle crosses its segment's vertex window"));
                }
                indices.push(
                    u16::try_from(index - segment.first_vertex)
                        .map_err(|_| error("segment vertex index exceeds u16"))?,
                );
            }
        }
        native_segments.push(native::SubObjectSegment {
            bone_ids: ZeroVec::alloc_from_slice(&native_bones),
            vert_count: i32::try_from(len)
                .map_err(|_| error("vertex count exceeds native width"))?,
            vert_offset: i32::try_from(
                first
                    .checked_mul(layout.stride as usize)
                    .ok_or_else(|| error("vertex byte offset overflows"))?,
            )
            .map_err(|_| error("vertex byte offset exceeds native width"))?,
            index_count: i32::try_from(
                triangles_len
                    .checked_mul(3)
                    .ok_or_else(|| error("index count overflows"))?,
            )
            .map_err(|_| error("index count exceeds native width"))?,
            index_offset: i32::try_from(index_offset)
                .map_err(|_| error("index offset exceeds native width"))?,
        });
    }
    if next_vertex != count_vertices || next_triangle != triangles.len() {
        return Err(error("semantic segments do not cover all geometry"));
    }
    Ok(EncodedGeometry {
        vertex_buffer: layout.buffer(&bytes)?,
        index_buffer: native::GeoChunkIndexBuffer {
            pdw_chunk_indices: ZeroVec::alloc_from_slice(&indices),
            ibid: -1,
            ..Default::default()
        },
        segments: native_segments,
    })
}
fn error(message: &str) -> Error {
    Error::new("d4.geometry", message)
}
