//! Model accessors for pivot points.
use crate::ModelVersion;
use crate::{
    Cursor, DecodeError, Encodable, EncodeError, Encoder, Model, PivotPointsChunk, Readable, Vec3,
};

/// One model pivot point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PivotPoint(pub Vec3);

impl Encodable for PivotPoint {
    fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        output.write(self.0);
        Ok(())
    }
}

impl Readable for PivotPoint {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        Ok(Self(cursor.read()?))
    }
}

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
