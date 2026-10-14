//! Common shader bodies retain every authored input, operation, and layer feature.
mod common;
mod features;
mod input;
mod values;
use crate::model::conversion::Error;
use crate::model::wem::{self, RawDocument, ReadError, Record, RecordLayout, Ref, Target, Write};
pub(super) use common::{read, write};
fn error(error: ReadError) -> Error {
    Error::new("wem.material", error.to_string())
}
fn rows<'view, T>(document: &'view RawDocument<'_>, reference: Ref<T>) -> Result<Vec<T>, Error>
where
    T: Record + RecordLayout + for<'record> wem::Read<'record>,
{
    document
        .resolve(reference)
        .map_err(error)?
        .iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(error)
}
fn one<'view, T, U: Target>(document: &'view RawDocument<'_>, reference: Ref<U>) -> Result<T, Error>
where
    T: Record + RecordLayout + for<'record> wem::Read<'record>,
{
    if reference.entries != 1 {
        return Err(Error::new(
            "wem.material",
            "body reference must select exactly one value",
        ));
    }
    rows(document, reference.cast())?
        .pop()
        .ok_or_else(|| Error::new("wem.material", "missing body"))
}
fn put<T: Record + Write, U: Target>(
    document: &mut RawDocument<'_>,
    values: &[T],
) -> Result<Ref<U>, Error> {
    if values.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(values.len())
        .map_err(|_| Error::new("wem.material", "record count exceeds u32"))?;
    let section = document
        .append(values)
        .map_err(|e| Error::new("wem.material", e.to_string()))?;
    Ok(Ref::from_section(count, section.index()))
}

#[cfg(test)]
mod tests;
