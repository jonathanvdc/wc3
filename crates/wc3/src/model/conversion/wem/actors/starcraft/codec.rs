use crate::model::conversion::Error;
use crate::model::semantic::Values;
use crate::model::wem::{records, RawDocument, Record, RecordLayout, Ref, Write};
use std::collections::BTreeMap;
use std::fmt::Display;
pub(super) fn read_error(error: impl Display) -> Error {
    Error::new("wem.starcraft", error.to_string())
}
pub(super) fn append<T: Record + RecordLayout + Write>(
    document: &mut RawDocument<'_>,
    rows: &[T],
) -> Result<Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(rows.len()).map_err(read_error)?;
    let section = document.append(rows).map_err(read_error)?;
    Ok(Ref::from_section(count, section.index()))
}
pub(super) fn read_words<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::U32>,
) -> Result<Values<'a, u32>, Error> {
    let rows = document
        .resolve::<records::U32>(reference.cast())
        .map_err(read_error)?;
    for row in rows.iter() {
        row.map_err(read_error)?;
    }
    Ok(Values::view(rows.len(), move |index| {
        rows.get(index)
            .expect("checked selector")
            .expect("validated selector")
            .value
    }))
}
pub(super) fn words(
    document: &mut RawDocument<'_>,
    rows: impl IntoIterator<Item = u32>,
) -> Result<Ref<records::targets::U32>, Error> {
    Ok(append(
        document,
        &rows
            .into_iter()
            .map(|value| records::U32 { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
pub(super) fn lookup<T: Copy>(table: &[T], index: u32, field: &str) -> Result<T, Error> {
    table
        .get(index as usize)
        .copied()
        .ok_or_else(|| Error::new("wem.starcraft", format!("{field} belongs to another model")))
}
pub(super) fn mapped<T: Ord>(table: &BTreeMap<T, u32>, id: T, field: &str) -> Result<u32, Error> {
    table
        .get(&id)
        .copied()
        .ok_or_else(|| Error::new("wem.starcraft", format!("{field} has no output ordinal")))
}
