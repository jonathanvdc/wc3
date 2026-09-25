//! Semantic checks for all currently known MDX chunk layouts.

use crate::{Encoder, Error, Model, ModelChunk};

impl Model {
    /// Validates every known chunk against the model's current version.
    /// Unknown chunks remain valid and round-trip unchanged.
    pub fn validate(&self) -> Result<(), Error> {
        for chunk in self.chunks() {
            if let ModelChunk::Malformed(malformed) = chunk {
                return Err(malformed.error().clone());
            }
            let mut bytes = Vec::new();
            chunk.encode_to(&mut Encoder::new(&mut bytes))?;
            let tag = chunk.tag();
            let data = bytes[8..].to_vec();
            if tag == *b"VERS" && data.len() < 4 {
                return Err(Error::InvalidVersionChunk);
            }
            if let ModelChunk::Malformed(malformed) =
                ModelChunk::from_raw(crate::RawChunk::new(tag, data), self.version())
            {
                return Err(malformed.error);
            }
        }
        Ok(())
    }
}
