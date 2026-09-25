//! Reforged bind-pose matrices in `BPOS` chunks.

use crate::Record;
use crate::{Error, KnownChunk, Model};

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
        pose.encode()?;
        Ok(pose)
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
            .filter(|chunk| chunk.tag() == BindPose::TAG)
            .map(|chunk| match chunk {
                crate::ModelChunk::BindPose(decoded) => Ok(decoded.clone()),
                crate::ModelChunk::Malformed(malformed) => Err(malformed.error.clone()),
                crate::ModelChunk::Unknown(raw) => BindPose::decode(&raw.data, self.version()),
                _ => unreachable!("BPOS tag matched another typed chunk"),
            })
            .collect()
    }

    /// Replaces the first `BPOS` chunk or appends one. Other `BPOS` chunks
    /// remain intact.
    pub fn set_bind_pose(&mut self, pose: &BindPose) {
        if let Some(chunk) = self.chunk_mut(BindPose::TAG) {
            *chunk = crate::ModelChunk::BindPose(pose.clone());
        } else {
            self.push(crate::ModelChunk::BindPose(pose.clone()));
        }
    }
}

impl Record for BindPose {
    fn decode_one(cursor: &mut crate::Cursor<'_>, _version: u32) -> Result<Self, Error> {
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

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let count = u32::try_from(self.matrices.len()).map_err(|_| Error::ChunkTooLarge {
            tag: BindPose::TAG,
            size: self.matrices.len(),
        })?;
        let size = self
            .matrices
            .len()
            .checked_mul(MATRIX_SIZE)
            .and_then(|n| n.checked_add(4))
            .filter(|&size| size <= u32::MAX as usize)
            .ok_or(Error::ChunkTooLarge {
                tag: BindPose::TAG,
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
}

impl KnownChunk for BindPose {
    const TAG: [u8; 4] = *b"BPOS";
}
