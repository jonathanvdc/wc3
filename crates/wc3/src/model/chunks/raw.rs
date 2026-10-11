use super::Chunk;
use crate::model::mdx;
use crate::model::ModelOwnership;
use crate::model::{Encoder, Tag};
use std::borrow::Cow;

/// A tagged top-level MDX chunk. The payload is stored without interpretation.
#[derive(Clone, Debug, Eq, PartialEq, ModelOwnership)]
pub struct RawChunk<'data> {
    /// Four-byte binary identifier, such as `VERS`, `MODL`, or `GEOS`.
    pub tag: Tag,
    /// Original payload bytes, preserved on round-trip.
    pub data: Cow<'data, [u8]>,
}

impl<'data> RawChunk<'data> {
    /// Creates a chunk from its tag and payload.
    pub fn new(tag: Tag, data: impl Into<Cow<'data, [u8]>>) -> Self {
        Self {
            tag,
            data: data.into(),
        }
    }
}

impl<'data> Chunk for RawChunk<'data> {
    fn tag(&self) -> Tag {
        self.tag
    }

    fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        output.write_bytes(&self.data);
        Ok(())
    }
}
