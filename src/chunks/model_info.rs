//! The complete model-information chunk.
use crate::Encoder;
use crate::{Tag, Version};

use crate::{Cursor, ModelInfo};
use crate::{Error, KnownChunk, Record};

/// A complete `MODL` chunk, including bytes after the standard record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelInfoChunk {
    pub info: ModelInfo,
    pub extension: Vec<u8>,
}

impl ModelInfoChunk {
    pub fn new(info: ModelInfo, extension: Vec<u8>) -> Self {
        Self { info, extension }
    }
}

impl Record for ModelInfoChunk {
    fn decode_one(cursor: &mut Cursor<'_>, version: Version) -> Result<Self, Error> {
        let _ = version;
        let info = ModelInfo::decode_one(cursor, version)?;
        let extension = cursor.remaining().to_vec();
        cursor.read_exact(extension.len())?;
        Ok(Self::new(info, extension))
    }

    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
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

        self.info.encode_to(bytes)?;
        bytes.write_bytes(&self.extension);
        Ok(())
    }
}

impl KnownChunk for ModelInfoChunk {
    const TAG: Tag = ModelInfo::TAG;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_extension_bytes() {
        let original = ModelInfoChunk::new(ModelInfo::default(), vec![1, 2, 3]);
        let payload = original.encode().unwrap();
        assert_eq!(ModelInfoChunk::decode(&payload, 1800).unwrap(), original);
    }
}
