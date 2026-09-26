//! The complete global-sequences chunk.
use crate::Encoder;
use crate::Tag;

use super::checked_chunk_size;
use crate::Cursor;
use crate::{Chunk, Error, KnownChunk};

/// The complete `GLBS` chunk.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GlobalSequencesChunk {
    pub durations: Vec<u32>,
}

impl Chunk for GlobalSequencesChunk {
    fn tag(&self) -> Tag {
        Self::TAG
    }

    fn encode_payload_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        checked_chunk_size(self.durations.len(), 4, Self::TAG)?;

        for duration in &self.durations {
            bytes.write(duration);
        }
        Ok(())
    }
}

impl KnownChunk for GlobalSequencesChunk {
    fn decode_payload(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let size = cursor.remaining().len();
        if size % 4 != 0 {
            return Err(Error::MalformedChunk {
                tag: Self::TAG,
                size,
                expected: 4,
            });
        }
        let mut durations = Vec::new();
        while !cursor.remaining().is_empty() {
            durations.push(cursor.read_u32()?);
        }
        Ok(Self { durations })
    }

    const TAG: Tag = *b"GLBS";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decodable, Encodable};

    #[test]
    fn round_trips_entire_payload() {
        let original = GlobalSequencesChunk {
            durations: vec![100, 200, 300],
        };
        let payload = original.encode().unwrap();
        assert_eq!(
            GlobalSequencesChunk::decode(&payload, 800).unwrap(),
            original
        );
        assert!(GlobalSequencesChunk::decode(&[1], 800).is_err());
    }
}
