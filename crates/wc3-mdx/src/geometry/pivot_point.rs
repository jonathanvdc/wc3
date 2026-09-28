//! Model accessors for pivot points.
use crate::mdl::{MdlRead, MdlWrite};
use crate::ModelVersion;
use crate::{Model, PivotPointsChunk, Readable, Vec3, Writable};

/// One model pivot point.
#[derive(Clone, Copy, Debug, PartialEq, Readable, Writable, MdlRead, MdlWrite)]
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
