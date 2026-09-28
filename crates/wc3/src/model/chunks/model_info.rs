//! The complete model-information chunk.
use crate::model::Encoder;
use crate::model::Tag;
use crate::model::WriteError;

use crate::model::{Chunk, KnownChunk, ReadError};
use crate::model::{Cursor, ModelInfo};

/// A complete `MODL` chunk, including bytes after the standard record.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelInfoChunk {
    pub info: ModelInfo,
    pub extension: Vec<u8>,
}

impl ModelInfoChunk {
    pub fn new(info: ModelInfo, extension: Vec<u8>) -> Self {
        Self { info, extension }
    }
}

impl Chunk for ModelInfoChunk {
    fn tag(&self) -> Tag {
        Self::TAG
    }

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), WriteError> {
        let size = 372usize
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

        bytes.write(&self.info)?;
        bytes.write_bytes(&self.extension);
        Ok(())
    }
}

impl KnownChunk for ModelInfoChunk {
    fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let info = cursor.read()?;
        let extension = cursor.remaining().to_vec();
        cursor.read_exact(extension.len())?;
        Ok(Self::new(info, extension))
    }

    const TAG: Tag = ModelInfo::TAG;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::mdx::Read;
    use crate::model::mdx::Write;

    #[test]
    fn keeps_extension_bytes() {
        let original = ModelInfoChunk::new(ModelInfo::default(), vec![1, 2, 3]);
        let payload = original.encode_mdx().unwrap();
        assert_eq!(ModelInfoChunk::decode_mdx(&payload).unwrap(), original);
    }
}
