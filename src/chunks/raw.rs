use super::Chunk;
use crate::{Encodable, Tag};

/// A tagged top-level MDX chunk. The payload is stored without interpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawChunk {
    /// Four-byte binary identifier, such as `VERS`, `MODL`, or `GEOS`.
    pub tag: Tag,
    /// Original payload bytes, preserved on round-trip.
    pub data: Vec<u8>,
}

impl RawChunk {
    /// Creates a chunk from its tag and payload.
    pub fn new(tag: Tag, data: Vec<u8>) -> Self {
        Self { tag, data }
    }
}

impl Encodable for RawChunk {
    fn encode_to(&self, output: &mut crate::Encoder<'_>) -> Result<(), crate::Error> {
        output.write_bytes(&self.data);
        Ok(())
    }
}

impl Chunk for RawChunk {
    fn tag(&self) -> Tag {
        self.tag
    }
}
