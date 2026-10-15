//! Geoset whitelist entries for world ray picking, independent of collision shapes.
use crate::model::formats::mdlx::Dialect;
use crate::model::formats::mdlx::{mdl, mdx, Document, GlidersChunk};

/// A geoset that a world-picking ray may hit.
///
/// This whitelist controls world ray picking independently of collision shapes.
/// Entries are supported in every model version and retain their order.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, mdx::Read, mdx::ReadOwned, mdx::Write, mdl::Read, mdl::Write,
)]
#[mdl(block = "Glider")]
pub struct Glider {
    #[mdl(property = "GeosetId")]
    /// Index into the model geoset collection.
    pub geoset_id: u32,
}

impl<'data, D: Dialect> Document<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn gliders<'s>(&'s self) -> impl Iterator<Item = &'s Glider> + use<'data, 's, D> {
        self.records::<GlidersChunk>()
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn gliders_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut Glider> + use<'data, 's, D> {
        self.records_mut::<GlidersChunk>()
    }

    /// Replaces the whitelist, removing all DILG chunks when empty.
    pub fn set_gliders(&mut self, gliders: impl IntoIterator<Item = Glider>) {
        let gliders: Vec<_> = gliders.into_iter().collect();
        if gliders.is_empty() {
            self.chunks
                .retain(|chunk| chunk.standard_tag() != Some(*b"DILG"));
        } else {
            self.replace_chunk(GlidersChunk::new(gliders));
        }
    }
}
