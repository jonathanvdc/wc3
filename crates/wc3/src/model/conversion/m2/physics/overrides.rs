//! Sparse customization matrices represented as named WEM matrix-pose slots.
use super::*;
pub(in super::super) fn import_overrides<D: Dialect>(
    document: wem::Document<'static>,
    source: &Bundle<'_, D>,
) -> Result<wem::Document<'static>, Error> {
    let files: Vec<_> = source
        .files()
        .filter_map(|(key, file)| {
            if let File::Bone(value) = file {
                Some((key, value))
            } else {
                None
            }
        })
        .collect();
    if files.is_empty() {
        return Ok(document);
    }
    let root = document.root::<wr::DocumentRecord<wr::V1>>()?;
    let mut model = document
        .raw()
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut nodes: Vec<_> = document
        .raw()
        .resolve::<wr::Node<wr::V19>>(model.nodes.nodes.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    let mut raw = document.into_raw();
    let mut schemas = Vec::new();
    let mut matrices = vec![Vec::new(); nodes.len()];
    let mut sparse_indices = Vec::new();
    for (key, file) in files {
        if file.indices.len() != file.matrices.len() {
            return Err(Error::new("bonefile", "sparse index/matrix counts differ"));
        }
        let slot = schemas.len();
        let name = match key {
            native::FileKey::FileId(id) => format!("m2-customization-{id}"),
            native::FileKey::Path(path) => path.clone(),
            _ => format!("m2-customization-{slot}"),
        };
        let chars: Vec<_> = name.bytes().map(|value| wr::Character { value }).collect();
        schemas.push(wr::PoseSchema::<wr::V2> {
            name: put(&mut raw, &chars)?.cast(),
            storage: wr::PoseStorage::MATRIX,
            ..Default::default()
        });
        for values in &mut matrices {
            values.push(wr::Matrix44 { value: identity() });
        }
        for (index, matrix) in file.indices.iter().zip(&file.matrices) {
            let values = matrices.get_mut(usize::from(index)).ok_or_else(|| {
                Error::new("bonefile.index", "override references a missing bone")
            })?;
            values[slot] = wr::Matrix44 {
                value: from_fn(|row| from_fn(|column| matrix.values[row * 4 + column])),
            };
        }
        let words = file
            .indices
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        sparse_indices.push((slot, words, key.clone()));
    }
    for (node, values) in nodes.iter_mut().zip(matrices) {
        node.pose_matrices = put(&mut raw, &values)?.cast();
    }
    let nodes_id = raw
        .section_id(model.nodes.nodes.index)
        .ok_or_else(|| Error::new("wem.nodes", "missing node table"))?;
    raw.replace(nodes_id, &nodes)?;
    model.nodes.pose_schema = put(&mut raw, &schemas)?.cast();
    let model_id = raw
        .section_id(root.models.index)
        .ok_or_else(|| Error::new("wem.model", "missing model table"))?;
    raw.replace(model_id, &[model.clone()])?;
    let mut profile = raw
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.profile", "missing profile"))??;
    append_bag(
        &mut raw,
        &mut profile.native,
        &[("m2.overrides.count", schemas.len() as i64)],
    )?;
    let mut entries: Vec<_> = raw
        .resolve::<wr::NativeEntry<wr::V2>>(profile.native.entries.cast())?
        .iter()
        .collect::<Result<_, _>>()?;
    for (slot, indices, key) in sparse_indices {
        let name = format!("m2.override.{slot}.indices");
        let name = put(
            &mut raw,
            &name
                .bytes()
                .map(|value| wr::Character { value })
                .collect::<Vec<_>>(),
        )?
        .cast();
        let text = put(
            &mut raw,
            &indices
                .bytes()
                .map(|value| wr::Character { value })
                .collect::<Vec<_>>(),
        )?
        .cast();
        let value = match key {
            native::FileKey::FileId(id) => i64::from(id),
            _ => -1,
        };
        entries.push(wr::NativeEntry::<wr::V2> { name, value, text });
    }
    profile.native.entries = put(&mut raw, &entries)?.cast();
    let profile_id = raw
        .section_id(model.profile_sets.index)
        .ok_or_else(|| Error::new("wem.profile", "missing profile table"))?;
    raw.replace(profile_id, &[profile])?;
    Ok(wem::Document::from_raw(raw)?)
}

pub(in super::super) fn export_overrides<D: ExportDialect>(
    model: &sm::Model<'_>,
    bundle: &mut Bundle<'static, D>,
    context: &mut Context,
) -> Result<(), Error> {
    let Some(native) = &model.native else {
        return Ok(());
    };
    let data = profile_metadata(model)?;
    let count = data.get("m2.overrides.count").copied().unwrap_or(0) as usize;
    if count == 0 {
        return Ok(());
    }
    let raw = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let record = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let profile = raw
        .resolve::<wr::ProfileMaterialSet>(record.profile_sets.cast())?
        .iter()
        .find_map(|p| match p {
            Ok(p) if p.profile == wr::ProfileId::WOW => Some(Ok(p)),
            Err(e) => Some(Err(e)),
            _ => None,
        })
        .transpose()?
        .ok_or_else(|| Error::new("wem.profile", "missing WOW profile"))?;
    let schemas = raw.resolve::<wr::PoseSchema<wr::V2>>(record.nodes.pose_schema.cast())?;
    let nodes = native_nodes(model)?;
    let mut ids = Vec::new();
    for slot in 0..count {
        let key = format!("m2.override.{slot}.indices");
        let indices = index_words(metadata_text(raw, &profile.native, &key)?)?;
        let mut matrices = Vec::new();
        for index in indices.iter() {
            let node = nodes
                .get(usize::from(index))
                .ok_or_else(|| Error::new("bonefile.index", "missing override node"))?;
            let matrix = raw
                .resolve::<wr::Matrix44>(node.pose_matrices.cast())?
                .get(slot)
                .ok_or_else(|| Error::new("bonefile.pose", "missing matrix pose slot"))??;
            matrices.push(native::companions::BoneMatrix {
                values: from_fn(|index| matrix.value[index / 4][index % 4]),
            });
        }
        let id = data.get(&key).copied().unwrap_or(-1);
        let filekey = if id >= 0 {
            ids.push(id as u32);
            native::FileKey::FileId(id as u32)
        } else {
            let schema = schemas
                .get(slot)
                .ok_or_else(|| Error::new("bonefile.pose", "missing schema"))??;
            native::FileKey::Path(raw.string(schema.name)?.unwrap_or_default().to_owned())
        };
        bundle.insert(
            filekey,
            File::Bone(native::companions::BoneFile {
                version: 1,
                indices,
                matrices,
            }),
        );
    }
    D::bone_ids(&mut bundle.model, ids, context)?;
    Ok(())
}
