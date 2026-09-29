//! The complete version chunk.
use crate::model::Encoder;
use crate::model::WriteError;
use crate::model::{mdl, ModelVersion, Tag};
use std::marker::PhantomData;

use crate::model::Cursor;
use crate::model::{Chunk, KnownChunk, ReadError};

/// A complete `VERS` chunk, including bytes after the version number.
#[derive(Clone, Debug, Eq, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "Version", validate_write = "Self::validate_mdl_write", virtual_fields(
    #[mdl(property = "FormatVersion", get = "Self::mdl_version", set = "Self::set_mdl_version")]
    format_version: u32,
))]
pub struct VersionChunk<V: ModelVersion> {
    #[mdl(skip, default)]
    pub extension: Vec<u8>,
    #[mdl(skip, default)]
    version: PhantomData<V>,
}

impl<V: ModelVersion> VersionChunk<V> {
    pub fn new() -> Self {
        Self {
            extension: Vec::new(),
            version: PhantomData,
        }
    }
}

impl<V: ModelVersion> Default for VersionChunk<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V: ModelVersion> Chunk for VersionChunk<V> {
    fn tag(&self) -> Tag {
        Self::TAG
    }

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        let size = 4usize
            .checked_add(self.extension.len())
            .ok_or(WriteError::ChunkTooLarge {
                tag: Self::TAG,
                size: usize::MAX,
            })?;
        if size > u32::MAX as usize {
            return Err(WriteError::ChunkTooLarge {
                tag: Self::TAG,
                size,
            });
        }

        bytes.write(&V::NUMBER)?;
        bytes.write_bytes(&self.extension);
        Ok(())
    }
}

impl<V: ModelVersion> KnownChunk for VersionChunk<V> {
    fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let version = cursor.read().map_err(|_| ReadError::InvalidVersionChunk)?;
        let extension = cursor.remaining().to_vec();
        cursor.read_bytes(extension.len())?;
        if version != V::NUMBER {
            return Err(ReadError::VersionMismatch {
                expected: V::NUMBER,
                actual: version,
            });
        }
        Ok(Self {
            extension,
            version: PhantomData,
        })
    }

    const TAG: Tag = *b"VERS";
}

#[cfg(test)]
mod version_chunk_tests {
    use super::*;
    use crate::model::{mdx::Read, mdx::Write, V1800, V800};

    #[test]
    fn preserves_version_extension_bytes() {
        let mut original = VersionChunk::<V1800>::new();
        original.extension = vec![9, 8, 7];
        let payload = original.encode_mdx().unwrap();
        assert_eq!(
            VersionChunk::<V1800>::decode_mdx(&payload).unwrap(),
            original
        );
        assert_eq!(
            VersionChunk::<V800>::decode_mdx(
                &[b"VERS".as_slice(), &3u32.to_le_bytes(), &[1, 2, 3]].concat()
            ),
            Err(ReadError::InvalidVersionChunk)
        );
    }
}
