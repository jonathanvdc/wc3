//! Shared images are emitted once; shader filename leaves reuse their declaration buffers.
use super::super::resources;
use crate::model::conversion::Error;
use crate::model::semantic::profiles::Wem;
use crate::model::semantic::{Resources, TextureId};
use crate::model::wem::{records, RawDocument, Ref};
use std::collections::BTreeMap;
pub(super) struct Encoded {
    pub reference: Ref<records::targets::TextureRef>,
    pub indices: BTreeMap<TextureId, u32>,
    pub filenames: BTreeMap<TextureId, Ref<records::targets::Character>>,
}
pub(super) fn write(
    document: &mut RawDocument<'_>,
    values: &Resources<'_, Wem>,
) -> Result<Encoded, Error> {
    let mut indices = BTreeMap::new();
    let mut filenames = BTreeMap::new();
    let mut rows = Vec::new();
    for (id, value) in values.textures() {
        indices.insert(
            id,
            u32::try_from(rows.len())
                .map_err(|_| Error::new("wem.textures", "shared image count exceeds u32"))?,
        );
        let row = resources::write(document, value)?;
        let filename = match row.key {
            records::TextureKey::Path(ref value) => Some(value.value),
            _ if row.path.entries != 0 => Some(row.path),
            _ => None,
        };
        if let Some(filename) = filename {
            filenames.insert(id, filename);
        }
        rows.push(row);
    }
    let reference = if rows.is_empty() {
        Ref::default()
    } else {
        let count = u32::try_from(rows.len())
            .map_err(|_| Error::new("wem.textures", "shared image count exceeds u32"))?;
        let section = document
            .append(&rows)
            .map_err(|error| Error::new("wem.textures", error.to_string()))?;
        Ref::from_section(count, section.index())
    };
    Ok(Encoded {
        reference,
        indices,
        filenames,
    })
}
