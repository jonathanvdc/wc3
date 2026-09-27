//! The complete version chunk.
use crate::EncodeError;
use crate::Encoder;
use crate::{ModelVersion, Tag};
use std::marker::PhantomData;

use crate::Cursor;
use crate::{Chunk, DecodeError, KnownChunk};

/// A complete `VERS` chunk, including bytes after the version number.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionChunk<V: ModelVersion> {
    pub extension: Vec<u8>,
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

impl<V: ModelVersion> Chunk for VersionChunk<V> {
    fn tag(&self) -> Tag {
        Self::TAG
    }

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let size = 4usize
            .checked_add(self.extension.len())
            .ok_or(EncodeError::ChunkTooLarge {
                tag: Self::TAG,
                size: usize::MAX,
            })?;
        if size > u32::MAX as usize {
            return Err(EncodeError::ChunkTooLarge {
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
    fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let version = cursor
            .read()
            .map_err(|_| DecodeError::InvalidVersionChunk)?;
        let extension = cursor.remaining().to_vec();
        cursor.read_exact(extension.len())?;
        if version != V::NUMBER {
            return Err(DecodeError::VersionMismatch {
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
    use crate::{Readable, Writable, V1800, V800};

    #[test]
    fn preserves_version_extension_bytes() {
        let mut original = VersionChunk::<V1800>::new();
        original.extension = vec![9, 8, 7];
        let payload = original.encode().unwrap();
        assert_eq!(VersionChunk::<V1800>::decode(&payload).unwrap(), original);
        assert_eq!(
            VersionChunk::<V800>::decode(
                &[b"VERS".as_slice(), &3u32.to_le_bytes(), &[1, 2, 3]].concat()
            ),
            Err(DecodeError::InvalidVersionChunk)
        );
    }
}
