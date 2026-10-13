//! Historical destination records rebuilt from the active WEM graph.
use super::*;
fn group_to16(source: &Group<'_, native::Classic>) -> Result<old::Group16<'static>, Error> {
    let h = source.header();
    let mut group = old::Group16::new();
    group.header = old::GroupHeader16 {
        name_offset: h.name_offset,
        descriptive_name_offset: h.descriptive_name_offset,
        flags: h.flags,
        bounds_minimum: h.bounds_minimum,
        bounds_maximum: h.bounds_maximum,
        portal_start: i16::try_from(h.portal_start)
            .map_err(|error| failure("group.portal_start", error))?,
        portal_count: h.portal_count,
        transition_batch_count: h.transition_batch_count,
        interior_batch_count: h.interior_batch_count,
        exterior_batch_count: h.exterior_batch_count,
        reserved_batch_count: h.reserved_batch_count,
        fog_indices: h.fog_indices,
        liquid_type: h.liquid_type,
        unique_id: h.unique_id as i32,
        flags2: h.flags2,
        reserved_tail: h.tail,
    };
    for chunk in source.chunks() {
        match chunk {
            GroupChunk::Triangles(rows) => {
                let rows = collect(rows, "triangles")?
                    .into_iter()
                    .map(|row| old::LegacyTriangle {
                        flags: row.flags,
                        material: row.material,
                        lightmap: 0,
                        reserved: 0,
                    })
                    .collect::<Vec<_>>();
                group
                    .chunks
                    .push(old::LegacyGroupChunk::Triangles(array(&rows)?));
            }
            GroupChunk::Triangles2(_) => {
                return Err(Error::new(
                    "triangles",
                    "version 16 cannot encode extended triangle materials or flags",
                ));
            }
            GroupChunk::Batches(rows) => {
                let rows = collect(rows, "batches")?
                    .into_iter()
                    .map(|row| old::Batch16 {
                        bounds: row.prefix,
                        start_index: row.start_index,
                        index_count: row.index_count,
                        minimum_vertex: row.minimum_vertex,
                        maximum_vertex: row.maximum_vertex,
                        flags: row.flags,
                        material: row.material,
                        extension: [0; 8],
                    })
                    .collect::<Vec<_>>();
                group
                    .chunks
                    .push(old::LegacyGroupChunk::Batches(array(&rows)?));
            }
            GroupChunk::Liquid(value) => {
                group
                    .chunks
                    .push(old::LegacyGroupChunk::Liquid(old::LegacyLiquid {
                        header: value.header,
                        vertices: Array::decode(value.vertices.as_bytes())
                            .map_err(|error| failure("liquid", error))?
                            .into_owned(),
                        tiles: value.tiles.clone().into_owned(),
                    }));
            }
            _ => group
                .chunks
                .push(old::LegacyGroupChunk::Common(chunk.clone().into_owned())),
        }
    }
    Ok(group)
}
fn group_to14(source: &Group<'_, native::Classic>) -> Result<old::Group14<'static>, Error> {
    let h = source.header();
    let mut group = old::Group14::new();
    group.header = old::GroupHeader14 {
        name_offset: h.name_offset,
        descriptive_name_offset: h.descriptive_name_offset,
        flags: h.flags,
        bounds_minimum: h.bounds_minimum,
        bounds_maximum: h.bounds_maximum,
        portal_start: i32::from(h.portal_start),
        portal_count: u32::from(h.portal_count),
        fog_indices: h.fog_indices,
        liquid_type: h.liquid_type,
        unique_id: h.unique_id as i32,
        ..Default::default()
    };
    let mut vertex_count = 0usize;
    let mut batch_count = 0usize;
    for chunk in source.chunks() {
        match chunk {
            GroupChunk::Triangles(rows) => {
                let rows = collect(rows, "triangles")?
                    .into_iter()
                    .map(|row| old::LegacyTriangle {
                        flags: row.flags,
                        material: row.material,
                        lightmap: 0,
                        reserved: 0,
                    })
                    .collect::<Vec<_>>();
                group
                    .chunks
                    .push(old::LegacyGroupChunk::Triangles(array(&rows)?));
            }
            GroupChunk::Triangles2(_) => {
                return Err(Error::new(
                    "triangles",
                    "version 14 cannot encode extended triangle materials or flags",
                ));
            }
            GroupChunk::Batches(rows) => {
                let rows = collect(rows, "batches")?
                    .into_iter()
                    .map(|row| {
                        Ok(old::Batch14 {
                            lightmap: 0,
                            material: row.material,
                            bounds: row.prefix,
                            start_index: u16::try_from(row.start_index)
                                .map_err(|error| failure("batch.start", error))?,
                            index_count: row.index_count,
                            minimum_vertex: row.minimum_vertex,
                            maximum_vertex: row.maximum_vertex,
                            flags: row.flags,
                            reserved: 0,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                batch_count += rows.len();
                group
                    .chunks
                    .push(old::LegacyGroupChunk::Batches(array(&rows)?));
            }
            GroupChunk::Liquid(value) => {
                group
                    .chunks
                    .push(old::LegacyGroupChunk::Liquid(old::LegacyLiquid {
                        header: value.header,
                        vertices: Array::decode(value.vertices.as_bytes())
                            .map_err(|error| failure("liquid", error))?
                            .into_owned(),
                        tiles: value.tiles.clone().into_owned(),
                    }));
            }
            GroupChunk::Positions(rows) => {
                vertex_count += rows.len();
                group
                    .chunks
                    .push(old::LegacyGroupChunk::Common(chunk.clone().into_owned()));
            }
            _ => group
                .chunks
                .push(old::LegacyGroupChunk::Common(chunk.clone().into_owned())),
        }
    }
    if h.transition_batch_count != 0 {
        return Err(Error::new(
            "batch.class",
            "alpha WMO has no transition-batch classification",
        ));
    }
    let vertices =
        u16::try_from(vertex_count).map_err(|error| failure("group.vertex_count", error))?;
    if usize::from(h.interior_batch_count) + usize::from(h.exterior_batch_count) != batch_count {
        return Err(Error::new(
            "batch.class",
            "native batch counts do not cover alpha output table",
        ));
    }
    group.header.interior_batches[0] = old::HeaderBatchRange {
        vertex_start: 0,
        vertex_count: vertices,
        batch_start: 0,
        batch_count: h.interior_batch_count,
    };
    group.header.exterior_batches[0] = old::HeaderBatchRange {
        vertex_start: 0,
        vertex_count: vertices,
        batch_start: h.interior_batch_count,
        batch_count: h.exterior_batch_count,
    };
    Ok(group)
}
impl Destination for V16 {
    type Output = old::Bundle16<'static>;
    fn plan(asset: &Asset, options: &ExportOptions) -> Result<Plan<Self::Output>, Error> {
        let planned = export::modern::<native::Classic>(asset, options)?.execute()?;
        let mut source = planned.value;
        if !source
            .root
            .chunks()
            .any(|chunk| matches!(chunk, RootChunk::TextureNames(_)))
        {
            for chunk in source.root.chunks() {
                if let RootChunk::Materials(rows) = chunk {
                    if collect(rows, "materials")?
                        .iter()
                        .any(|material| material.textures().iter().any(|value| *value != 0))
                    {
                        return Err(Error::new(
                            "textures",
                            "historical WMO output requires texture paths; resolve file IDs before exporting",
                        ));
                    }
                }
            }
            root_chunk(&mut source.root, RootChunk::TextureNames(array(&[0u8])?))?;
        }
        if source.root.header().lod_count() != 0 {
            return Err(Error::new(
                "lod",
                "version 16 cannot encode modern LOD companions",
            ));
        }
        let mut root = old::Root16::new();
        *root.header_mut() = source.root.header().clone();
        for chunk in source.root.chunks() {
            root.push_chunk(chunk.clone().into_owned())
                .map_err(|error| failure("root16", error))?;
        }
        let mut bundle = old::Bundle16::from_root(root);
        for index in 0..source.root.header().group_count as usize {
            let group = source
                .group(GroupKey { index, lod: 0 })
                .ok_or_else(|| Error::new("groups", "missing group"))?;
            bundle.insert_group(
                index,
                FileKey::Sibling {
                    owner: Box::new(FileKey::Root),
                    suffix: format!("_{index:03}.wmo"),
                },
                group_to16(group)?,
            );
        }
        bundle
            .validate_complete()
            .map_err(|error| failure("wmo16.output", error))?;
        Ok(Plan::new(bundle, planned.report))
    }
}
impl Destination for V14 {
    type Output = old::Root14<'static>;
    fn plan(asset: &Asset, options: &ExportOptions) -> Result<Plan<Self::Output>, Error> {
        let mut context = Context::export(options);
        let planned = export::modern::<native::Classic>(asset, options)?.execute()?;
        let fields = profile_fields(asset, options)?;
        let mut source = planned.value;
        if !source
            .root
            .chunks()
            .any(|chunk| matches!(chunk, RootChunk::TextureNames(_)))
        {
            for chunk in source.root.chunks() {
                if let RootChunk::Materials(rows) = chunk {
                    if collect(rows, "materials")?
                        .iter()
                        .any(|material| material.textures().iter().any(|value| *value != 0))
                    {
                        return Err(Error::new(
                            "textures",
                            "historical WMO output requires texture paths; resolve file IDs before exporting",
                        ));
                    }
                }
            }
            root_chunk(&mut source.root, RootChunk::TextureNames(array(&[0u8])?))?;
        }
        if source.root.header().lod_count() != 0 {
            return Err(Error::new(
                "lod",
                "version 14 cannot encode modern LOD companions",
            ));
        }
        let mut root = old::Root14::new();
        root.header_mut().wmo_id = source.root.header().wmo_id;
        root.header_mut().ambient_color = source.root.header().ambient_color;
        if let Some((_, value)) = fields.get("wmo.legacy14.header_reserved") {
            let bytes = metadata::parse_words(value)?
                .into_iter()
                .map(|value| u8::try_from(value).map_err(|error| failure("header.reserved", error)))
                .collect::<Result<Vec<_>, _>>()?;
            root.header_mut().reserved = bytes.as_slice().try_into().map_err(|_| {
                Error::new(
                    "header.reserved",
                    "alpha header requires twenty-eight reserved bytes",
                )
            })?;
        }
        for chunk in source.root.chunks() {
            match chunk {
                RootChunk::Materials(rows) => {
                    let rows = collect(rows, "materials")?;
                    let mut values = Vec::new();
                    for (index, row) in rows.into_iter().enumerate() {
                        if row.shader != 0 || row.texture2 != 0 {
                            context.unsupported(
                                format!("materials[{index}].shader"),
                                "Alpha materials have no shader-table index or third texture slot",
                            )?;
                        }
                        let mut reserved = [0; 8];
                        if let Some((_, value)) =
                            fields.get(&format!("wmo.material.{index}.legacy_reserved"))
                        {
                            let values = metadata::parse_words(value)?
                                .into_iter()
                                .map(|value| {
                                    u8::try_from(value)
                                        .map_err(|error| failure("material.reserved", error))
                                })
                                .collect::<Result<Vec<_>, _>>()?;
                            reserved = values.as_slice().try_into().map_err(|_| {
                                Error::new(
                                    "material.reserved",
                                    "alpha material requires eight reserved bytes",
                                )
                            })?;
                        }
                        values.push(old::Material14 {
                            version: metadata::number(
                                &fields,
                                &format!("wmo.material.{index}.legacy_version"),
                                0,
                            )?,
                            flags: row.flags,
                            blend_mode: row.blend_mode,
                            texture0: row.texture0,
                            sidn_color: row.sidn_color,
                            frame_sidn_color: row.frame_sidn_color,
                            texture1: row.texture1,
                            diffuse_color: row.diffuse_color,
                            ground_type: row.ground_type,
                            reserved,
                        });
                    }
                    root.push_chunk(old::RootChunk14::Materials(array(&values)?))
                        .map_err(|error| failure("materials14", error))?;
                }
                RootChunk::Groups(_) => {}
                RootChunk::Lights(rows) => {
                    let rows = collect(rows, "lights")?;
                    let mut values = Vec::new();
                    for (index, row) in rows.into_iter().enumerate() {
                        if row.rotation != [0., 0., 0., 1.] {
                            context.unsupported(
                                format!("lights[{index}].rotation"),
                                "Alpha lights do not store orientation quaternions",
                            )?;
                        }
                        values.push(old::Light14 {
                            kind: row.kind,
                            use_attenuation: row.use_attenuation,
                            reserved: row.padding,
                            color: row.color,
                            position: row.position,
                            intensity: row.intensity,
                            attenuation_start: row.attenuation_start,
                            attenuation_end: row.attenuation_end,
                        });
                    }
                    root.push_chunk(old::RootChunk14::Lights(array(&values)?))
                        .map_err(|error| failure("lights14", error))?;
                }
                _ => root
                    .push_chunk(old::RootChunk14::Common(chunk.clone().into_owned()))
                    .map_err(|error| failure("root14", error))?,
            }
        }
        for index in 0..source.root.header().group_count as usize {
            let group = source
                .group(GroupKey { index, lod: 0 })
                .ok_or_else(|| Error::new("groups", "missing group"))?;
            let info = old::GroupInfo14 {
                flags: group.header().flags,
                bounds: [group.header().bounds_minimum, group.header().bounds_maximum],
                name_offset: group.header().name_offset as i32,
                ..Default::default()
            };
            root.push_group(group_to14(group)?, info)
                .map_err(|error| failure("group14", error))?;
        }
        root.synchronize_counts()
            .map_err(|error| failure("root14.counts", error))?;
        root.validate()
            .map_err(|error| failure("wmo14.output", error))?;
        let mut report = planned.report;
        report.issues.extend(context.report.issues);
        Ok(Plan::new(root, report))
    }
}
