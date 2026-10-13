//! Complete root/group assembly before creating editable WEM records.
use super::*;
/// Imports geometry, materials, lights, attachments, collision, and liquid surfaces.
pub(super) fn modern<D: Dialect>(
    source: &Bundle<'_, D>,
    options: &ImportOptions,
) -> Result<Conversion<Asset>, Error> {
    source
        .validate_complete()
        .map_err(|error| failure("wmo.bundle", error))?;
    let mut context = Context::import(options);
    let mut model = s::Model {
        profile: wr::ProfileId::WOW,
        name: Cow::Borrowed("World model"),
        bounds: bounds(source.root.header().bounds),
        ..Default::default()
    };
    let material_sources = materials::import(&source.root, &mut model)?;
    let node_sources = environment::import(source, &mut model)?;
    let names = source
        .root
        .chunks()
        .find_map(|chunk| {
            if let RootChunk::GroupNames(rows) = chunk {
                Some(rows.as_bytes())
            } else {
                None
            }
        })
        .unwrap_or_default();
    let mut meshes = Vec::new();
    let dependencies = source
        .dependencies()
        .map_err(|error| failure("dependencies", error))?;
    let mut keys = BTreeSet::new();
    for dependency in &dependencies {
        if let native::DependencyKind::Group { index, lod } = dependency.kind {
            keys.insert(GroupKey { index, lod });
        }
    }
    for index in 0..source.root.header().group_count as usize {
        keys.insert(GroupKey { index, lod: 0 });
    }
    for key in &keys {
        let group = source.group(*key).ok_or_else(|| {
            Error::new(
                format!("groups[{}].lod[{}]", key.index, key.lod),
                "required group companion is missing",
            )
        })?;
        geometry::import(group, *key, names, &mut model, &mut meshes, &mut context)?;
    }
    for chunk in source.root.chunks() {
        match chunk{RootChunk::Materials(_)|RootChunk::TextureNames(_)|RootChunk::GroupNames(_)|RootChunk::Groups(_)|RootChunk::GroupFileIds(_)|RootChunk::Lights(_)|RootChunk::DoodadNames(_)|RootChunk::DoodadFileIds(_)|RootChunk::Doodads(_)|RootChunk::DoodadSets(_)=>{},RootChunk::Unknown{..}=>{},_=>context.unsupported(format!("root.{}",String::from_utf8_lossy(&chunk.tag())),"Native world visibility/environment feature has no equivalent active WEM representation")?,}
    }
    let mut raw = s::build(&model)?.into_raw();
    let root = raw
        .resolve::<wr::DocumentRecord<wr::V1>>(Ref::new(1, raw.root_reference().index, 0))?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("wem.root", "missing root"))?;
    let mut graph = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("wem.model", "missing model"))?;
    let mut profiles = raw
        .resolve::<wr::ProfileMaterialSet>(graph.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut fields = metadata::source_root(&source.root);
    for (index, material) in material_sources.iter().enumerate() {
        for (name, value, text) in material {
            fields.push((
                format!(
                    "wmo.material.{index}.{}",
                    name.trim_start_matches("wmo.material.")
                ),
                *value,
                text.clone(),
            ));
        }
    }
    if let Some(ids) = source
        .root
        .chunks()
        .find_map(|chunk| {
            if let RootChunk::GroupFileIds(rows) = chunk {
                Some(collect(rows, "group.files"))
            } else {
                None
            }
        })
        .transpose()?
    {
        fields.push(("wmo.group_file_ids".into(), 0, metadata::words(ids)));
    }
    for chunk in source.root.chunks() {
        if let RootChunk::DoodadSets(rows) = chunk {
            for (index, set) in collect(rows, "doodad.sets")?.iter().enumerate() {
                let end = set.name.iter().position(|byte| *byte == 0).unwrap_or(20);
                fields.push((
                    format!("wmo.doodad_set.{index}.name"),
                    0,
                    from_utf8(&set.name[..end])
                        .map_err(|error| failure("doodad_set.name", error))?
                        .to_owned(),
                ));
                fields.push((
                    format!("wmo.doodad_set.{index}.start"),
                    i64::from(set.start_index),
                    String::new(),
                ));
                fields.push((
                    format!("wmo.doodad_set.{index}.count"),
                    i64::from(set.count),
                    String::new(),
                ));
            }
        }
    }
    profiles[0].native = metadata::bag(&mut raw, &fields)?;
    raw.replace(raw.section_id(graph.profile_sets.index).unwrap(), &profiles)?;
    let rows = raw
        .resolve::<wr::Mesh<wr::V2>>(graph.meshes.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    for (row, source) in rows.iter().zip(&meshes) {
        let mut sections = raw
            .resolve::<wr::MeshSection>(row.sections.cast())?
            .iter()
            .collect::<Result<Vec<_>, _>>()?;
        let mut fields = source.group_fields.clone();
        fields.push((
            "wmo.triangle_flags".into(),
            0,
            metadata::words(source.triangle_flags.iter().copied()),
        ));
        fields.push((
            "wmo.triangle_materials".into(),
            0,
            metadata::words(source.triangle_materials.iter().copied()),
        ));
        fields.push((
            "wmo.source_lod".into(),
            source.group.lod as i64,
            String::new(),
        ));
        sections[0].native = metadata::bag(&mut raw, &fields)?;
        raw.replace(raw.section_id(row.sections.index).unwrap(), &sections)?;
    }
    environment::decorate(&mut raw, &mut graph, source, &node_sources)?;
    physics::decorate(&mut raw, &mut graph, source)?;
    let mut asset = Asset::new(wem::Document::from_raw(raw)?);
    for chunk in source.root.chunks() {
        if let RootChunk::Unknown { tag, data } = chunk {
            unknown(&mut asset, &mut context, FileKey::Root, *tag, data, options)?;
        }
    }
    for key in keys {
        let group = source.group(key).unwrap();
        for chunk in group.chunks() {
            if let GroupChunk::Unknown { tag, data } = chunk {
                unknown(
                    &mut asset,
                    &mut context,
                    dependencies
                        .iter()
                        .find_map(|dependency| match dependency.kind {
                            native::DependencyKind::Group { index, lod }
                                if index == key.index && lod == key.lod =>
                            {
                                Some(dependency.key.clone())
                            }
                            _ => None,
                        })
                        .unwrap_or(FileKey::Sibling {
                            owner: Box::new(FileKey::Root),
                            suffix: format!("_{:03}.wmo", key.index),
                        }),
                    *tag,
                    data,
                    options,
                )?;
            }
        }
    }
    context.issue(
        "wmo.geometry",
        IssueKind::Normalized,
        "Native groups organized as editable WEM draw sections",
    );
    Ok(Conversion {
        value: asset,
        report: context.report,
    })
}
pub(super) fn unknown(
    asset: &mut Asset,
    context: &mut Context,
    file: FileKey,
    tag: [u8; 4],
    data: &[u8],
    options: &ImportOptions,
) -> Result<(), Error> {
    let path = format!("wmo.{}", String::from_utf8_lossy(&tag));
    match options.unknown {
        UnknownPolicy::Reject => Err(Error::new(path, "unknown payload needs an explicit policy")),
        UnknownPolicy::Drop => {
            context.issue(path, IssueKind::Dropped, "Unknown payload discarded");
            Ok(())
        }
        UnknownPolicy::Archive => {
            asset.opaque.push(OpaquePayload {
                format: Format::Wmo,
                file,
                tag,
                revision: Some(17),
                count: None,
                bytes: Cow::Owned(data.to_vec()),
            });
            context.issue(
                path,
                IssueKind::Archived,
                "Unknown payload retained separately from active WEM records",
            );
            Ok(())
        }
    }
}
