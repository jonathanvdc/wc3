//! Text and path references borrow only their actual character buffers.
use crate::model::conversion::Error;
use crate::model::wem::{records, RawDocument, Ref};
use std::borrow::Cow;

pub(super) fn read_bytes<'view>(
    document: &'view RawDocument<'_>,
    reference: Ref<records::targets::Character>,
) -> Result<Cow<'view, [u8]>, Error> {
    let Some(section) = document
        .referenced_section(reference)
        .map_err(|error| Error::new("wem.text", error.to_string()))?
    else {
        return Ok(Cow::Borrowed(&[]));
    };
    if section.tag() != *b"CHAR" || section.revision() != 0 {
        return Err(Error::new(
            "wem.text",
            "character reference has a different layout",
        ));
    }
    section
        .bytes()
        .get(..reference.entries as usize)
        .map(Cow::Borrowed)
        .ok_or_else(|| Error::new("wem.text", "character span exceeds its section"))
}
pub(super) fn write_bytes(
    document: &mut RawDocument<'_>,
    bytes: &[u8],
) -> Result<Ref<records::targets::Character>, Error> {
    if bytes.is_empty() {
        return Ok(Ref::default());
    }
    let count = u32::try_from(bytes.len())
        .map_err(|_| Error::new("wem.text", "character count exceeds u32"))?;
    let values: Vec<_> = bytes
        .iter()
        .map(|value| records::Character { value: *value })
        .collect();
    let id = document
        .append(&values)
        .map_err(|error| Error::new("wem.text", error.to_string()))?;
    Ok(Ref::from_section(count, id.index()))
}
pub(super) fn read<'view>(
    document: &'view RawDocument<'_>,
    reference: Ref<records::targets::Character>,
) -> Result<Cow<'view, str>, Error> {
    document
        .string(reference)
        .map(|text| Cow::Borrowed(text.unwrap_or("")))
        .map_err(|error| Error::new("wem.text", error.to_string()))
}
