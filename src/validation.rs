//! Errors retained while decoding MDX chunks.

use crate::{Error, Model, ModelChunk};

impl Model {
    /// Returns the first error retained while decoding a malformed chunk.
    pub fn validate(&self) -> Result<(), Error> {
        for chunk in self.chunks() {
            if let ModelChunk::Malformed(malformed) = chunk {
                return Err(malformed.error().clone());
            }
        }
        Ok(())
    }
}
