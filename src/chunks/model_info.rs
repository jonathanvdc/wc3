//! The complete model-information chunk.

use crate::{Error, KnownChunk, Record};

/// A complete `MODL` chunk, including bytes after the standard record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInfoChunk {
    pub info: crate::ModelInfo,
    pub extension: Vec<u8>,
}

impl ModelInfoChunk {
    pub fn new(info: crate::ModelInfo, extension: Vec<u8>) -> Self {
        Self { info, extension }
    }
}

impl Record for ModelInfoChunk {
    fn decode_one(cursor: &mut crate::Cursor<'_>, version: u32) -> Result<Self, Error> {
        let _ = version;
        let info = crate::ModelInfo::parse(cursor.remaining())?;
        cursor.read_exact(372)?;
        let extension = cursor.remaining().to_vec();
        cursor.read_exact(extension.len())?;
        Ok(Self::new(info, extension))
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let size = 372usize
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
        bytes.extend_from_slice(self.info.as_bytes());
        bytes.extend_from_slice(&self.extension);
        Ok(bytes)
    }
}

impl KnownChunk for ModelInfoChunk {
    const TAG: [u8; 4] = crate::ModelInfo::TAG;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chunk;

    #[test]
    fn keeps_extension_bytes() {
        let original = ModelInfoChunk::new(crate::ModelInfo::default(), vec![1, 2, 3]);
        let raw = original.encode_chunk().unwrap();
        assert_eq!(ModelInfoChunk::decode_chunk(&raw, 1800).unwrap(), original);
    }
}
