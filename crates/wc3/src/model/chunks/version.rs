//! The complete version chunk.
use crate::model::mdx;
use crate::model::Cursor;
use crate::model::ModelOwnership;
use crate::model::{mdl, ModelVersion, Tag};
use crate::model::{Chunk, KnownChunk};
use crate::model::{Encoder, ModelBytes};
use std::marker::PhantomData;

/// A complete `VERS` chunk, including bytes after the version number.
#[derive(Clone, Debug, Eq, PartialEq, mdl::Read, mdl::Write, ModelOwnership)]
#[mdl(block = "Version", validate_write = "Self::validate_mdl_write", virtual_fields(
    #[mdl(property = "FormatVersion", get = "Self::mdl_version", set = "Self::set_mdl_version")]
    format_version: u32,
))]
pub struct VersionChunk<'data, V: ModelVersion> {
    #[mdl(skip, default)]
    /// Uninterpreted bytes after the version number; omitted from MDL.
    pub extension: ModelBytes<'data>,
    #[mdl(skip, default)]
    version: PhantomData<V>,
}

impl<'data, V: ModelVersion> VersionChunk<'data, V> {
    /// Creates a version chunk with no extension bytes.
    pub fn new() -> Self {
        Self {
            extension: ModelBytes::default(),
            version: PhantomData,
        }
    }
}

impl<'data, V: ModelVersion> Default for VersionChunk<'data, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'data, V: ModelVersion> Chunk for VersionChunk<'data, V> {
    fn tag(&self) -> Tag {
        Self::TAG
    }

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        let size =
            4usize
                .checked_add(self.extension.len())
                .ok_or(mdx::WriteError::SizeOverflow {
                    field: "encoded size",
                    tag: Self::TAG,
                    size: usize::MAX,
                })?;
        if size > u32::MAX as usize {
            return Err(mdx::WriteError::SizeOverflow {
                field: "encoded size",
                tag: Self::TAG,
                size,
            });
        }

        bytes.write(&V::NUMBER)?;
        bytes.write_bytes(&self.extension);
        Ok(())
    }
}

impl<'data, V: ModelVersion> KnownChunk<'data> for VersionChunk<'data, V> {
    fn decode_payload(cursor: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
        let offset = cursor.absolute_position();
        let version = cursor.read().map_err(|error| error.with_tag(Self::TAG))?;
        let extension = cursor.remaining_bytes();
        cursor.read_bytes(extension.len())?;
        if version != V::NUMBER {
            return Err(mdx::ReadError::new(
                offset,
                mdx::ReadErrorKind::VersionMismatch {
                    expected: V::NUMBER,
                    actual: version,
                },
            )
            .with_tag(Self::TAG));
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
        original.extension = vec![9, 8, 7].into();
        let payload = original.encode_mdx().unwrap();
        assert_eq!(
            VersionChunk::<V1800>::decode_mdx(&payload).unwrap(),
            original
        );
        assert_eq!(
            VersionChunk::<V800>::decode_mdx(
                &[b"VERS".as_slice(), &3u32.to_le_bytes(), &[1, 2, 3]].concat()
            ),
            Err(mdx::ReadError {
                offset: 8,
                tag: Some(*b"VERS"),
                kind: mdx::ReadErrorKind::UnexpectedEnd {
                    needed: 4,
                    remaining: 3
                }
            })
        );
    }
}
