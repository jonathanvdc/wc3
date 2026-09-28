//! Model accessors for pivot points.

use crate::model::mdl;
use crate::model::mdx;
use crate::model::ModelVersion;
use crate::model::{Model, PivotPointsChunk, Vec3};

/// One model pivot point.
#[derive(Clone, Copy, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(entry)]
pub struct PivotPoint(pub Vec3);

impl<V: ModelVersion> Model<V> {
    /// Returns XYZ pivot points from every `PIVT` chunk in file order.
    pub fn pivot_points(&self) -> Vec<Vec3> {
        self.collect_chunk_records::<PivotPointsChunk>()
            .into_iter()
            .map(|point| point.0)
            .collect()
    }

    /// Writes XYZ pivot points to a `PIVT` chunk.
    pub fn set_pivot_points(&mut self, points: &[Vec3]) {
        self.replace_chunk(PivotPointsChunk::new(
            points.iter().copied().map(PivotPoint).collect(),
        ));
    }
}
