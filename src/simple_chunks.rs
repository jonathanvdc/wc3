//! Model accessors for scalar chunks.

use crate::{Error, GlobalSequencesChunk, Model, ModelChunk, PivotPointsChunk};

impl Model {
    /// Returns durations from every `GLBS` chunk in file order.
    pub fn global_sequences(&self) -> Result<Vec<u32>, Error> {
        self.collect_chunk_items(*b"GLBS", |chunk| match chunk {
            ModelChunk::GlobalSequences(decoded) => Some(&decoded.durations),
            _ => None,
        })
    }

    /// Writes global sequence durations to a `GLBS` chunk.
    pub fn set_global_sequences(&mut self, durations: &[u32]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::GlobalSequences(GlobalSequencesChunk {
            durations: durations.to_vec(),
        }))
    }

    /// Returns XYZ pivot points from every `PIVT` chunk in file order.
    pub fn pivot_points(&self) -> Result<Vec<[f32; 3]>, Error> {
        self.collect_chunk_items(*b"PIVT", |chunk| match chunk {
            ModelChunk::PivotPoints(decoded) => Some(&decoded.points),
            _ => None,
        })
    }

    /// Writes XYZ pivot points to a `PIVT` chunk.
    pub fn set_pivot_points(&mut self, points: &[[f32; 3]]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::PivotPoints(PivotPointsChunk {
            points: points.to_vec(),
        }))
    }

    pub(crate) fn replace_chunk(&mut self, chunk: ModelChunk) -> Result<(), Error> {
        let tag = chunk.tag();
        if let Some(index) = self
            .chunks()
            .iter()
            .position(|existing| existing.tag() == tag)
        {
            self.chunks_mut()[index] = chunk;
            let mut seen = false;
            self.chunks_mut().retain(|existing| {
                if existing.tag() != tag {
                    return true;
                }
                if seen {
                    false
                } else {
                    seen = true;
                    true
                }
            });
        } else {
            self.push(chunk);
        }
        Ok(())
    }
}
