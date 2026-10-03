//! The complete model-information chunk.
use crate::model::mdx;
use crate::model::Encoder;
use crate::model::Tag;
use crate::model::{Chunk, KnownChunk};
use crate::model::{Cursor, ModelInfo};

/// A complete `MODL` chunk, including bytes after the standard record.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelInfoChunk {
    /// Decoded model metadata and bounds.
    pub info: ModelInfo,
    /// Uninterpreted trailing MDX bytes, preserved when writing MDX.
    pub extension: Vec<u8>,
}

impl ModelInfoChunk {
    /// Creates a chunk from metadata and trailing extension bytes.
    pub fn new(info: ModelInfo, extension: Vec<u8>) -> Self {
        Self { info, extension }
    }
}

impl Chunk for ModelInfoChunk {
    fn tag(&self) -> Tag {
        Self::TAG
    }

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        let size =
            372usize
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

        bytes.write(&self.info)?;
        bytes.write_bytes(&self.extension);
        Ok(())
    }
}

impl KnownChunk for ModelInfoChunk {
    fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        let info = cursor.read()?;
        let extension = cursor.remaining().to_vec();
        cursor.read_bytes(extension.len())?;
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
