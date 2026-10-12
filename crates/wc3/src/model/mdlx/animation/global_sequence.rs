//! Loop durations for animations that run independently of model sequences.

use crate::model::mdlx::mdl;
use crate::model::mdlx::mdx;
use crate::model::mdlx::ModelDialect;
use crate::model::mdlx::{GlobalSequencesChunk, Model};

/// One global sequence duration in milliseconds.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, mdx::Read, mdx::ReadOwned, mdx::Write, mdl::Read, mdl::Write,
)]
#[mdl(property = "Duration")]
pub struct GlobalSequence(
    /// Loop duration in milliseconds.
    pub u32,
);

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Returns loop durations in milliseconds, in model order.
    /// Tracks refer to these durations by collection index.
    pub fn global_sequences(&self) -> Vec<u32> {
        self.records::<GlobalSequencesChunk>()
            .map(|sequence| sequence.0)
            .collect()
    }

    /// Replaces global sequence durations. Track indices are not adjusted.
    pub fn set_global_sequences(&mut self, durations: &[u32]) {
        self.replace_chunk(GlobalSequencesChunk::new(
            durations.iter().copied().map(GlobalSequence).collect(),
        ));
    }
}
