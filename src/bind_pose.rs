//! Reforged bind-pose matrices in `BPOS` chunks.

use crate::{Chunk, Error, Model};

const TAG: [u8; 4] = *b"BPOS";
const MATRIX_SIZE: usize = 48;

/// A `BPOS` payload containing decoded 3-by-4 floating-point matrices.
#[derive(Clone, Debug, PartialEq)]
pub struct BindPose {
    matrices: Vec<[f32; 12]>,
}

impl BindPose {
    /// Creates a bind pose from 12-float matrices.
    pub fn new(matrices: &[[f32; 12]]) -> Result<Self, Error> {
        let pose = Self {
            matrices: matrices.to_vec(),
        };
        pose.to_bytes()?;
        Ok(pose)
    }

    /// Parses one `BPOS` payload after checking its matrix count.
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
        let matrices = bytes[4..]
            .chunks_exact(MATRIX_SIZE)
            .map(|matrix| {
                std::array::from_fn(|coordinate| {
                    let offset = coordinate * 4;
                    f32::from_le_bytes(
                        matrix[offset..offset + 4]
                            .try_into()
                            .expect("four-byte field"),
                    )
                })
            })
            .collect();
        Ok(Self { matrices })
    }

    /// Serializes the `BPOS` payload.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let count = u32::try_from(self.matrices.len()).map_err(|_| Error::ChunkTooLarge {
            tag: TAG,
            size: self.matrices.len(),
        })?;
        let size = self
            .matrices
            .len()
            .checked_mul(MATRIX_SIZE)
            .and_then(|n| n.checked_add(4))
            .filter(|&size| size <= u32::MAX as usize)
            .ok_or(Error::ChunkTooLarge {
                tag: TAG,
                size: usize::MAX,
            })?;
        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(&count.to_le_bytes());
        for matrix in &self.matrices {
            for value in matrix {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        Ok(bytes)
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
            chunk.data = pose.to_bytes().expect("validated bind pose");
        } else {
            self.push(Chunk::new(
                TAG,
                pose.to_bytes().expect("validated bind pose"),
            ));
        }
    }
}
