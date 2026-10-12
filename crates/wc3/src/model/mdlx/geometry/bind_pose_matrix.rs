//! One matrix in a Reforged bind pose.
use crate::model::mdlx::ModelDialect;
use crate::model::mdlx::{mdl, mdx};
use crate::model::mdlx::{BindPoseChunk, Model};
use crate::model::mdlx::{SupportsReforgedChunks, ValueError};

/// A 3-by-4 floating-point bind-pose matrix.
#[derive(
    Clone, Copy, Debug, PartialEq, mdx::Read, mdx::ReadOwned, mdx::Write, mdl::Read, mdl::Write,
)]
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

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn try_bind_poses_mut<'s>(
        &'s mut self,
    ) -> Result<impl Iterator<Item = &'s mut BindPoseMatrix> + use<'data, 's, D>, ValueError> {
        self.check_chunk_version(*b"BPOS")?;
        Ok(self.records_mut::<BindPoseChunk>())
    }

    /// Replaces bind-pose matrices with one `BPOS` chunk, removing duplicate chunks.
    /// Returns an error if this model version does not support `BPOS`.
    pub fn try_set_bind_poses(
        &mut self,
        poses: impl IntoIterator<Item = BindPoseMatrix>,
    ) -> Result<(), ValueError> {
        self.check_chunk_version(*b"BPOS")?;
        self.replace_chunk(BindPoseChunk::new(poses.into_iter().collect()));
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

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn bind_poses_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut BindPoseMatrix> + use<'data, 's, D> {
        self.records_mut::<BindPoseChunk>()
    }

    /// Replaces `BPOS` records.
    pub fn set_bind_poses(&mut self, poses: impl IntoIterator<Item = BindPoseMatrix>) {
        self.replace_chunk(BindPoseChunk::new(poses.into_iter().collect()));
    }
}
