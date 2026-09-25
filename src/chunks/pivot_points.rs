//! The complete pivot-points chunk.

use super::checked_chunk_size;
use crate::{Error, KnownChunk, Record};

/// The complete `PIVT` payload.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PivotPointsChunk {
    pub points: Vec<[f32; 3]>,
}

impl Record for PivotPointsChunk {
    fn decode_one(cursor: &mut crate::Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let size = cursor.remaining().len();
        if size % 12 != 0 {
            return Err(Error::MalformedChunk {
                tag: Self::TAG,
                size,
                expected: 12,
            });
        }
        let mut points = Vec::new();
        while !cursor.remaining().is_empty() {
            points.push([cursor.read_f32()?, cursor.read_f32()?, cursor.read_f32()?]);
        }
        Ok(Self { points })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let size = checked_chunk_size(self.points.len(), 12, Self::TAG)?;
        let mut bytes = Vec::with_capacity(size);
        for point in &self.points {
            for coordinate in point {
                bytes.extend_from_slice(&coordinate.to_le_bytes());
            }
        }
        Ok(bytes)
    }
}

impl KnownChunk for PivotPointsChunk {
    const TAG: [u8; 4] = *b"PIVT";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chunk;

    #[test]
    fn round_trips_entire_payload() {
        let original = PivotPointsChunk {
            points: vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        };
        let raw = original.encode_chunk().unwrap();
        assert_eq!(PivotPointsChunk::decode_chunk(&raw, 800).unwrap(), original);
        assert!(PivotPointsChunk::decode(&[1], 800).is_err());
    }
}
