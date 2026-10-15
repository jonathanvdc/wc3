//! Node transform origins, indexed by object ID.

use crate::model::formats::mdlx::mdl;
use crate::model::formats::mdlx::mdx;
use crate::model::formats::mdlx::Dialect;
use crate::model::formats::mdlx::{Document, PivotPointsChunk, Vec3};

/// Transform origin for the node whose object ID matches this point's index.
#[derive(
    Clone, Copy, Debug, PartialEq, mdx::Read, mdx::ReadOwned, mdx::Write, mdl::Read, mdl::Write,
)]
#[mdl(entry)]
pub struct PivotPoint(
    /// XYZ transform origin in model coordinates.
    pub Vec3,
);

impl<'data, D: Dialect> Document<'data, D> {
    /// Returns XYZ pivot points from every `PIVT` chunk in file order.
    pub fn pivot_points(&self) -> Vec<Vec3> {
        self.records::<PivotPointsChunk>()
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
