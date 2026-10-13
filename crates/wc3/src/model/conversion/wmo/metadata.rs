//! Named native field metadata attached to active WEM records.
use super::*;
pub(super) fn append<T: wem::Record + wem::Write, U: wem::Target>(
    doc: &mut RawDocument<'static>,
    values: &[T],
) -> Result<Ref<U>, Error> {
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let id = doc.append(values)?;
    Ok(Ref::from_section(values.len() as u32, id.index()))
}
pub(super) fn string(
    doc: &mut RawDocument<'static>,
    value: &str,
) -> Result<Ref<wr::targets::Character>, Error> {
    append(
        doc,
        &value
            .as_bytes()
            .iter()
            .map(|&value| wr::Character { value })
            .collect::<Vec<_>>(),
    )
}
pub(super) fn bag(
    doc: &mut RawDocument<'static>,
    fields: &[(String, i64, String)],
) -> Result<wr::NativeBag, Error> {
    let mut rows = Vec::new();
    for (name, value, text) in fields {
        rows.push(wr::NativeEntry::<wr::V2> {
            name: string(doc, name)?,
            value: *value,
            text: string(doc, text)?,
        });
    }
    Ok(wr::NativeBag {
        entries: append(doc, &rows)?,
    })
}
pub(super) fn read(
    doc: &RawDocument<'_>,
    bag: wr::NativeBag,
) -> Result<BTreeMap<String, (i64, String)>, Error> {
    doc.resolve::<wr::NativeEntry<wr::V2>>(bag.entries.cast())?
        .iter()
        .map(|row| {
            let row = row?;
            Ok((
                doc.string(row.name)?.unwrap_or_default().to_owned(),
                (
                    row.value,
                    doc.string(row.text)?.unwrap_or_default().to_owned(),
                ),
            ))
        })
        .collect()
}
pub(super) fn floats(values: impl IntoIterator<Item = f32>) -> String {
    values
        .into_iter()
        .map(|value| format!("{:08x}", value.to_bits()))
        .collect::<Vec<_>>()
        .join(",")
}
pub(super) fn parse_floats(value: &str) -> Result<Vec<f32>, Error> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|word| {
            u32::from_str_radix(word, 16)
                .map(f32::from_bits)
                .map_err(|error| failure("wmo.metadata.float", error))
        })
        .collect()
}
pub(super) fn words(values: impl IntoIterator<Item = u32>) -> String {
    values
        .into_iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
pub(super) fn parse_words(value: &str) -> Result<Vec<u32>, Error> {
    if value.is_empty() {
        return Ok(Vec::new());
    }
    value
        .split(',')
        .map(|word| {
            word.parse()
                .map_err(|error| failure("wmo.metadata.word", error))
        })
        .collect()
}
pub(super) fn number(
    fields: &BTreeMap<String, (i64, String)>,
    name: &str,
    default: u32,
) -> Result<u32, Error> {
    fields.get(name).map_or(Ok(default), |(value, _)| {
        u32::try_from(*value).map_err(|error| failure(name, error))
    })
}
pub(super) fn source_root<D: Dialect>(root: &Root<'_, D>) -> Vec<(String, i64, String)> {
    vec![
        ("wmo.version".into(), 17, String::new()),
        (
            "wmo.root.group_count".into(),
            i64::from(root.header().group_count),
            String::new(),
        ),
        (
            "wmo.root.flags".into(),
            i64::from(root.header().flags()),
            String::new(),
        ),
        (
            "wmo.root.lod_count".into(),
            i64::from(root.header().lod_count()),
            String::new(),
        ),
        (
            "wmo.root.id".into(),
            i64::from(root.header().wmo_id),
            String::new(),
        ),
        (
            "wmo.root.ambient_color".into(),
            i64::from(u32::from_le_bytes(root.header().ambient_color)),
            String::new(),
        ),
    ]
}
