//! One matrix in a Reforged bind pose.
use crate::ModelVersion;
use crate::{BindPoseChunk, Model, Readable, Writable};

/// A 3-by-4 floating-point bind-pose matrix.
#[derive(Clone, Copy, Debug, PartialEq, Readable, Writable)]
pub struct BindPoseMatrix(pub [f32; 12]);

impl<V: ModelVersion> Model<V> {
    /// Returns decoded `BPOS` chunks separately, preserving chunk boundaries.
    pub fn bind_poses(&self) -> Vec<BindPoseChunk> {
        self.decoded_chunks::<BindPoseChunk>().cloned().collect()
    }

    /// Replaces all `BPOS` chunks with one decoded chunk at the first one's position.
    pub fn set_bind_pose(&mut self, pose: &BindPoseChunk) {
        self.replace_chunk(pose.clone());
    }
}
