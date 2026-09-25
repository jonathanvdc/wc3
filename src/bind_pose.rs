//! Reforged bind-pose matrices in `BPOS` chunks.

use crate::{Chunk, Error, Model};

const TAG: [u8; 4] = *b"BPOS";
const MATRIX_SIZE: usize = 48;

/// A `BPOS` payload containing 3-by-4 floating-point matrices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BindPose {
    bytes: Vec<u8>,
}

impl BindPose {
    /// Creates a bind pose from 12-float matrices.
    pub fn new(matrices: &[[f32; 12]]) -> Result<Self, Error> {
        if matrices.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            });
        }
        let size = matrices
            .len()
            .checked_mul(MATRIX_SIZE)
            .and_then(|n| n.checked_add(4))
            .filter(|&size| size <= u32::MAX as usize)
            .ok_or(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            })?;
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(&(matrices.len() as u32).to_le_bytes());
        for matrix in matrices {
            for value in matrix {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        Ok(Self { bytes })
    }

    /// Wraps one `BPOS` payload after checking its matrix count.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let count_bytes = bytes.get(..4).ok_or(Error::MalformedChunk {
            tag: TAG,
            size: bytes.len(),
            expected: 4,
        })?;
        let count = u32::from_le_bytes(count_bytes.try_into().expect("four-byte count")) as usize;
        let expected = count
            .checked_mul(MATRIX_SIZE)
            .and_then(|n| n.checked_add(4))
            .ok_or(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            })?;
        if bytes.len() != expected {
            return Err(Error::MalformedChunk {
                tag: TAG,
                size: bytes.len(),
                expected,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the original payload bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the number of matrices.
    pub fn len(&self) -> usize {
        (self.bytes.len() - 4) / MATRIX_SIZE
    }

    /// Returns whether this bind pose contains no matrices.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns one 3-by-4 matrix by index.
    pub fn matrix(&self, index: usize) -> Option<[f32; 12]> {
        if index >= self.len() {
            return None;
        }
        let start = 4 + index * MATRIX_SIZE;
        Some(std::array::from_fn(|coordinate| {
            let offset = start + coordinate * 4;
            f32::from_le_bytes(
                self.bytes[offset..offset + 4]
                    .try_into()
                    .expect("four-byte field"),
            )
        }))
    }

    /// Replaces one matrix, returning false for an out-of-range index.
    pub fn set_matrix(&mut self, index: usize, matrix: [f32; 12]) -> bool {
        if index >= self.len() {
            return false;
        }
        let start = 4 + index * MATRIX_SIZE;
        for (coordinate, value) in matrix.into_iter().enumerate() {
            let offset = start + coordinate * 4;
            self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        true
    }
}

impl Model {
    /// Decodes every `BPOS` chunk separately, preserving chunk boundaries.
    pub fn bind_poses(&self) -> Result<Vec<BindPose>, Error> {
        self.chunks()
            .iter()
            .filter(|chunk| chunk.tag == TAG)
            .map(|chunk| BindPose::from_bytes(&chunk.data))
            .collect()
    }

    /// Replaces the first `BPOS` chunk or appends one. Other `BPOS` chunks
    /// remain intact.
    pub fn set_bind_pose(&mut self, pose: &BindPose) {
        if let Some(chunk) = self.chunk_mut(TAG) {
            chunk.data = pose.as_bytes().to_vec();
        } else {
            self.push(Chunk::new(TAG, pose.as_bytes().to_vec()));
        }
    }
}
