//! Model accessors for scalar chunks.
use crate::Vec3;

use crate::{GlobalSequencesChunk, Model, ModelChunk, PivotPointsChunk};

impl Model {
    /// Returns durations from every `GLBS` chunk in file order.
    pub fn global_sequences(&self) -> Vec<u32> {
        self.collect_chunk_items::<GlobalSequencesChunk, _>(|chunk| &chunk.durations)
    }

    /// Writes global sequence durations to a `GLBS` chunk.
    pub fn set_global_sequences(&mut self, durations: &[u32]) {
        self.replace_chunk(ModelChunk::GlobalSequences(GlobalSequencesChunk {
            durations: durations.to_vec(),
        }));
    }

    /// Returns XYZ pivot points from every `PIVT` chunk in file order.
    pub fn pivot_points(&self) -> Vec<Vec3> {
        self.collect_chunk_items::<PivotPointsChunk, _>(|chunk| &chunk.points)
    }

    /// Writes XYZ pivot points to a `PIVT` chunk.
    pub fn set_pivot_points(&mut self, points: &[Vec3]) {
        self.replace_chunk(ModelChunk::PivotPoints(PivotPointsChunk {
            points: points.to_vec(),
        }));
    }
}
