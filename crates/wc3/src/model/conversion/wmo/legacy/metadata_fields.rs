//! Historical root/material values absent from modern native layouts.
use super::*;
pub(super) fn set_version(asset: &mut Asset, version: u32) -> Result<(), Error> {
    let mut raw = asset.document.clone().into_raw();
    let root = raw
        .resolve::<wr::DocumentRecord<wr::V1>>(Ref::new(1, raw.root_reference().index, 0))?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("root", "missing root"))?;
    let model = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("model", "missing model"))?;
    let mut rows = raw
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut values = metadata::read(&raw, rows[0].native.clone())?;
    values.insert("wmo.version".into(), (version as i64, String::new()));
    rows[0].native = metadata::bag(
        &mut raw,
        &values
            .into_iter()
            .map(|(name, (number, text))| (name, number, text))
            .collect::<Vec<_>>(),
    )?;
    raw.replace(raw.section_id(model.profile_sets.index).unwrap(), &rows)?;
    asset.document = wem::Document::from_raw(raw)?;
    Ok(())
}
pub(super) fn legacy_metadata14(
    raw: &mut RawDocument<'static>,
    source: &old::Root14<'_>,
) -> Result<(), Error> {
    let root = raw
        .resolve::<wr::DocumentRecord<wr::V1>>(Ref::new(1, raw.root_reference().index, 0))?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("root", "missing root"))?;
    let model = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .transpose()?
        .ok_or_else(|| Error::new("model", "missing model"))?;
    let mut profiles = raw
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let mut fields = metadata::read(raw, profiles[0].native.clone())?;
    fields.insert("wmo.version".into(), (14, String::new()));
    fields.insert(
        "wmo.legacy14.header_reserved".into(),
        (
            0,
            metadata::words(source.header().reserved.into_iter().map(u32::from)),
        ),
    );
    let mut material_index = 0;
    for chunk in source.chunks() {
        if let old::RootChunk14::Materials(rows) = chunk {
            for material in collect(rows, "material14")? {
                fields.insert(
                    format!("wmo.material.{material_index}.legacy_version"),
                    (material.version as i64, String::new()),
                );
                fields.insert(
                    format!("wmo.material.{material_index}.legacy_reserved"),
                    (
                        0,
                        metadata::words(material.reserved.into_iter().map(u32::from)),
                    ),
                );
                material_index += 1;
            }
        }
    }
    profiles[0].native = metadata::bag(
        raw,
        &fields
            .into_iter()
            .map(|(name, (number, text))| (name, number, text))
            .collect::<Vec<_>>(),
    )?;
    raw.replace(raw.section_id(model.profile_sets.index).unwrap(), &profiles)?;
    Ok(())
}
pub(super) fn profile_fields(
    asset: &Asset,
    options: &ExportOptions,
) -> Result<BTreeMap<String, (i64, String)>, Error> {
    let raw = asset.document.raw();
    let root = asset.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model = raw
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(options.model as usize)
        .transpose()?
        .ok_or_else(|| Error::new("model", "missing model"))?;
    for profile in raw
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
    {
        let profile = profile?;
        if profile.profile == options.profile.unwrap_or(root.default_profile) {
            return metadata::read(raw, profile.native);
        }
    }
    Ok(BTreeMap::new())
}
