//! One matrix in a Reforged bind pose.
use crate::model::{mdl, mdx};
use crate::model::{BindPoseChunk, Model};
use crate::model::{ModelVersion, SupportsReforgedChunks, ValueError};

/// A 3-by-4 floating-point bind-pose matrix.
#[derive(Clone, Copy, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(entry)]
pub struct BindPoseMatrix(pub [f32; 12]);

impl<V: ModelVersion> Model<V> {
    /// Returns an error if this model version does not support `BPOS`.
    pub fn try_bind_poses(&self) -> Result<Vec<BindPoseMatrix>, ValueError> {
        self.check_chunk_version(*b"BPOS")?;
        Ok(self
            .decoded_chunks::<BindPoseChunk>()
            .flat_map(|chunk| chunk.records.iter())
            .copied()
            .collect())
    }

    /// Returns an error if this model version does not support `BPOS`.
    pub fn try_set_bind_poses(&mut self, poses: &[BindPoseMatrix]) -> Result<(), ValueError> {
        self.check_chunk_version(*b"BPOS")?;
        self.replace_chunk(BindPoseChunk::new(poses.to_vec()));
        Ok(())
    }
}

impl<V: SupportsReforgedChunks> Model<V> {
    /// Returns owned bind-pose matrices in model order.
    pub fn bind_poses(&self) -> Vec<BindPoseMatrix> {
        self.decoded_chunks::<BindPoseChunk>()
            .flat_map(|chunk| chunk.records.iter())
            .copied()
            .collect()
    }

    /// Replaces `BPOS` records.
    pub fn set_bind_poses(&mut self, poses: &[BindPoseMatrix]) {
        self.replace_chunk(BindPoseChunk::new(poses.to_vec()));
    }
}
