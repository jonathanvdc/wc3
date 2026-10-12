//! The complete model-information chunk.
use crate::model::common::Ownership;
use crate::model::mdlx::mdx;
use crate::model::mdlx::Encoder;
use crate::model::mdlx::Tag;
use crate::model::mdlx::{Chunk, KnownChunk};
use crate::model::mdlx::{Cursor, ModelInfo};
use std::borrow::Cow;

/// A complete `MODL` chunk, including bytes after the standard record.
#[derive(Clone, Debug, PartialEq, Ownership, mdx::ReadOwned)]
pub struct ModelInfoChunk<'data> {
    /// Decoded model metadata and bounds.
    pub info: ModelInfo,
    /// Uninterpreted trailing MDX bytes, preserved when writing MDX.
    pub extension: Cow<'data, [u8]>,
}

impl<'data> ModelInfoChunk<'data> {
    /// Creates a chunk from metadata and trailing extension bytes.
    pub fn new(info: ModelInfo, extension: impl Into<Cow<'data, [u8]>>) -> Self {
        Self {
            info,
            extension: extension.into(),
        }
    }
}

impl<'data> Chunk for ModelInfoChunk<'data> {
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

impl<'data> KnownChunk<'data> for ModelInfoChunk<'data> {
    fn decode_payload(cursor: &mut Cursor<'data>) -> Result<Self, mdx::ReadError> {
        let info = cursor.read()?;
        let extension = cursor.remaining_bytes();
        cursor.read_bytes(extension.len())?;
        Ok(Self::new(info, extension))
    }

    const TAG: Tag = ModelInfo::TAG;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::mdlx::mdx::Read;
    use crate::model::mdlx::mdx::Write;

    #[test]
    fn keeps_extension_bytes() {
        let original = ModelInfoChunk::new(ModelInfo::default(), vec![1, 2, 3]);
        let payload = original.encode_mdx().unwrap();
        assert_eq!(ModelInfoChunk::decode_mdx(&payload).unwrap(), original);
    }
}
