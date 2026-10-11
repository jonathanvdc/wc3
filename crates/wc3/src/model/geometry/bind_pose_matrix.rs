//! One matrix in a Reforged bind pose.
use crate::model::ModelDialect;
use crate::model::{mdl, mdx};
use crate::model::{BindPoseChunk, Model};
use crate::model::{SupportsReforgedChunks, ValueError};

/// A 3-by-4 floating-point bind-pose matrix.
#[derive(Clone, Copy, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(entry)]
pub struct BindPoseMatrix(
    /// The twelve matrix components in serialized order.
    pub [f32; 12],
);

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Borrows bind-pose records in file order, or reports unsupported versions.
    pub fn try_bind_poses<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s BindPoseMatrix> + use<'data, 's, D>, ValueError> {
        self.check_chunk_version(*b"BPOS")?;
        Ok(self.records::<BindPoseChunk>())
    }

    /// Replaces bind-pose matrices with one `BPOS` chunk, removing duplicate chunks.
    /// Returns an error if this model version does not support `BPOS`.
    pub fn try_set_bind_poses(&mut self, poses: &[BindPoseMatrix]) -> Result<(), ValueError> {
        self.check_chunk_version(*b"BPOS")?;
        self.replace_chunk(BindPoseChunk::new(poses.to_vec()));
        Ok(())
    }
}

impl<'data, D: ModelDialect> Model<'data, D>
where
    D::Version: SupportsReforgedChunks,
{
    /// Borrows bind-pose records in file order.
    pub fn bind_poses<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'s BindPoseMatrix> + use<'data, 's, D> {
        self.records::<BindPoseChunk>()
    }

    /// Replaces `BPOS` records.
    pub fn set_bind_poses(&mut self, poses: &[BindPoseMatrix]) {
        self.replace_chunk(BindPoseChunk::new(poses.to_vec()));
    }
}
