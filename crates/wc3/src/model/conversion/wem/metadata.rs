//! Ordered authoring metadata retains duplicate names and independently borrowed text.
use crate::model::conversion::wem::text;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::wem::{Metadata, MetadataEntry};
use crate::model::wem::{records, RawDocument, Ref};

pub(super) fn read<'view>(
    document: &'view RawDocument<'_>,
    bag: &records::NativeBag,
) -> Result<Metadata<'view>, Error> {
    let Some(section) = document
        .referenced_section(bag.entries)
        .map_err(|error| Error::new("wem.metadata", error.to_string()))?
    else {
        return Ok(Metadata::default());
    };
    let mut entries = Vec::new();
    match section.revision() {
        1 => {
            for value in document
                .resolve::<records::NativeEntry<records::V1>>(bag.entries.cast())
                .map_err(|error| Error::new("wem.metadata", error.to_string()))?
                .iter()
            {
                let value = value.map_err(|error| Error::new("wem.metadata", error.to_string()))?;
                entries.push(MetadataEntry {
                    name: text::read(document, value.name)?,
                    value: value.value,
                    text: Default::default(),
                });
            }
        }
        2 => {
            for value in document
                .resolve::<records::NativeEntry<records::V2>>(bag.entries.cast())
                .map_err(|error| Error::new("wem.metadata", error.to_string()))?
                .iter()
            {
                let value = value.map_err(|error| Error::new("wem.metadata", error.to_string()))?;
                entries.push(MetadataEntry {
                    name: text::read(document, value.name)?,
                    value: value.value,
                    text: text::read(document, value.text)?,
                });
            }
        }
        _ => {
            return Err(Error::new(
                "wem.metadata",
                "unsupported authoring entry revision",
            ));
        }
    }
    Ok(Metadata::new(entries))
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    metadata: &Metadata<'_>,
) -> Result<records::NativeBag, Error> {
    let mut entries = Vec::new();
    for entry in metadata.entries() {
        entries.push(records::NativeEntry::<records::V2> {
            name: text::write_bytes(document, entry.name.as_bytes())?,
            value: entry.value,
            text: text::write_bytes(document, entry.text.as_bytes())?,
        });
    }
    if entries.is_empty() {
        return Ok(records::NativeBag::default());
    }
    let count = u32::try_from(entries.len())
        .map_err(|_| Error::new("wem.metadata", "entry count exceeds u32"))?;
    let id = document
        .append(&entries)
        .map_err(|error| Error::new("wem.metadata", error.to_string()))?;
    Ok(records::NativeBag {
        entries: Ref::from_section(count, id.index()),
    })
}
