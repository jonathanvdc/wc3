use crate::model::conversion::Error;
use crate::model::semantic::Values;
use crate::model::wem::{records, RawDocument, Record, RecordLayout, Ref, Write};
use std::fmt::Display;
pub(super) fn error(value: impl Display) -> Error {
    Error::new("wem.program", value.to_string())
}
pub(super) fn append<T: Record + RecordLayout + Write>(
    document: &mut RawDocument<'_>,
    rows: &[T],
) -> Result<Ref<T>, Error> {
    if rows.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(rows.len()).map_err(error)?;
    let section = document.append(rows).map_err(error)?;
    Ok(Ref::from_section(count, section.index()))
}
pub(super) fn read_words<'a>(
    document: &'a RawDocument<'_>,
    reference: Ref<records::targets::U32>,
) -> Result<Values<'a, u32>, Error> {
    let rows = document
        .resolve::<records::U32>(reference.cast())
        .map_err(error)?;
    for row in rows.iter() {
        row.map_err(error)?;
    }
    Ok(Values::view(rows.len(), move |index| {
        rows.get(index)
            .expect("checked program word")
            .expect("validated program word")
            .value
    }))
}
pub(super) fn words(
    document: &mut RawDocument<'_>,
    values: impl IntoIterator<Item = u32>,
) -> Result<Ref<records::targets::U32>, Error> {
    Ok(append(
        document,
        &values
            .into_iter()
            .map(|value| records::U32 { value })
            .collect::<Vec<_>>(),
    )?
    .cast())
}
