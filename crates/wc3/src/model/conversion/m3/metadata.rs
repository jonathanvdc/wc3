//! Named source control words retained independently of record bytes and offsets.
use super::{native, nr, read_error, Error};
use crate::model::wem::{self, records as wr, semantic as sem};
use native::Envelope;
use std::collections::BTreeMap;
fn put<T: wem::Record + wem::Write>(
    doc: &mut wem::RawDocument<'static>,
    rows: &[T],
) -> Result<wem::Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(wem::Ref::default());
    }
    let id = doc.append(rows)?;
    Ok(wem::Ref::from_section(rows.len() as u32, id.index()))
}
fn text(
    doc: &mut wem::RawDocument<'static>,
    value: &str,
) -> Result<wem::Ref<wr::targets::Character>, Error> {
    Ok(put(
        doc,
        &value
            .bytes()
            .map(|value| wr::Character { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
fn read(doc: &wem::RawDocument<'_>, bag: &wr::NativeBag) -> Result<BTreeMap<String, i64>, Error> {
    let mut values = BTreeMap::new();
    if bag.entries.entries == 0 {
        return Ok(values);
    }
    for row in doc
        .resolve::<wr::NativeEntry<wr::V2>>(bag.entries.cast())?
        .iter()
    {
        let row = row?;
        values.insert(
            doc.string(row.name)?.unwrap_or_default().to_owned(),
            row.value,
        );
    }
    Ok(values)
}
pub(super) fn decorate<E: Envelope>(
    source: &native::Model<'_, E>,
    document: wem::Document<'static>,
) -> Result<wem::Document<'static>, Error> {
    let mut doc = document.into_raw();
    let root = doc
        .resolve::<wr::DocumentRecord<wr::V1>>(wem::Ref::new(
            doc.root_reference().entries,
            doc.root_reference().index,
            doc.root_reference().flags,
        ))?
        .get(0)
        .ok_or_else(|| Error::new("wem.root", "missing root"))??;
    let model = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(0)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let mut sets = doc
        .resolve::<wr::ProfileMaterialSet>(model.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    let Some(set) = sets.first_mut() else {
        return Ok(wem::Document::from_raw(doc)?);
    };
    let mut values = read(&doc, &set.native)?;
    native::visit_m3_root!(source.root_record().map_err(read_error)?, |r| {
        values.insert("m3.model.flags".into(), i64::from((r.flags).bits()));
        values.insert(
            "m3.model.animation_split_count".into(),
            i64::from(r.animation_split_count),
        );
        values.insert(
            "m3.model.skin_bone_count".into(),
            i64::from(r.skin_bone_count),
        );
    });
    let native_root = source.root_record().map_err(read_error)?;
    if let native::RootRecord::V30(root) = native_root {
        let maps = super::material::complete_maps(source)?;
        for (index, map) in maps.iter().enumerate() {
            if map.material_type != nr::MaterialKind::DataDriven {
                continue;
            }
            macro_rules! read_reserved {
                ($v:ty) => {{
                    let row = source
                        .document()
                        .resolve::<nr::DataDrivenMaterial<$v>>(root.data_driven_materials.cast())
                        .map_err(read_error)?
                        .get(map.material_index as usize)
                        .ok_or_else(|| Error::new("m3.material", "missing data-driven material"))?
                        .map_err(read_error)?;
                    [
                        row.reserved_0,
                        row.reserved_1,
                        row.reserved_2,
                        row.reserved_3,
                    ]
                }};
            }
            let revision = source
                .document()
                .section_id(root.data_driven_materials.index)
                .and_then(|id| source.document().section(id))
                .ok_or_else(|| Error::new("m3.material", "missing data-driven table"))?
                .revision();
            let words = match revision {
                0 => read_reserved!(nr::V0),
                1 => read_reserved!(nr::V1),
                2 => read_reserved!(nr::V2),
                3 => read_reserved!(nr::V3),
                _ => {
                    return Err(Error::new(
                        "m3.material",
                        "unsupported data-driven revision",
                    ));
                }
            };
            for (slot, words) in words.into_iter().enumerate() {
                for (name, value) in [
                    ("entries", words.entries),
                    ("index", words.index),
                    ("third_word", words.third_word),
                ] {
                    values.insert(
                        format!("m3.materials[{index}].reserved[{slot}].{name}"),
                        i64::from(value),
                    );
                }
            }
        }
    }
    let mut rows = Vec::new();
    for (key, value) in values {
        rows.push(wr::NativeEntry::<wr::V2> {
            name: text(&mut doc, &key)?,
            value,
            text: wem::Ref::default(),
        });
    }
    set.native = wr::NativeBag {
        entries: put(&mut doc, &rows)?.cast(),
    };
    let id = doc
        .section_id(model.profile_sets.index)
        .ok_or_else(|| Error::new("wem.profile", "missing profile section"))?;
    doc.replace(id, &sets)?;
    Ok(wem::Document::from_raw(doc)?)
}
pub(super) fn values(model: &sem::Model<'_>) -> Result<BTreeMap<String, i64>, Error> {
    let Some(native) = &model.native else {
        return Ok(BTreeMap::new());
    };
    let doc = native.document.raw();
    let root = native.document.root::<wr::DocumentRecord<wr::V1>>()?;
    let model_row = doc
        .resolve::<wr::ModelRecord<wr::V9>>(root.models.cast())?
        .get(native.model as usize)
        .ok_or_else(|| Error::new("wem.model", "missing model"))??;
    let sets = doc
        .resolve::<wr::ProfileMaterialSet>(model_row.profile_sets.cast())?
        .iter()
        .collect::<Result<Vec<_>, _>>()?;
    sets.iter()
        .find(|set| set.profile == model.profile)
        .map(|set| read(doc, &set.native))
        .transpose()
        .map(|r| r.unwrap_or_default())
}

pub(super) fn restore_reserved(
    model: &sem::Model<'_>,
    maps: &[nr::MaterialMap],
    materials: &mut [nr::DataDrivenMaterial<nr::V3>],
) -> Result<(), Error> {
    let values = values(model)?;
    for (index, map) in maps.iter().enumerate() {
        if map.material_type != nr::MaterialKind::DataDriven {
            continue;
        }
        let mut words = [nr::ReservedReferenceWords::default(); 4];
        for (slot, word) in words.iter_mut().enumerate() {
            let get = |name| {
                values
                    .get(&format!("m3.materials[{index}].reserved[{slot}].{name}"))
                    .copied()
                    .unwrap_or(0) as u32
            };
            *word = nr::ReservedReferenceWords {
                entries: get("entries"),
                index: get("index"),
                third_word: get("third_word"),
            };
        }
        let row = materials
            .get_mut(map.material_index as usize)
            .ok_or_else(|| Error::new("m3.material", "missing exported data-driven material"))?;
        [
            row.reserved_0,
            row.reserved_1,
            row.reserved_2,
            row.reserved_3,
        ] = words;
    }
    Ok(())
}
