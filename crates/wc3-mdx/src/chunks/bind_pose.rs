//! Reforged bind-pose matrices in `BPOS` chunks.
use crate::{
    BindPoseMatrix, Chunk, CollectionChunk, Cursor, DecodeError, EncodeError, Encoder, KnownChunk,
    Tag,
};

const MATRIX_SIZE: usize = 48;

/// A `BPOS` chunk containing a counted collection of matrices.
#[derive(Clone, Debug, PartialEq)]
pub struct BindPoseChunk {
    pub records: Vec<BindPoseMatrix>,
}

impl BindPoseChunk {
    pub fn new(records: Vec<BindPoseMatrix>) -> Self {
        Self { records }
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl CollectionChunk for BindPoseChunk {
    type Item = BindPoseMatrix;

    fn tag() -> Tag {
        Self::TAG
    }

    fn records(&self) -> &[Self::Item] {
        &self.records
    }

    fn from_records(records: Vec<Self::Item>) -> Self {
        Self { records }
    }
}

impl Chunk for BindPoseChunk {
    fn tag(&self) -> Tag {
        Self::TAG
    }

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let size = self
            .records
            .len()
            .checked_mul(MATRIX_SIZE)
            .and_then(|n| n.checked_add(4))
            .ok_or(EncodeError::ChunkTooLarge {
                tag: Self::TAG,
                size: usize::MAX,
            })?;
        if size > u32::MAX as usize {
            return Err(EncodeError::ChunkTooLarge {
                tag: Self::TAG,
                size,
            });
        }
        bytes.write(self.records.len() as u32)?;
        for record in &self.records {
            bytes.write(record)?;
        }
        Ok(())
    }
}

impl KnownChunk for BindPoseChunk {
    const TAG: Tag = *b"BPOS";

    fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let size = cursor.remaining().len();
        let count = cursor
            .read::<u32>()
            .map_err(|_| DecodeError::MalformedChunk {
                tag: Self::TAG,
                size,
                expected: 4,
            })? as usize;
        let expected = count
            .checked_mul(MATRIX_SIZE)
            .and_then(|n| n.checked_add(4))
            .ok_or(DecodeError::MalformedRecord {
                tag: Self::TAG,
                offset: 0,
            })?;
        if size != expected {
            return Err(DecodeError::MalformedChunk {
                tag: Self::TAG,
                size,
                expected,
            });
        }
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            records.push(cursor.read()?);
        }
        Ok(Self { records })
    }
}
