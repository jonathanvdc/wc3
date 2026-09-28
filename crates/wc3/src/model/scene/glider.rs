//! Geoset whitelist entries for world ray picking, independent of collision shapes.
use crate::model::{mdx, GlidersChunk, Model, ModelVersion};

/// A geoset that a world-picking ray may hit.
///
/// `DILG` has no version gate. Entries retain their order, including duplicates;
/// the library does not emulate the client's defect that overwrites slot zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq, mdx::Read, mdx::Write)]
pub struct Glider {
    pub geoset_id: u32,
}

impl<V: ModelVersion> Model<V> {
    /// Returns world-picking whitelist entries in chunk and record order.
    pub fn gliders(&self) -> Vec<Glider> {
        self.collect_chunk_records::<GlidersChunk>()
    }

    /// Replaces the whitelist, removing all DILG chunks when empty.
    pub fn set_gliders(&mut self, gliders: &[Glider]) {
        if gliders.is_empty() {
            self.chunks_mut().retain(|chunk| chunk.tag() != *b"DILG");
        } else {
            self.replace_chunk(GlidersChunk::new(gliders.to_vec()));
        }
    }
}
