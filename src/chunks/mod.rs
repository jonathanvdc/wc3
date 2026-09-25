//! Raw and typed top-level MDX chunks.

use crate::{Error, Record};

mod raw;
pub use raw::RawChunk;
mod bind_pose;
pub use bind_pose::BindPose;
mod collections;
pub use collections::*;
mod global_sequences;
pub use global_sequences::GlobalSequencesChunk;
mod model_info;
pub use model_info::ModelInfoChunk;
mod pivot_points;
pub use pivot_points::PivotPointsChunk;
mod version;
pub use version::VersionChunk;

/// A value that represents a complete top-level chunk.
pub trait Chunk {
    /// Returns this value's chunk tag.
    fn tag(&self) -> [u8; 4];

    /// Converts this value to a raw chunk.
    fn encode_chunk(&self) -> Result<RawChunk, Error>;
}

/// A complete chunk whose tag is fixed by its type.
pub trait KnownChunk: Chunk + Record {
    /// The four-byte chunk tag for this type.
    const TAG: [u8; 4];

    /// Decodes the complete payload after checking its tag.
    fn decode_chunk(chunk: &RawChunk, version: u32) -> Result<Self, Error> {
        if chunk.tag != Self::TAG {
            return Err(Error::MalformedRecord {
                tag: chunk.tag,
                offset: 0,
            });
        }
        Self::decode(&chunk.data, version)
    }
}

impl<T: KnownChunk> Chunk for T {
    fn tag(&self) -> [u8; 4] {
        T::TAG
    }

    fn encode_chunk(&self) -> Result<RawChunk, Error> {
        Ok(RawChunk::new(T::TAG, self.encode()?))
    }
}

fn checked_chunk_size(count: usize, width: usize, tag: [u8; 4]) -> Result<usize, Error> {
    let size = count.checked_mul(width).ok_or(Error::ChunkTooLarge {
        tag,
        size: usize::MAX,
    })?;
    if size > u32::MAX as usize {
        return Err(Error::ChunkTooLarge { tag, size });
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip<C: Chunk>(chunk: &C) -> RawChunk {
        let raw = chunk.encode_chunk().unwrap();
        assert_eq!(chunk.tag(), raw.tag);
        raw
    }

    #[test]
    fn raw_and_known_chunks_share_the_chunk_interface() {
        let raw = RawChunk::new(*b"FUTR", vec![1, 2, 3]);
        assert_eq!(round_trip(&raw), raw);

        let known = VersionChunk::new(800);
        let encoded = round_trip(&known);
        assert_eq!(VersionChunk::decode_chunk(&encoded, 800).unwrap(), known);
        assert!(VersionChunk::decode_chunk(&raw, 800).is_err());
    }
}
