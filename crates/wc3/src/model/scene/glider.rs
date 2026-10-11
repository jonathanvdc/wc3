//! Geoset whitelist entries for world ray picking, independent of collision shapes.
use crate::model::ModelDialect;
use crate::model::{mdl, mdx, GlidersChunk, Model};

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

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn gliders<'s>(&'s self) -> impl Iterator<Item = &'s Glider> + use<'data, 's, D> {
        self.records::<GlidersChunk>()
    }

    /// Replaces the whitelist, removing all DILG chunks when empty.
    pub fn set_gliders(&mut self, gliders: &[Glider]) {
        if gliders.is_empty() {
            self.chunks
                .retain(|chunk| chunk.standard_tag() != Some(*b"DILG"));
        } else {
            self.replace_chunk(GlidersChunk::new(gliders.to_vec()));
        }
    }
}
