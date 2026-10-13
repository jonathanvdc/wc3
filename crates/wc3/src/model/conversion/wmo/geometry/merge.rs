//! Draw-section grouping and native companion reassembly.
use super::*;
use std::iter::repeat_n;
struct Streams<D: Dialect> {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    faces: Vec<(u32, u32)>,
    uvs: Vec<Vec<[f32; 2]>>,
    colors: Vec<[u8; 4]>,
    batches: Vec<nr::Batch<D>>,
}
fn rebuild<D: Dialect>(group: &mut Group<'static, D>, streams: Streams<D>) -> Result<(), Error> {
    let Streams {
        positions,
        normals,
        indices,
        faces,
        uvs,
        colors,
        batches,
    } = streams;
    group.chunks_mut().retain(|chunk| {
        !matches!(
            chunk,
            GroupChunk::Positions(_)
                | GroupChunk::Normals(_)
                | GroupChunk::Indices(_)
                | GroupChunk::Indices32(_)
                | GroupChunk::Triangles(_)
                | GroupChunk::Triangles2(_)
                | GroupChunk::TextureCoordinates(_)
                | GroupChunk::VertexColors(_)
                | GroupChunk::Batches(_)
        )
    });
    group
        .chunks_mut()
        .push(GroupChunk::Positions(array(&positions)?));
    if !normals.is_empty() {
        group
            .chunks_mut()
            .push(GroupChunk::Normals(array(&normals)?));
    }
    if indices.iter().any(|value| *value > 65535) {
        group
            .chunks_mut()
            .push(GroupChunk::Indices32(array(&indices)?));
    } else {
        let values = indices
            .iter()
            .map(|value| *value as u16)
            .collect::<Vec<_>>();
        group
            .chunks_mut()
            .push(GroupChunk::Indices(array(&values)?));
    }
    if faces
        .iter()
        .any(|(flags, material)| *flags > 255 || *material > 255)
    {
        let rows = faces
            .iter()
            .map(|(flags, material)| nr::Triangle2 {
                flags: nr::TriangleFlags2(*flags as u16),
                material: *material as u16,
            })
            .collect::<Vec<_>>();
        group
            .chunks_mut()
            .push(GroupChunk::Triangles2(array(&rows)?));
    } else {
        let rows = faces
            .iter()
            .map(|(flags, material)| nr::Triangle {
                flags: nr::TriangleFlags(*flags as u8),
                material: *material as u8,
            })
            .collect::<Vec<_>>();
        group
            .chunks_mut()
            .push(GroupChunk::Triangles(array(&rows)?));
    }
    for uv in uvs {
        group
            .chunks_mut()
            .push(GroupChunk::TextureCoordinates(array(&uv)?));
    }
    if !colors.is_empty() {
        group
            .chunks_mut()
            .push(GroupChunk::VertexColors(array(&colors)?));
    }
    if !batches.is_empty() {
        group
            .chunks_mut()
            .push(GroupChunk::Batches(array(&batches)?));
    }
    Ok(())
}
pub(in crate::model::conversion::wmo) fn merge<D: Dialect>(
    target: &mut Group<'static, D>,
    source: Group<'static, D>,
) -> Result<(), Error> {
    fn values<
        'a,
        'data,
        D: Dialect,
        T: for<'input> native::Read<'input> + native::RecordLayout + 'a,
    >(
        group: &'a Group<'data, D>,
        select: impl Fn(&'a GroupChunk<'data, D>) -> Option<&'a Array<'data, T>>,
    ) -> Result<Vec<T>, Error> {
        group
            .chunks()
            .iter()
            .filter_map(select)
            .map(|array| collect(array, "group.merge"))
            .collect::<Result<Vec<_>, _>>()
            .map(|rows| rows.into_iter().flatten().collect())
    }
    let mut positions = values(target, |chunk| {
        if let GroupChunk::Positions(rows) = chunk {
            Some(rows)
        } else {
            None
        }
    })?;
    let offset =
        u32::try_from(positions.len()).map_err(|error| failure("group.vertices", error))?;
    positions.extend(values(&source, |chunk| {
        if let GroupChunk::Positions(rows) = chunk {
            Some(rows)
        } else {
            None
        }
    })?);
    let mut normals = values(target, |chunk| {
        if let GroupChunk::Normals(rows) = chunk {
            Some(rows)
        } else {
            None
        }
    })?;
    let next_normals = values(&source, |chunk| {
        if let GroupChunk::Normals(rows) = chunk {
            Some(rows)
        } else {
            None
        }
    })?;
    if normals.is_empty() && !next_normals.is_empty() {
        normals.resize(offset as usize, [0.; 3]);
    }
    if !normals.is_empty() && next_normals.is_empty() {
        normals.resize(positions.len(), [0.; 3]);
    } else {
        normals.extend(next_normals);
    }
    let get_indices = |group: &Group<'_, D>| -> Result<Vec<u32>, Error> {
        let mut output = Vec::new();
        for chunk in group.chunks() {
            match chunk {
                GroupChunk::Indices(rows) => {
                    output.extend(collect(rows, "indices")?.into_iter().map(u32::from))
                }
                GroupChunk::Indices32(rows) => output.extend(collect(rows, "indices")?),
                _ => {}
            }
        }
        Ok(output)
    };
    let mut indices = get_indices(target)?;
    let first_index = indices.len() as u32;
    for value in get_indices(&source)? {
        indices.push(
            value
                .checked_add(offset)
                .ok_or_else(|| Error::new("group.indices", "vertex offset overflow"))?,
        );
    }
    let get_faces = |group: &Group<'_, D>| -> Result<Vec<(u32, u32)>, Error> {
        let mut output = Vec::new();
        for chunk in group.chunks() {
            match chunk {
                GroupChunk::Triangles(rows) => output.extend(
                    collect(rows, "triangles")?
                        .into_iter()
                        .map(|row| (u32::from(row.flags), row.material as u32)),
                ),
                GroupChunk::Triangles2(rows) => output.extend(
                    collect(rows, "triangles")?
                        .into_iter()
                        .map(|row| (u32::from(row.flags), row.material as u32)),
                ),
                _ => {}
            }
        }
        Ok(output)
    };
    let mut faces = get_faces(target)?;
    faces.extend(get_faces(&source)?);
    let get_uvs = |group: &Group<'_, D>| -> Result<Vec<Vec<[f32; 2]>>, Error> {
        group
            .chunks()
            .iter()
            .filter_map(|chunk| {
                if let GroupChunk::TextureCoordinates(rows) = chunk {
                    Some(collect(rows, "uvs"))
                } else {
                    None
                }
            })
            .collect()
    };
    let mut uvs = get_uvs(target)?;
    let next_uvs = get_uvs(&source)?;
    let channels = uvs.len().max(next_uvs.len());
    uvs.resize(channels, vec![[0.; 2]; offset as usize]);
    for (index, uv) in uvs.iter_mut().enumerate() {
        if let Some(next) = next_uvs.get(index) {
            uv.extend(next);
        } else {
            uv.resize(positions.len(), [0.; 2]);
        }
    }
    let mut colors = values(target, |chunk| {
        if let GroupChunk::VertexColors(rows) = chunk {
            Some(rows)
        } else {
            None
        }
    })?;
    let next = values(&source, |chunk| {
        if let GroupChunk::VertexColors(rows) = chunk {
            Some(rows)
        } else {
            None
        }
    })?;
    if colors.is_empty() && !next.is_empty() {
        colors.resize(offset as usize, [255; 4]);
    }
    if !colors.is_empty() && next.is_empty() {
        colors.resize(positions.len(), [255; 4]);
    } else {
        colors.extend(next);
    }
    let classes = |group: &Group<'_, D>| {
        [
            group.header().transition_batch_count,
            group.header().interior_batch_count,
            group.header().exterior_batch_count,
        ]
        .into_iter()
        .enumerate()
        .flat_map(|(class, count)| repeat_n(class, usize::from(count)))
        .collect::<Vec<_>>()
    };
    let mut order = classes(target);
    order.extend(classes(&source));
    let mut batches = values(target, |chunk| {
        if let GroupChunk::Batches(rows) = chunk {
            Some(rows)
        } else {
            None
        }
    })?;
    for mut batch in values(&source, |chunk| {
        if let GroupChunk::Batches(rows) = chunk {
            Some(rows)
        } else {
            None
        }
    })? {
        batch.start_index = batch
            .start_index
            .checked_add(first_index)
            .ok_or_else(|| Error::new("batch.start", "index offset overflow"))?;
        let (first, last) = batch.vertex_range();
        let first = first
            .checked_add(offset)
            .ok_or_else(|| Error::new("batch.range", "vertex offset overflow"))?;
        let last = last
            .checked_add(offset)
            .ok_or_else(|| Error::new("batch.range", "vertex offset overflow"))?;
        if last > 65535 || batch.flags.extended_vertices() {
            let mut probe = [0u8; 12];
            probe[0] = 1;
            if Decoder::new(&probe)
                .read::<D::BatchPrefix>()
                .map_err(|e| failure("batch.range", e))?
                .vertex_range(nr::BatchFlags(4), 0, 0)
                .0
                != 65536
            {
                return Err(Error::new(
                    "batch.range",
                    "selected WMO batch layout cannot address merged vertex ranges",
                ));
            }
            use native::Write as _;
            let mut encoder = native::codec::Encoder::new();
            batch
                .prefix
                .write(&mut encoder)
                .map_err(|e| failure("batch.range", e))?;
            let mut bytes = encoder.finish();
            bytes[0] = u8::try_from(first >> 16).map_err(|e| failure("batch.range", e))?;
            bytes[1] = u8::try_from(last >> 16).map_err(|e| failure("batch.range", e))?;
            batch.prefix = Decoder::new(&bytes)
                .read()
                .map_err(|e| failure("batch.range", e))?;
            batch.flags.set_extended_vertices(true);
        }
        batch.minimum_vertex = first as u16;
        batch.maximum_vertex = last as u16;
        batches.push(batch);
    }
    if batches.len() != order.len() {
        return Err(Error::new(
            "batch.class",
            "batch class counts do not cover the draw table",
        ));
    }
    let mut ordered = order.into_iter().zip(batches).collect::<Vec<_>>();
    ordered.sort_by_key(|(class, _)| *class);
    let batches = ordered
        .into_iter()
        .map(|(_, batch)| batch)
        .collect::<Vec<_>>();
    for i in 0..3 {
        target.header_mut().bounds_minimum[i] =
            target.header().bounds_minimum[i].min(source.header().bounds_minimum[i]);
        target.header_mut().bounds_maximum[i] =
            target.header().bounds_maximum[i].max(source.header().bounds_maximum[i]);
    }
    for class in 0..3 {
        let counts = match class {
            0 => (
                target.header().transition_batch_count,
                source.header().transition_batch_count,
            ),
            1 => (
                target.header().interior_batch_count,
                source.header().interior_batch_count,
            ),
            _ => (
                target.header().exterior_batch_count,
                source.header().exterior_batch_count,
            ),
        };
        let count = counts
            .0
            .checked_add(counts.1)
            .ok_or_else(|| Error::new("batch.count", "batch count overflow"))?;
        match class {
            0 => target.header_mut().transition_batch_count = count,
            1 => target.header_mut().interior_batch_count = count,
            _ => target.header_mut().exterior_batch_count = count,
        }
    }
    rebuild(
        target,
        Streams {
            positions,
            normals,
            indices,
            faces,
            uvs,
            colors,
            batches,
        },
    )
}
