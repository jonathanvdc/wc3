//! Model accessors for global sequences.

use crate::model::mdl;
use crate::model::mdx;
use crate::model::ModelVersion;
use crate::model::{GlobalSequencesChunk, Model};

/// One global sequence duration in milliseconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(property = "Duration")]
pub struct GlobalSequence(pub u32);

impl<V: ModelVersion> Model<V> {
    /// Returns durations from every `GLBS` chunk in file order.
    pub fn global_sequences(&self) -> Vec<u32> {
        self.collect_chunk_records::<GlobalSequencesChunk>()
            .into_iter()
            .map(|sequence| sequence.0)
            .collect()
    }

    /// Writes global sequence durations to a `GLBS` chunk.
    pub fn set_global_sequences(&mut self, durations: &[u32]) {
        self.replace_chunk(GlobalSequencesChunk::new(
            durations.iter().copied().map(GlobalSequence).collect(),
        ));
    }
}
