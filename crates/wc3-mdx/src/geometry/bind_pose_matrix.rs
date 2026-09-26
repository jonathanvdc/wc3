//! One matrix in a Reforged bind pose.
use crate::{
    BindPoseChunk, Cursor, Decodable, DecodeError, Encodable, EncodeError, Encoder, Model, Version,
};

/// A 3-by-4 floating-point bind-pose matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BindPoseMatrix(pub [f32; 12]);

impl Encodable for BindPoseMatrix {
    fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        output.write(self.0);
        Ok(())
    }
}

impl Decodable for BindPoseMatrix {
    fn decode_one(cursor: &mut Cursor<'_>, _version: Version) -> Result<Self, DecodeError> {
        Ok(Self(cursor.read()?))
    }
}

impl Model {
    /// Returns decoded `BPOS` chunks separately, preserving chunk boundaries.
    pub fn bind_poses(&self) -> Vec<BindPoseChunk> {
        self.decoded_chunks::<BindPoseChunk>().cloned().collect()
    }

    /// Replaces all `BPOS` chunks with one decoded chunk at the first one's position.
    pub fn set_bind_pose(&mut self, pose: &BindPoseChunk) {
        self.replace_chunk(pose.clone());
    }
}
