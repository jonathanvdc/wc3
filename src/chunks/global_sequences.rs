//! The complete global-sequences chunk.

use super::checked_chunk_size;
use crate::{Error, KnownChunk, Record};

/// The complete `GLBS` payload.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GlobalSequencesChunk {
    pub durations: Vec<u32>,
}

impl Record for GlobalSequencesChunk {
    fn decode_one(bytes: &[u8], _version: u32) -> Result<(Self, usize), Error> {
        let length = bytes.len();
        let value = {
            if bytes.len() % 4 != 0 {
                return Err(Error::MalformedChunk {
                    tag: Self::TAG,
                    size: bytes.len(),
                    expected: 4,
                });
            }
            Ok(Self {
                durations: bytes
                    .chunks_exact(4)
                    .map(|word| u32::from_le_bytes(word.try_into().expect("four-byte field")))
                    .collect(),
            })
        }?;
        Ok((value, length))
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
    const TAG: [u8; 4] = *b"GLBS";
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
