//! The complete version chunk payload.

use crate::{Error, KnownChunk, Record};

/// A complete `VERS` payload, including bytes after the version number.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionChunk {
    pub version: u32,
    pub extension: Vec<u8>,
}

impl VersionChunk {
    pub fn new(version: u32) -> Self {
        Self {
            version,
            extension: Vec::new(),
        }
    }
}

impl Record for VersionChunk {
    fn decode_one(bytes: &[u8], _version: u32) -> Result<(Self, usize), Error> {
        let length = bytes.len();
        let value = {
            let number = bytes.get(..4).ok_or(Error::InvalidVersionChunk)?;
            Ok(Self {
                version: u32::from_le_bytes(number.try_into().expect("four-byte version")),
                extension: bytes[4..].to_vec(),
            })
        }?;
        Ok((value, length))
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
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
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        bytes.extend_from_slice(&self.extension);
        Ok(bytes)
    }
}

impl KnownChunk for VersionChunk {
    const TAG: [u8; 4] = *b"VERS";
}

#[cfg(test)]
mod version_chunk_tests {
    use super::*;
    use crate::Chunk;

    #[test]
    fn preserves_version_extension_bytes() {
        let original = VersionChunk {
            version: 1800,
            extension: vec![9, 8, 7],
        };
        let chunk = original.encode_chunk().unwrap();
        assert_eq!(VersionChunk::decode_chunk(&chunk, 1800).unwrap(), original);
        assert_eq!(
            VersionChunk::decode(&[1, 2, 3], 800),
            Err(Error::InvalidVersionChunk)
        );
    }
}
