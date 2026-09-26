//! Reforged bind-pose matrices in `BPOS` chunks.
use crate::Encoder;
use crate::Tag;

use crate::Record;
use crate::{Cursor, ModelChunk};
use crate::{Error, KnownChunk, Model};

const MATRIX_SIZE: usize = 48;

/// A `BPOS` payload containing decoded 3-by-4 floating-point matrices.
#[derive(Clone, Debug, PartialEq)]
pub struct BindPose {
    matrices: Vec<[f32; 12]>,
}

impl BindPose {
    /// Creates a bind pose from 12-float matrices.
    pub fn new(matrices: &[[f32; 12]]) -> Self {
        Self {
            matrices: matrices.to_vec(),
        }
    }

    /// Returns the number of matrices.
    pub fn len(&self) -> usize {
        self.matrices.len()
    }

    /// Returns whether this bind pose contains no matrices.
    pub fn is_empty(&self) -> bool {
        self.matrices.is_empty()
    }

    /// Borrows all decoded matrices.
    pub fn matrices(&self) -> &[[f32; 12]] {
        &self.matrices
    }

    /// Borrows decoded matrices for bulk editing.
    pub fn matrices_mut(&mut self) -> &mut [[f32; 12]] {
        &mut self.matrices
    }

    /// Returns one 3-by-4 matrix by index.
    pub fn matrix(&self, index: usize) -> Option<[f32; 12]> {
        self.matrices.get(index).copied()
    }

    /// Replaces one matrix, returning false for an out-of-range index.
    pub fn set_matrix(&mut self, index: usize, matrix: [f32; 12]) -> bool {
        if let Some(slot) = self.matrices.get_mut(index) {
            *slot = matrix;
            true
        } else {
            false
        }
    }
}

impl Model {
    /// Returns decoded `BPOS` chunks separately, preserving chunk boundaries.
    pub fn bind_poses(&self) -> Vec<BindPose> {
        self.decoded_chunks(|chunk| match chunk {
            ModelChunk::BindPose(decoded) => Some(decoded),
            _ => None,
        })
        .cloned()
        .collect()
    }

    /// Replaces all `BPOS` chunks with one decoded chunk at the first one's
    /// position, or appends one if none exists.
    pub fn set_bind_pose(&mut self, pose: &BindPose) {
        self.replace_chunk(ModelChunk::BindPose(pose.clone()));
    }
}

impl Record for BindPose {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let size = cursor.remaining().len();
        let count = cursor.read_u32().map_err(|_| Error::MalformedChunk {
            tag: Self::TAG,
            size,
            expected: 4,
        })? as usize;
        let body_size = count
            .checked_mul(MATRIX_SIZE)
            .ok_or(Error::MalformedRecord {
                tag: Self::TAG,
                offset: 0,
            })?;
        let expected = body_size.checked_add(4).ok_or(Error::MalformedRecord {
            tag: Self::TAG,
            offset: 0,
        })?;
        if size != expected {
            return Err(Error::MalformedChunk {
                tag: Self::TAG,
                size,
                expected,
            });
        }
        let mut matrices = Vec::new();
        for _ in 0..count {
            matrices.push(std::array::from_fn(|_| {
                cursor.read_f32().expect("validated matrix length")
            }));
        }
        Ok(Self { matrices })
    }

    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        self.matrices
            .len()
            .checked_mul(MATRIX_SIZE)
            .and_then(|n| n.checked_add(4))
            .filter(|&size| size <= u32::MAX as usize)
            .ok_or(Error::ChunkTooLarge {
                tag: BindPose::TAG,
                size: usize::MAX,
            })?;

        bytes.write(self.matrices.len() as u32);
        for matrix in &self.matrices {
            for value in matrix {
                bytes.write(value);
            }
        }
        Ok(())
    }
}

impl KnownChunk for BindPose {
    const TAG: Tag = *b"BPOS";
}
