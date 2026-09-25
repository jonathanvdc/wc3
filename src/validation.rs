//! Semantic checks for all currently known MDX chunk layouts.

use crate::{Error, Model, ModelChunk};

impl Model {
    /// Validates every known chunk against the model's current version.
    /// Unknown chunks remain valid and round-trip unchanged.
    pub fn validate(&self) -> Result<(), Error> {
        for chunk in self.chunks() {
            if let ModelChunk::Malformed(malformed) = chunk {
                return Err(malformed.error().clone());
            }
            let raw = chunk.to_raw()?;
            if raw.tag == *b"VERS" && raw.data.len() < 4 {
                return Err(Error::InvalidVersionChunk);
            }
            if let ModelChunk::Malformed(malformed) = ModelChunk::from_raw(raw, self.version()) {
                return Err(malformed.error);
            }
        }
        Ok(())
    }
}
