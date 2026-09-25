//! The complete global-sequences chunk.
use crate::Tag;

use super::checked_chunk_size;
use crate::Cursor;
use crate::{Error, KnownChunk, Record};

/// The complete `GLBS` payload.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GlobalSequencesChunk {
    pub durations: Vec<u32>,
}

impl Record for GlobalSequencesChunk {
    fn decode_one(cursor: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
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

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let size = checked_chunk_size(self.durations.len(), 4, Self::TAG)?;
        let mut bytes = Vec::with_capacity(size);
        for duration in &self.durations {
            bytes.extend_from_slice(&duration.to_le_bytes());
        }
        Ok(bytes)
    }
}

impl KnownChunk for GlobalSequencesChunk {
    const TAG: Tag = *b"GLBS";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Chunk;

    #[test]
    fn round_trips_entire_payload() {
        let original = GlobalSequencesChunk {
            durations: vec![100, 200, 300],
        };
        let raw = original.encode_chunk().unwrap();
        assert_eq!(
            GlobalSequencesChunk::decode_chunk(&raw, 800).unwrap(),
            original
        );
        assert!(GlobalSequencesChunk::decode(&[1], 800).is_err());
    }
}
