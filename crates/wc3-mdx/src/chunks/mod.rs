//! Raw and typed top-level MDX chunks.
use crate::EncodeError;
use crate::{Cursor, Decodable, Encodable, Encoder, Tag, Version};

use crate::DecodeError;

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
pub trait Chunk {
    /// Returns this value's chunk tag.
    fn tag(&self) -> Tag;

    /// Writes only the contents of the chunk, without its tag or size.
    fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError>;
}

impl<T: Chunk + ?Sized> Encodable for T {
    fn encode_to(&self, output: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let tag = self.tag();
        output.write_bytes(&tag);
        let marker = output.begin_sized();
        self.encode_payload_to(output)?;
        output.finish_payload(marker, tag)
    }
}

/// A complete chunk whose tag is fixed by its type.
pub trait KnownChunk: Chunk + Sized {
    /// The four-byte chunk tag for this type.
    const TAG: Tag;

    /// Decodes a bounded chunk payload, without the tag or size.
    fn decode_payload(cursor: &mut Cursor<'_>, version: Version) -> Result<Self, DecodeError>;
}

impl<T: KnownChunk> Decodable for T {
    fn decode_one(cursor: &mut Cursor<'_>, version: Version) -> Result<Self, DecodeError> {
        let mut next = *cursor;
        let offset = next.absolute_position();
        if next.remaining().len() < 8 {
            return Err(DecodeError::TruncatedHeader { offset });
        }
        let tag: Tag = next.read_exact(4)?.try_into().expect("four-byte tag");
        if tag != T::TAG {
            return Err(DecodeError::UnexpectedChunkTag {
                expected: T::TAG,
                actual: tag,
            });
        }
        let size = next.read()?;
        let mut payload = next
            .slice(size as usize)
            .map_err(|_| DecodeError::TruncatedChunk { tag, offset, size })?;
        let decoded = T::decode_payload(&mut payload, version)?;
        payload.finish()?;
        *cursor = next;
        Ok(decoded)
    }
}

fn checked_chunk_size(count: usize, width: usize, tag: Tag) -> Result<usize, EncodeError> {
    let size = count.checked_mul(width).ok_or(EncodeError::ChunkTooLarge {
        tag,
        size: usize::MAX,
    })?;
    if size > u32::MAX as usize {
        return Err(EncodeError::ChunkTooLarge { tag, size });
    }
    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Decodable, Encodable};

    #[test]
    fn chunks_encode_complete_headers_without_nesting() {
        let known = VersionChunk::new(800);
        let expected = [
            b"VERS".as_slice(),
            &4u32.to_le_bytes(),
            &800u32.to_le_bytes(),
        ]
        .concat();
        assert_eq!(known.encode().unwrap(), expected);
        assert_eq!(ModelChunk::from(known.clone()).encode().unwrap(), expected);
        assert_eq!(VersionChunk::decode(&expected, 800).unwrap(), known);

        let raw = RawChunk::new(*b"FUTR", vec![1, 2, 3]);
        let raw_bytes = [b"FUTR".as_slice(), &3u32.to_le_bytes(), &[1, 2, 3]].concat();
        assert_eq!(raw.encode().unwrap(), raw_bytes);
        assert_eq!(ModelChunk::Unknown(raw).encode().unwrap(), raw_bytes);
    }

    #[test]
    fn known_chunk_decoder_checks_header_and_boundaries() {
        let bytes = VersionChunk::new(800).encode().unwrap();
        let mut wrong = bytes.clone();
        wrong[..4].copy_from_slice(b"MODL");
        assert_eq!(
            VersionChunk::decode(&wrong, 800),
            Err(DecodeError::UnexpectedChunkTag {
                expected: *b"VERS",
                actual: *b"MODL",
            })
        );
        let mut short = bytes.clone();
        short[4..8].copy_from_slice(&5u32.to_le_bytes());
        assert!(matches!(
            VersionChunk::decode(&short, 800),
            Err(DecodeError::TruncatedChunk { .. })
        ));
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(matches!(
            VersionChunk::decode(&trailing, 800),
            Err(DecodeError::TrailingRecordBytes { .. })
        ));
        let joined = [bytes.clone(), bytes].concat();
        let mut cursor = Cursor::new(&joined);
        VersionChunk::decode_one(&mut cursor, 800).unwrap();
        VersionChunk::decode_one(&mut cursor, 800).unwrap();
        cursor.finish().unwrap();
    }

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
