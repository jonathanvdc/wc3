//! Geoset whitelist entries for world ray picking, independent of collision shapes.
use crate::model::{mdl, mdx, GlidersChunk, Model, ModelVersion};

/// A geoset that a world-picking ray may hit.
///
/// This whitelist controls world ray picking independently of collision shapes.
/// Entries are supported in every model version and retain their order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(block = "Glider")]
pub struct Glider {
    #[mdl(property = "GeosetId")]
    /// Index into the model geoset collection.
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
            self.chunks.retain(|chunk| chunk.tag() != *b"DILG");
        } else {
            self.replace_chunk(GlidersChunk::new(gliders.to_vec()));
        }
    }
}
