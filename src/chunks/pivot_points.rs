//! The complete pivot-points chunk.
use crate::Encoder;
use crate::{Tag, Vec3};

use super::checked_chunk_size;
use crate::Cursor;
use crate::{Chunk, Error, KnownChunk};

/// The complete `PIVT` chunk.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PivotPointsChunk {
    pub points: Vec<Vec3>,
}

impl Chunk for PivotPointsChunk {
    fn tag(&self) -> Tag {
        Self::TAG
    }

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        checked_chunk_size(self.points.len(), 12, Self::TAG)?;

        for point in &self.points {
            for coordinate in point {
                bytes.write(coordinate);
            }
        }
        Ok(())
    }
}

impl KnownChunk for PivotPointsChunk {
    fn decode_payload(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
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
            points.push(cursor.read_vec3()?);
        }
        Ok(Self { points })
    }

    const TAG: Tag = *b"PIVT";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decodable, Encodable};

    #[test]
    fn round_trips_entire_payload() {
        let original = PivotPointsChunk {
            points: vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
        };
        let payload = original.encode().unwrap();
        assert_eq!(PivotPointsChunk::decode(&payload, 800).unwrap(), original);
        assert!(PivotPointsChunk::decode(&[1], 800).is_err());
    }
}
