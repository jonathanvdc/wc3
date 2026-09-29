//! Reforged bind-pose matrices in `BPOS` chunks.
use crate::model::mdx;
use crate::model::{mdl, BindPoseMatrix, Chunk, CollectionChunk, Cursor, Encoder, KnownChunk, Tag};

const MATRIX_SIZE: usize = 48;

/// A `BPOS` chunk containing a counted collection of matrices.
#[derive(Clone, Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(block = "BindPose")]
pub struct BindPoseChunk {
    #[mdl(counted = "Matrices")]
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

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        let size = self
            .records
            .len()
            .checked_mul(MATRIX_SIZE)
            .and_then(|n| n.checked_add(4))
            .ok_or(mdx::WriteError::SizeOverflow {
                field: "encoded size",
                tag: Self::TAG,
                size: usize::MAX,
            })?;
        if size > u32::MAX as usize {
            return Err(mdx::WriteError::SizeOverflow {
                field: "encoded size",
                tag: Self::TAG,
                size,
            });
        }
        bytes.write(&(self.records.len() as u32))?;
        for record in &self.records {
            bytes.write(record)?;
        }
        Ok(())
    }
}

impl KnownChunk for BindPoseChunk {
    const TAG: Tag = *b"BPOS";

    fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, mdx::ReadError> {
        let offset = cursor.absolute_position();
        let size = cursor.remaining().len();
        let count = cursor
            .read::<u32>()
            .map_err(|error| error.with_tag(Self::TAG))? as usize;
        let expected = count
            .checked_mul(MATRIX_SIZE)
            .and_then(|n| n.checked_add(4))
            .ok_or(
                mdx::ReadError::new(offset, mdx::ReadErrorKind::SizeOverflow).with_tag(Self::TAG),
            )?;
        if size != expected {
            return Err(mdx::ReadError::new(
                offset,
                mdx::ReadErrorKind::SizeMismatch {
                    actual: size,
                    expected,
                },
            )
            .with_tag(Self::TAG));
        }
        let mut records = Vec::with_capacity(count);
        for _ in 0..count {
            records.push(cursor.read()?);
        }
        Ok(Self { records })
    }
}
