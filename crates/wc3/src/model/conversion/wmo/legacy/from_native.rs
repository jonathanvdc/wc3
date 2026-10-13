//! Historical source records decoded into common/native WEM values.
use super::*;
fn group14(
    source: &old::Group14<'_>,
    context: &mut Context,
) -> Result<Group<'static, native::Classic>, Error> {
    let h = &source.header;
    let mut group = Group::<native::Classic>::new();
    *group.header_mut() = nr::GroupHeader {
        name_offset: h.name_offset,
        descriptive_name_offset: h.descriptive_name_offset,
        flags: h.flags,
        bounds_minimum: h.bounds_minimum,
        bounds_maximum: h.bounds_maximum,
        portal_start: u16::try_from(h.portal_start.max(0))
            .map_err(|error| failure("group.portal_start", error))?,
        portal_count: u16::try_from(h.portal_count)
            .map_err(|error| failure("group.portal_count", error))?,
        fog_indices: h.fog_indices,
        liquid_type: h.liquid_type,
        unique_id: h.unique_id as u32,
        ..Default::default()
    };
    group.header_mut().interior_batch_count = h
        .interior_batches
        .iter()
        .try_fold(0u16, |sum, row| sum.checked_add(row.batch_count))
        .ok_or_else(|| Error::new("batch.count", "interior batch count overflow"))?;
    group.header_mut().exterior_batch_count = h
        .exterior_batches
        .iter()
        .try_fold(0u16, |sum, row| sum.checked_add(row.batch_count))
        .ok_or_else(|| Error::new("batch.count", "exterior batch count overflow"))?;
    if h.reserved_tail != [0; 8] {
        context.unsupported(
            "group14.reserved_tail",
            "undocumented alpha group control bytes require a native field mapping",
        )?;
    }
    for chunk in &source.chunks {
        match chunk {
            old::LegacyGroupChunk::Common(chunk) => {
                group.chunks_mut().push(chunk.clone().into_owned())
            }
            old::LegacyGroupChunk::Triangles(rows) => {
                let rows = collect(rows, "legacy.triangles")?;
                if rows.iter().any(|row| row.reserved != 0) {
                    context.unsupported(
                        "legacy.reserved",
                        "undocumented legacy alignment bytes need a native field mapping",
                    )?;
                }
                if rows.iter().any(|row| row.lightmap != 0) {
                    context.unsupported("legacy.triangles.lightmap","Legacy lightmap selectors need an explicit lightmap resource representation")?;
                }
                group.chunks_mut().push(GroupChunk::Triangles(array(
                    &rows
                        .iter()
                        .map(|row| nr::Triangle {
                            flags: row.flags,
                            material: row.material,
                        })
                        .collect::<Vec<_>>(),
                )?));
            }
            old::LegacyGroupChunk::Batches(rows) => {
                let rows = collect(rows, "legacy.batches")?;
                if rows.iter().any(|row| row.lightmap != 0) {
                    context.unsupported("legacy.batches.lightmap","Legacy lightmap selectors need an explicit lightmap resource representation")?;
                }
                group.chunks_mut().push(GroupChunk::Batches(array(
                    &rows
                        .iter()
                        .map(|row| nr::Batch::<native::Classic> {
                            prefix: row.bounds,
                            start_index: u32::from(row.start_index),
                            index_count: row.index_count,
                            minimum_vertex: row.minimum_vertex,
                            maximum_vertex: row.maximum_vertex,
                            flags: row.flags,
                            material: row.material,
                        })
                        .collect::<Vec<_>>(),
                )?));
            }
            old::LegacyGroupChunk::LegacyIndices(rows) => group
                .chunks_mut()
                .push(GroupChunk::Indices(rows.clone().into_owned())),
            old::LegacyGroupChunk::Liquid(liquid) => {
                let vertices = Array::<LiquidVertex>::decode(liquid.vertices.as_bytes())
                    .map_err(|error| failure("legacy.liquid", error))?
                    .into_owned();
                group
                    .chunks_mut()
                    .push(GroupChunk::Liquid(native::group::Liquid {
                        header: liquid.header,
                        vertices: LiquidVertices::Unresolved(vertices),
                        tiles: liquid.tiles.clone().into_owned(),
                    }));
            }
            _ => context.unsupported(
                format!("legacy.group.{}", String::from_utf8_lossy(&chunk.tag())),
                "Legacy lightmap/batch feature has no active WEM equivalent",
            )?,
        }
    }
    Ok(group)
}
fn group16(
    source: &old::Group16<'_>,
    context: &mut Context,
) -> Result<Group<'static, native::Classic>, Error> {
    let h = &source.header;
    let mut group = Group::<native::Classic>::new();
    *group.header_mut() = nr::GroupHeader {
        name_offset: h.name_offset,
        descriptive_name_offset: h.descriptive_name_offset,
        flags: h.flags,
        bounds_minimum: h.bounds_minimum,
        bounds_maximum: h.bounds_maximum,
        portal_start: u16::try_from(h.portal_start.max(0))
            .map_err(|error| failure("group.portal_start", error))?,
        portal_count: h.portal_count,
        transition_batch_count: h.transition_batch_count,
        interior_batch_count: h.interior_batch_count,
        exterior_batch_count: h.exterior_batch_count,
        reserved_batch_count: h.reserved_batch_count,
        fog_indices: h.fog_indices,
        liquid_type: h.liquid_type,
        unique_id: h.unique_id as u32,
        flags2: h.flags2,
        tail: h.reserved_tail,
    };
    for chunk in &source.chunks {
        match chunk {
            old::LegacyGroupChunk::Common(chunk) => {
                group.chunks_mut().push(chunk.clone().into_owned())
            }
            old::LegacyGroupChunk::Triangles(rows) => {
                let rows = collect(rows, "legacy.triangles")?;
                if rows.iter().any(|row| row.lightmap != 0) {
                    context.unsupported("legacy.triangles.lightmap","Legacy lightmap selectors need an explicit lightmap resource representation")?;
                }
                group.chunks_mut().push(GroupChunk::Triangles(array(
                    &rows
                        .iter()
                        .map(|row| nr::Triangle {
                            flags: row.flags,
                            material: row.material,
                        })
                        .collect::<Vec<_>>(),
                )?));
            }
            old::LegacyGroupChunk::Batches(rows) => {
                let rows = collect(rows, "legacy.batches")?;
                if rows.iter().any(|row| row.extension != [0; 8]) {
                    context.unsupported(
                        "legacy.batches.extension",
                        "Beta batch extension bytes have no target semantics",
                    )?;
                }
                group.chunks_mut().push(GroupChunk::Batches(array(
                    &rows
                        .iter()
                        .map(|row| nr::Batch::<native::Classic> {
                            prefix: row.bounds,
                            start_index: row.start_index,
                            index_count: row.index_count,
                            minimum_vertex: row.minimum_vertex,
                            maximum_vertex: row.maximum_vertex,
                            flags: row.flags,
                            material: row.material,
                        })
                        .collect::<Vec<_>>(),
                )?));
            }
            old::LegacyGroupChunk::LegacyIndices(rows) => group
                .chunks_mut()
                .push(GroupChunk::Indices(rows.clone().into_owned())),
            old::LegacyGroupChunk::Liquid(liquid) => {
                let vertices = Array::<LiquidVertex>::decode(liquid.vertices.as_bytes())
                    .map_err(|error| failure("legacy.liquid", error))?
                    .into_owned();
                group
                    .chunks_mut()
                    .push(GroupChunk::Liquid(native::group::Liquid {
                        header: liquid.header,
                        vertices: LiquidVertices::Unresolved(vertices),
                        tiles: liquid.tiles.clone().into_owned(),
                    }));
            }
            _ => context.unsupported(
                format!("legacy.group.{}", String::from_utf8_lossy(&chunk.tag())),
                "Legacy lightmap/batch feature has no active WEM equivalent",
            )?,
        }
    }
    Ok(group)
}
impl Source for old::Root14<'_> {
    fn import(&self, options: &ImportOptions) -> Result<Conversion<Asset>, Error> {
        self.validate().map_err(|error| failure("wmo14", error))?;
        let mut context = Context::import(options);
        let mut root = Root::<native::Classic>::new();
        root.header_mut().wmo_id = self.header().wmo_id;
        root.header_mut().ambient_color = self.header().ambient_color;
        root.header_mut().bounds = [[f32::INFINITY; 3], [f32::NEG_INFINITY; 3]];
        for chunk in self.chunks() {
            match chunk {
                old::RootChunk14::Common(chunk) => {
                    root_chunk(&mut root, chunk.clone().into_owned())?
                }
                old::RootChunk14::Materials(rows) => {
                    let values = collect(rows, "legacy.materials")?
                        .into_iter()
                        .map(|value| nr::Material::<native::Classic> {
                            flags: value.flags,
                            blend_mode: value.blend_mode,
                            texture0: value.texture0,
                            sidn_color: value.sidn_color,
                            frame_sidn_color: value.frame_sidn_color,
                            texture1: value.texture1,
                            diffuse_color: value.diffuse_color,
                            ground_type: value.ground_type,
                            ..Default::default()
                        })
                        .collect::<Vec<_>>();
                    root_chunk(&mut root, RootChunk::Materials(array(&values)?))?;
                }
                old::RootChunk14::Groups(rows) => {
                    let values = collect(rows, "legacy.groups")?
                        .into_iter()
                        .map(|value| nr::GroupInfo {
                            flags: value.flags,
                            bounds: value.bounds,
                            name_offset: value.name_offset,
                        })
                        .collect::<Vec<_>>();
                    for value in &values {
                        for i in 0..3 {
                            root.header_mut().bounds[0][i] =
                                root.header().bounds[0][i].min(value.bounds[0][i]);
                            root.header_mut().bounds[1][i] =
                                root.header().bounds[1][i].max(value.bounds[1][i]);
                        }
                    }
                    root_chunk(&mut root, RootChunk::Groups(array(&values)?))?;
                }
                old::RootChunk14::Lights(rows) => {
                    let values = collect(rows, "legacy.lights")?
                        .into_iter()
                        .map(|value| nr::Light {
                            kind: value.kind,
                            use_attenuation: value.use_attenuation,
                            padding: value.reserved,
                            color: value.color,
                            position: value.position,
                            intensity: value.intensity,
                            rotation: [0., 0., 0., 1.],
                            attenuation_start: value.attenuation_start,
                            attenuation_end: value.attenuation_end,
                        })
                        .collect::<Vec<_>>();
                    root_chunk(&mut root, RootChunk::Lights(array(&values)?))?;
                }
            }
        }
        if self.group_count() == 0 {
            root.header_mut().bounds = [[0.; 3]; 2];
        }
        root.synchronize_counts()
            .map_err(|error| failure("legacy.root", error))?;
        let mut bundle = Bundle::from_root(root);
        for (index, source) in self.groups().enumerate() {
            bundle.insert_group(
                GroupKey { index, lod: 0 },
                FileKey::Sibling {
                    owner: Box::new(FileKey::Root),
                    suffix: format!("_{index:03}.wmo"),
                },
                group14(source, &mut context)?,
            );
        }
        let mut converted = import::modern(&bundle, options)?;
        converted
            .report
            .issues
            .splice(0..0, context.report.issues.drain(..));
        let mut raw = converted.value.document.into_raw();
        legacy_metadata14(&mut raw, self)?;
        converted.value.document = wem::Document::from_raw(raw)?;
        for chunk in self.unknown_chunks() {
            import::unknown(
                &mut converted.value,
                &mut context,
                FileKey::Root,
                chunk.tag,
                &chunk.data,
                options,
            )?;
        }
        for source in self.groups() {
            for entry in source.entries() {
                if let old::LegacyGroupEntry::Other { tag, data } = entry {
                    import::unknown(
                        &mut converted.value,
                        &mut context,
                        FileKey::Root,
                        *tag,
                        data,
                        options,
                    )?;
                }
            }
        }
        converted.report.issues.extend(context.report.issues);
        for payload in &mut converted.value.opaque {
            payload.revision = Some(14);
            payload.file = FileKey::Root;
        }
        Ok(converted)
    }
}
impl Source for old::Bundle16<'_> {
    fn import(&self, options: &ImportOptions) -> Result<Conversion<Asset>, Error> {
        self.validate_complete()
            .map_err(|error| failure("wmo16", error))?;
        let mut context = Context::import(options);
        let mut root = Root::<native::Classic>::new();
        *root.header_mut() = self.root.header().clone();
        for chunk in self.root.chunks() {
            root_chunk(&mut root, chunk.clone().into_owned())?;
        }
        let mut bundle = Bundle::from_root(root);
        for index in 0..self.root.group_count() {
            let source = self
                .group(index)
                .ok_or_else(|| Error::new("wmo16.group", "missing group companion"))?;
            bundle.insert_group(
                GroupKey { index, lod: 0 },
                FileKey::Sibling {
                    owner: Box::new(FileKey::Root),
                    suffix: format!("_{index:03}.wmo"),
                },
                group16(source, &mut context)?,
            );
        }
        let mut converted = import::modern(&bundle, options)?;
        converted
            .report
            .issues
            .splice(0..0, context.report.issues.drain(..));
        set_version(&mut converted.value, 16)?;
        for index in 0..self.root.group_count() {
            let source = self
                .group(index)
                .ok_or_else(|| Error::new("groups", "missing group"))?;
            for entry in source.entries() {
                if let old::LegacyGroupEntry::Other { tag, data } = entry {
                    import::unknown(
                        &mut converted.value,
                        &mut context,
                        FileKey::Sibling {
                            owner: Box::new(FileKey::Root),
                            suffix: format!("_{index:03}.wmo"),
                        },
                        *tag,
                        data,
                        options,
                    )?;
                }
            }
        }
        converted.report.issues.extend(context.report.issues);
        for payload in &mut converted.value.opaque {
            payload.revision = Some(16);
        }
        Ok(converted)
    }
}
