//! Raw and typed top-level MDX chunks.
use crate::{Encodable, Tag};

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
mod model_chunk;
pub use model_chunk::{MalformedChunk, ModelChunk};
mod pivot_points;
pub use pivot_points::PivotPointsChunk;
mod version;
pub use version::VersionChunk;

/// A value that represents a complete top-level chunk.
pub trait Chunk: Encodable {
    /// Returns this value's chunk tag.
    fn tag(&self) -> Tag;
}

/// A complete chunk whose tag is fixed by its type.
pub trait KnownChunk: Chunk + Record {
    /// The four-byte chunk tag for this type.
    const TAG: Tag;
}

impl<T: KnownChunk> Chunk for T {
    fn tag(&self) -> Tag {
        T::TAG
    }
}

fn checked_chunk_size(count: usize, width: usize, tag: Tag) -> Result<usize, Error> {
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
    use crate::{Decodable, Encodable};

    #[test]
    fn raw_and_known_chunks_expose_their_tags() {
        let raw = RawChunk::new(*b"FUTR", vec![1, 2, 3]);
        assert_eq!(raw.tag(), *b"FUTR");

        let known = VersionChunk::new(800);
        assert_eq!(known.tag(), *b"VERS");
        assert_eq!(
            VersionChunk::decode(&known.encode().unwrap(), 800).unwrap(),
            known
        );
    }
}
