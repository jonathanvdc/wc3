//! The complete model-information chunk.
use crate::EncodeError;
use crate::Encoder;
use crate::Tag;

use crate::{Chunk, DecodeError, KnownChunk, Writable};
use crate::{Cursor, ModelInfo};

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

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let size =
            372usize
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

        self.info.write_to(bytes)?;
        bytes.write_bytes(&self.extension);
        Ok(())
    }
}

impl KnownChunk for ModelInfoChunk {
    fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
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
    use crate::Readable;

    #[test]
    fn keeps_extension_bytes() {
        let original = ModelInfoChunk::new(ModelInfo::default(), vec![1, 2, 3]);
        let payload = original.encode().unwrap();
        assert_eq!(ModelInfoChunk::decode(&payload).unwrap(), original);
    }
}
