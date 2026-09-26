//! The complete version chunk payload.
use crate::Encoder;
use crate::{Tag, Version};

use crate::Cursor;
use crate::{Error, KnownChunk, Record};

/// A complete `VERS` payload, including bytes after the version number.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionChunk {
    pub version: Version,
    pub extension: Vec<u8>,
}

impl VersionChunk {
    pub fn new(version: Version) -> Self {
        Self {
            version,
            extension: Vec::new(),
        }
    }
}

impl Record for VersionChunk {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let version = cursor.read_u32().map_err(|_| Error::InvalidVersionChunk)?;
        let extension = cursor.remaining().to_vec();
        cursor.read_exact(extension.len())?;
        Ok(Self { version, extension })
    }

    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        let size = 4usize
            .checked_add(self.extension.len())
            .ok_or(Error::ChunkTooLarge {
                tag: Self::TAG,
                size: usize::MAX,
            })?;
        if size > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: Self::TAG,
                size,
            });
        }

        bytes.write(self.version);
        bytes.write_bytes(&self.extension);
        Ok(())
    }
}

impl KnownChunk for VersionChunk {
    const TAG: Tag = *b"VERS";
}

#[cfg(test)]
mod version_chunk_tests {
    use super::*;

    #[test]
    fn preserves_version_extension_bytes() {
        let original = VersionChunk {
            version: 1800,
            extension: vec![9, 8, 7],
        };
        let payload = original.encode().unwrap();
        assert_eq!(VersionChunk::decode(&payload, 1800).unwrap(), original);
        assert_eq!(
            VersionChunk::decode(&[1, 2, 3], 800),
            Err(Error::InvalidVersionChunk)
        );
    }
}
