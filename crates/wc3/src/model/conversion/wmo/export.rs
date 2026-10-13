//! Deterministic rebuilding of typed native root and group companions.
use super::*;
/// Plans an independently rebuilt WMO bundle for a client dialect.
pub(super) fn modern<D: Dialect>(
    asset: &Asset,
    options: &ExportOptions,
) -> Result<Plan<Bundle<'static, D>>, Error> {
    let mut context = Context::export(options);
    let model = asset.model(options)?;
    if model.unit_scale != 1. {
        context.unsupported("unit_scale","native model has no global unit scale; numeric geometry, physics, and effect rescaling needs an explicit policy")?;
    }
    if model.space != wr::CoordSpace::BLIZZARD {
        context.issue(
            "space",
            IssueKind::Normalized,
            "Authored numeric coordinates preserved; no basis transform applied",
        );
    }
    if !model.clips.is_empty() {
        context.unsupported("clips", "WMO geometry has no skeletal animation timeline")?;
    }
    let raw = asset.document.raw();
    let root_record = asset.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let graph = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root_record.models.cast())?
        .get(options.model as usize)
        .transpose()?
        .ok_or_else(|| Error::new("model", "missing selected model"))?;
    let profile = raw
        .resolve::<wr::ProfileMaterialSet>(graph.profile_sets.cast())?
        .iter()
        .find_map(|row| match row {
            Ok(row) if row.profile == model.profile => Some(Ok(row)),
            Err(error) => Some(Err(error)),
            _ => None,
        })
        .transpose()?;
    if let Some(profile) = &profile {
        for (index, row) in raw
            .resolve::<wr::MaterialRecord>(profile.materials.cast())?
            .iter()
            .enumerate()
        {
            let row = row?;
            if !matches!(row.native, wr::NativeMaterial::None) {
                context.unsupported(
                    format!("materials[{index}].native"),
                    "foreign shader payload needs an explicit WMO material mapping",
                )?;
            }
        }
    }
    let fields = profile
        .map(|profile| metadata::read(raw, profile.native))
        .transpose()?
        .unwrap_or_default();
    let mut material_fields = Vec::new();
    for index in 0..model.materials.len() {
        let prefix = format!("wmo.material.{index}.");
        let values = fields
            .iter()
            .filter_map(|(name, value)| {
                name.strip_prefix(&prefix)
                    .map(|name| (format!("wmo.material.{name}"), value.clone()))
            })
            .collect();
        material_fields.push(values);
    }
    let (materials, names) = materials::export::<D>(&model, &material_fields, &mut context)?;
    let mut root = Root::<D>::new();
    root.header_mut().bounds = [model.bounds.minimum, model.bounds.maximum];
    root.header_mut().wmo_id = metadata::number(&fields, "wmo.root.id", 0)?;
    root.header_mut().ambient_color =
        metadata::number(&fields, "wmo.root.ambient_color", 0)?.to_le_bytes();
    let flags = metadata::number(&fields, "wmo.root.flags", 0)?;
    let lod = metadata::number(&fields, "wmo.root.lod_count", 0)?;
    let mut tail = [0u8; 4];
    tail[..2].copy_from_slice(
        &u16::try_from(flags)
            .map_err(|error| failure("root.flags", error))?
            .to_le_bytes(),
    );
    tail[2..].copy_from_slice(
        &u16::try_from(lod)
            .map_err(|error| failure("root.lod", error))?
            .to_le_bytes(),
    );
    root.header_mut().tail = Decoder::new(&tail)
        .read()
        .map_err(|error| failure("root.tail", error))?;
    root_chunk(&mut root, RootChunk::Materials(array(&materials)?))?;
    if let Some(names) = names {
        root_chunk(&mut root, RootChunk::TextureNames(array(&names)?))?;
    }
    let source_meshes = raw
        .resolve::<wr::Mesh<wr::V2>>(graph.meshes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut names = vec![0u8];
    let mut infos = Vec::new();
    let mut groups = BTreeMap::<GroupKey, Group<'static, D>>::new();
    for (index, mesh) in model.meshes.iter().enumerate() {
        let projection = model
            .native
            .as_ref()
            .and_then(|native| native.mesh_projections.get(index));
        let source_mesh = &source_meshes[projection.map_or(index, |map| map.mesh as usize)];
        let section = raw
            .resolve::<wr::MeshSection>(source_mesh.sections.cast())?
            .get(projection.map_or(0, |map| map.section as usize))
            .transpose()?
            .unwrap_or_default();
        let native = metadata::read(raw, section.native)?;
        let triangle_flags = native
            .get("wmo.triangle_flags")
            .map(|(_, text)| metadata::parse_words(text))
            .transpose()?
            .unwrap_or_default();
        if !triangle_flags.is_empty() && triangle_flags.len() != mesh.triangles.len() {
            return Err(Error::new(
                format!("meshes[{index}].triangle_flags"),
                "native face flags are stale after topology edits",
            ));
        }
        let mut group = geometry::export::<D>(
            mesh,
            &triangle_flags,
            &mut context,
            &format!("meshes[{index}]"),
        )?;
        let original_materials = native
            .get("wmo.triangle_materials")
            .map(|(_, text)| metadata::parse_words(text))
            .transpose()?
            .unwrap_or_default();
        if !original_materials.is_empty() {
            if original_materials.len() != mesh.triangles.len() {
                return Err(Error::new(
                    "mesh.triangle_materials",
                    "native face materials are stale after topology edits",
                ));
            }
            // Collision-only sentinel selectors remain distinct from the active draw batch material.
            for chunk in group.chunks_mut() {
                match chunk {
                    GroupChunk::Triangles(rows) => {
                        let mut data = collect(rows, "triangles")?;
                        for (row, material) in data.iter_mut().zip(&original_materials) {
                            if *material == 255 {
                                row.material = 255;
                            }
                        }
                        *rows = array(&data)?;
                    }
                    GroupChunk::Triangles2(rows) => {
                        let mut data = collect(rows, "triangles")?;
                        for (row, material) in data.iter_mut().zip(&original_materials) {
                            if *material == 255 || *material == 65535 {
                                row.material = *material as u16;
                            }
                        }
                        *rows = array(&data)?;
                    }
                    _ => {}
                }
            }
        }
        let count = group.header().exterior_batch_count;
        let class = metadata::number(&native, "wmo.batch_class", 3)?;
        group.header_mut().exterior_batch_count = if class == 3 { count } else { 0 };
        group.header_mut().interior_batch_count = if class == 2 { count } else { 0 };
        group.header_mut().transition_batch_count = if class == 1 { count } else { 0 };
        if class == 0 {
            group
                .chunks_mut()
                .retain(|chunk| !matches!(chunk, GroupChunk::Batches(_)));
        }
        group.header_mut().flags = metadata::number(&native, "wmo.group.flags", 0)?;
        group.header_mut().flags2 = metadata::number(&native, "wmo.group.flags2", 0)?;
        group.header_mut().unique_id = metadata::number(&native, "wmo.group.id", index as u32)?;
        group.header_mut().liquid_type = metadata::number(&native, "wmo.group.liquid_type", 0)?;
        group.header_mut().fog_indices =
            metadata::number(&native, "wmo.group.fog_indices", u32::MAX)?.to_le_bytes();
        let tail = metadata::number(&native, "wmo.group.header_tail", 0)?.to_le_bytes();
        group.header_mut().tail = Decoder::new(&tail)
            .read()
            .map_err(|e| failure("group.header_tail", e))?;
        for (name, lights) in [
            ("wmo.group.light_references", true),
            ("wmo.group.doodad_references", false),
        ] {
            if let Some((_, text)) = native.get(name) {
                let values = metadata::parse_words(text)?
                    .into_iter()
                    .map(|v| u16::try_from(v).map_err(|e| failure(name, e)))
                    .collect::<Result<Vec<_>, _>>()?;
                if lights {
                    group
                        .chunks_mut()
                        .push(GroupChunk::LightReferences(array(&values)?));
                } else {
                    group
                        .chunks_mut()
                        .push(GroupChunk::DoodadReferences(array(&values)?));
                }
            }
        }
        let key = GroupKey {
            index: metadata::number(&native, "wmo.group_index", index as u32)? as usize,
            lod: mesh.lod as usize,
        };
        if let Some(target) = groups.get_mut(&key) {
            geometry::merge(target, group)?;
            liquids::restore(target, &native)?;
        } else {
            liquids::restore(&mut group, &native)?;
            groups.insert(key, group);
        }
    }
    physics::restore(raw, &graph, &mut groups, &mut context)?;
    let base_keys: Vec<_> = groups
        .keys()
        .filter(|key| key.lod == 0)
        .map(|key| key.index)
        .collect();
    let index_map: BTreeMap<_, _> = base_keys
        .iter()
        .enumerate()
        .map(|(index, original)| (*original, index))
        .collect();
    for key in &base_keys {
        let group = groups
            .get_mut(&GroupKey {
                index: *key,
                lod: 0,
            })
            .unwrap();
        let offset = names.len() as u32;
        let (mesh_name, description) = model
            .meshes
            .iter()
            .enumerate()
            .find_map(|(index, mesh)| {
                let projection = model
                    .native
                    .as_ref()
                    .and_then(|native| native.mesh_projections.get(index));
                let source_mesh =
                    source_meshes.get(projection.map_or(index, |map| map.mesh as usize))?;
                let section = raw
                    .resolve::<wr::MeshSection>(source_mesh.sections.cast())
                    .ok()?
                    .get(projection.map_or(0, |map| map.section as usize))?
                    .ok()?;
                let fields = metadata::read(raw, section.native).ok()?;
                let group = metadata::number(&fields, "wmo.group_index", 0).ok()?;
                (group as usize == *key).then(|| {
                    (
                        mesh.name.to_string(),
                        fields
                            .get("wmo.group.description")
                            .map(|(_, text)| text.clone())
                            .unwrap_or_default(),
                    )
                })
            })
            .unwrap_or(("Group".into(), String::new()));
        names.extend_from_slice(mesh_name.as_bytes());
        names.push(0);
        group.header_mut().name_offset = offset;
        group.header_mut().descriptive_name_offset = if description.is_empty() {
            0
        } else {
            let offset = names.len() as u32;
            names.extend_from_slice(description.as_bytes());
            names.push(0);
            offset
        };
        infos.push(nr::GroupInfo {
            flags: group.header().flags,
            bounds: [group.header().bounds_minimum, group.header().bounds_maximum],
            name_offset: offset as i32,
        });
    }
    root_chunk(&mut root, RootChunk::GroupNames(array(&names)?))?;
    root_chunk(&mut root, RootChunk::Groups(array(&infos)?))?;
    let levels = groups.keys().map(|key| key.lod + 1).max().unwrap_or(1);
    let old_group_count = metadata::number(&fields, "wmo.root.group_count", 0)? as usize;
    let old_ids = fields
        .get("wmo.group_file_ids")
        .map(|(_, value)| metadata::parse_words(value))
        .transpose()?
        .unwrap_or_default();
    let mut file_ids = Vec::new();
    if !old_ids.is_empty() {
        for lod in 0..levels {
            for original in &base_keys {
                let id = old_ids
                    .get(lod * old_group_count + original)
                    .copied()
                    .ok_or_else(|| {
                        Error::new("groups.files", "group FileDataID mapping is stale")
                    })?;
                file_ids.push(id);
            }
        }
        root_chunk(&mut root, RootChunk::GroupFileIds(array(&file_ids)?))?;
    } else if levels > 1 {
        return Err(Error::new(
            "groups.files",
            "WMO LOD companions need explicit FileDataID mappings",
        ));
    }
    let nodes = raw
        .resolve::<wr::Node<wr::V19>>(graph.nodes.nodes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let environment = environment::export(raw, &nodes, &mut context)?;
    if !environment.lights.is_empty() {
        root_chunk(&mut root, RootChunk::Lights(array(&environment.lights)?))?;
    }
    if !environment.doodads.is_empty() {
        root_chunk(&mut root, RootChunk::Doodads(array(&environment.doodads)?))?;
        if let Some(names) = environment.names {
            root_chunk(&mut root, RootChunk::DoodadNames(array(&names)?))?;
        }
        if let Some(ids) = environment.ids {
            root_chunk(&mut root, RootChunk::DoodadFileIds(array(&ids)?))?;
        }
    }
    for (index, points) in environment.points {
        let group = groups.get_mut(&GroupKey { index, lod: 0 }).ok_or_else(|| {
            Error::new(
                "lights.group",
                "point-light group identity no longer exists",
            )
        })?;
        group
            .chunks_mut()
            .push(GroupChunk::PointLights(array(&points)?));
    }
    let mut sets = Vec::new();
    let set_count = fields
        .keys()
        .filter(|name| name.starts_with("wmo.doodad_set.") && name.ends_with(".name"))
        .count();
    for index in 0..set_count {
        let prefix = format!("wmo.doodad_set.{index}");
        let value = fields
            .get(&format!("{prefix}.name"))
            .map(|(_, name)| name.as_str())
            .unwrap_or_default();
        if value.len() > 19 {
            return Err(Error::new(
                "doodad_set.name",
                "doodad set name exceeds nineteen bytes",
            ));
        }
        let mut name = [0; 20];
        name[..value.len()].copy_from_slice(value.as_bytes());
        sets.push(nr::DoodadSet {
            name,
            start_index: metadata::number(&fields, &format!("{prefix}.start"), 0)?,
            count: metadata::number(&fields, &format!("{prefix}.count"), 0)?,
            padding: 0,
        });
    }
    if !sets.is_empty() {
        root_chunk(&mut root, RootChunk::DoodadSets(array(&sets)?))?;
    }
    root.synchronize_counts()
        .map_err(|error| failure("root.counts", error))?;
    let mut bundle = Bundle::from_root(root);
    for (key, group) in groups {
        let index = *index_map
            .get(&key.index)
            .ok_or_else(|| Error::new("groups.lod", "LOD geometry has no base group"))?;
        let file = if file_ids.is_empty() {
            FileKey::Sibling {
                owner: Box::new(FileKey::Root),
                suffix: format!("_{index:03}.wmo"),
            }
        } else {
            FileKey::FileId(file_ids[key.lod * base_keys.len() + index])
        };
        bundle.insert_group(
            GroupKey {
                index,
                lod: key.lod,
            },
            file,
            group,
        );
    }
    bundle
        .validate_complete()
        .map_err(|error| failure("wmo.output", error))?;
    Ok(Plan::new(bundle, context.report))
}
