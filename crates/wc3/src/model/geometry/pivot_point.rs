//! Node transform origins, indexed by object ID.

use crate::model::mdl;
use crate::model::mdx;
use crate::model::ModelDialect;
use crate::model::{Model, PivotPointsChunk, Vec3};

/// Transform origin for the node whose object ID matches this point's index.
#[derive(Clone, Copy, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdl(entry)]
pub struct PivotPoint(
    /// XYZ transform origin in model coordinates.
    pub Vec3,
);

impl<D: ModelDialect> Model<D> {
    /// Returns XYZ pivot points from every `PIVT` chunk in file order.
    pub fn pivot_points(&self) -> Vec<Vec3> {
        self.collect_chunk_records::<PivotPointsChunk>()
            .into_iter()
            .map(|point| point.0)
            .collect()
    }

    /// Replaces pivot points. Point indices must match the corresponding node object IDs.
    pub fn set_pivot_points(&mut self, points: &[Vec3]) {
        self.replace_chunk(PivotPointsChunk::new(
            points.iter().copied().map(PivotPoint).collect(),
        ));
    }
}
