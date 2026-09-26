//! Semantic checks for all currently known MDX chunk layouts.

use crate::{Encoder, Error, Model, ModelChunk};

impl Model {
    /// Validates decoded chunks against the model's current version.
    /// Unknown and malformed chunks are ignored.
    pub fn validate(&self) -> Result<(), Error> {
        for chunk in self.chunks() {
            if matches!(chunk, ModelChunk::Unknown(_) | ModelChunk::Malformed(_)) {
                continue;
            }
            let mut bytes = Vec::new();
            chunk.encode_to(&mut Encoder::new(&mut bytes))?;
            let tag = chunk.tag();
            let data = bytes[8..].to_vec();
            if let ModelChunk::Malformed(malformed) =
                ModelChunk::from_raw(crate::RawChunk::new(tag, data), self.version())
            {
                return Err(malformed.error);
            }
        }
        Ok(())
    }
}
