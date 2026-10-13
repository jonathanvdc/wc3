//! Native face and draw-batch selection into common geometry.
use super::*;
pub(in crate::model::conversion::wmo) fn import<D: Dialect>(
    group: &Group<'_, D>,
    key: GroupKey,
    names: &[u8],
    model: &mut s::Model<'static>,
    sources: &mut Vec<MeshSource>,
    context: &mut Context,
) -> Result<(), Error> {
    let path = format!("groups[{}].lod[{}]", key.index, key.lod);
    let positions = group
        .chunks()
        .iter()
        .filter_map(|chunk| {
            if let GroupChunk::Positions(rows) = chunk {
                Some(collect(rows, &path))
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .concat();
    let normals = group
        .chunks()
        .iter()
        .find_map(|chunk| {
            if let GroupChunk::Normals(rows) = chunk {
                Some(collect(rows, &path))
            } else {
                None
            }
        })
        .transpose()?
        .unwrap_or_default();
    let mut indices = Vec::new();
    for chunk in group.chunks() {
        match chunk {
            GroupChunk::Indices(rows) => {
                indices.extend(collect(rows, &path)?.into_iter().map(u32::from))
            }
            GroupChunk::Indices32(rows) => indices.extend(collect(rows, &path)?),
            _ => {}
        }
    }
    if !indices.len().is_multiple_of(3) {
        return Err(Error::new(
            &path,
            "geometry index count is not divisible by three",
        ));
    }
    let mut triangles = Vec::new();
    for chunk in group.chunks() {
        match chunk {
            GroupChunk::Triangles(rows) => triangles.extend(
                collect(rows, &path)?
                    .into_iter()
                    .map(|row| (u32::from(row.flags), u32::from(row.material))),
            ),
            GroupChunk::Triangles2(rows) => triangles.extend(
                collect(rows, &path)?
                    .into_iter()
                    .map(|row| (u32::from(row.flags), u32::from(row.material))),
            ),
            _ => {}
        }
    }
    if triangles.is_empty() {
        triangles = vec![(0, 0); indices.len() / 3];
    }
    if triangles.len() != indices.len() / 3 {
        return Err(Error::new(
            &path,
            "face flags and geometry triangle counts differ",
        ));
    }
    let source_materials = triangles.iter().map(|row| row.1).collect::<Vec<_>>();
    let mut classes = vec![0u32; triangles.len()];
    let mut batch_index = 0usize;
    for chunk in group.chunks() {
        if let GroupChunk::Batches(rows) = chunk {
            for batch in collect(rows, &path)? {
                let start = batch.start_index as usize;
                let end = start
                    .checked_add(usize::from(batch.index_count))
                    .ok_or_else(|| Error::new(&path, "batch range overflow"))?;
                if !start.is_multiple_of(3) || !end.is_multiple_of(3) || end > indices.len() {
                    return Err(Error::new(
                        &path,
                        "batch does not address complete geometry triangles",
                    ));
                }
                let material = u32::from(batch.prefix.material(batch.flags, batch.material));
                let class = if batch_index < usize::from(group.header().transition_batch_count) {
                    1
                } else if batch_index
                    < usize::from(group.header().transition_batch_count)
                        + usize::from(group.header().interior_batch_count)
                {
                    2
                } else {
                    3
                };
                for face in start / 3..end / 3 {
                    triangles[face].1 = material;
                    classes[face] = class;
                }
                batch_index += 1;
            }
        }
    }
    let uvs = group
        .chunks()
        .iter()
        .filter_map(|chunk| {
            if let GroupChunk::TextureCoordinates(rows) = chunk {
                Some(collect(rows, &path).map(Cow::Owned))
            } else {
                None
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let colors = group
        .chunks()
        .iter()
        .find_map(|chunk| {
            if let GroupChunk::VertexColors(rows) | GroupChunk::VertexColors2(rows) = chunk {
                Some(collect(rows, &path))
            } else {
                None
            }
        })
        .transpose()?
        .unwrap_or_default()
        .into_iter()
        .map(|value| [value[2], value[1], value[0], value[3]].map(|v| f32::from(v) / 255.))
        .collect::<Vec<_>>();
    let name = if names.is_empty() {
        format!("Group {}", key.index)
    } else {
        text(names, group.header().name_offset, &path)?
    };
    let mut sets = BTreeMap::<(u32, u32), Vec<usize>>::new();
    for (face, (_, material)) in triangles.iter().enumerate() {
        sets.entry((classes[face], *material))
            .or_default()
            .push(face);
    }
    if sets.is_empty() {
        sets.insert((0, u32::MAX), Vec::new());
    }
    for ((class, material), faces) in sets {
        let slot = if material == 255 || material == 65535 {
            u32::MAX
        } else {
            material
        };
        let mut fields = vec![
            ("wmo.batch_class".into(), i64::from(class), String::new()),
            ("wmo.group_index".into(), key.index as i64, String::new()),
            (
                "wmo.group.flags".into(),
                i64::from(group.header().flags),
                String::new(),
            ),
            (
                "wmo.group.flags2".into(),
                i64::from(group.header().flags2),
                String::new(),
            ),
            (
                "wmo.group.id".into(),
                i64::from(group.header().unique_id),
                String::new(),
            ),
            (
                "wmo.group.liquid_type".into(),
                i64::from(group.header().liquid_type),
                String::new(),
            ),
            (
                "wmo.group.fog_indices".into(),
                i64::from(u32::from_le_bytes(group.header().fog_indices)),
                String::new(),
            ),
        ];
        use native::Write as _;
        let mut tail = native::codec::Encoder::new();
        group
            .header()
            .tail
            .write(&mut tail)
            .map_err(|e| failure("group.header_tail", e))?;
        let tail: [u8; 4] = tail
            .finish()
            .try_into()
            .map_err(|_| Error::new("group.header_tail", "unexpected native tail width"))?;
        fields.push((
            "wmo.group.header_tail".into(),
            i64::from(u32::from_le_bytes(tail)),
            String::new(),
        ));
        if group.header().descriptive_name_offset != 0 {
            fields.push((
                "wmo.group.description".into(),
                0,
                text(names, group.header().descriptive_name_offset, &path)?,
            ));
        }
        for chunk in group.chunks() {
            match chunk {
                GroupChunk::LightReferences(rows) => fields.push((
                    "wmo.group.light_references".into(),
                    0,
                    metadata::words(collect(rows, &path)?.into_iter().map(u32::from)),
                )),
                GroupChunk::DoodadReferences(rows) => fields.push((
                    "wmo.group.doodad_references".into(),
                    0,
                    metadata::words(collect(rows, &path)?.into_iter().map(u32::from)),
                )),
                _ => {}
            }
        }
        fields.extend(liquids::fields(group)?);
        let triangle_flags = faces.iter().map(|face| triangles[*face].0).collect();
        model.meshes.push(s::Mesh {
            name: Cow::Owned(name.clone()),
            positions: Cow::Owned(positions.clone()),
            normals: Cow::Owned(normals.clone()),
            uv_sets: uvs.clone(),
            colors: Cow::Owned(colors.clone()),
            triangles: Cow::Owned(
                faces
                    .iter()
                    .map(|face| {
                        [
                            indices[face * 3],
                            indices[face * 3 + 1],
                            indices[face * 3 + 2],
                        ]
                    })
                    .collect(),
            ),
            material_slot: slot,
            lod: key.lod as u32,
            bounds: bounds([group.header().bounds_minimum, group.header().bounds_maximum]),
            ..Default::default()
        });
        sources.push(MeshSource {
            group: key,
            triangle_flags,
            triangle_materials: faces.iter().map(|face| source_materials[*face]).collect(),
            group_fields: fields,
        });
    }
    for chunk in group.chunks() {
        match chunk {
            GroupChunk::Positions(_)
            | GroupChunk::Normals(_)
            | GroupChunk::Indices(_)
            | GroupChunk::Indices32(_)
            | GroupChunk::Triangles(_)
            | GroupChunk::Triangles2(_)
            | GroupChunk::TextureCoordinates(_)
            | GroupChunk::VertexColors(_)
            | GroupChunk::VertexColors2(_)
            | GroupChunk::Batches(_)
            | GroupChunk::BspNodes(_)
            | GroupChunk::BspFaces(_)
            | GroupChunk::Liquid(_)
            | GroupChunk::LightReferences(_)
            | GroupChunk::DoodadReferences(_)
            | GroupChunk::PointLights(_) => {}
            GroupChunk::Unknown { .. } => {}
            _ => context.unsupported(
                format!("{path}.{}", String::from_utf8_lossy(&chunk.tag())),
                "Native group feature has no editable WEM mapping",
            )?,
        }
    }
    Ok(())
}
