//! Access model records while preserving their MDX chunk organization.
//!
//! Match [`ModelChunk`] variants in a model's `chunks` vector to edit records
//! in place. This retains duplicate chunks and their order, unlike collection
//! setters that replace a complete collection. [`UnknownChunk`] keeps payloads
//! whose tags are not recognized; [`RawChunk`] is available for direct byte access.
//!
//! The chunk traits distinguish a complete chunk from its payload. Use model
//! codecs for whole files; `encode_payload_to()` writes neither the chunk tag
//! nor its size header.
use crate::model::mdx;
use crate::model::WriteError;
use crate::model::{Cursor, Encoder, Tag};

use crate::model::ReadError;

mod raw;
pub use raw::RawChunk;
mod bind_pose;
pub use bind_pose::BindPoseChunk;
mod collections;
pub use collections::*;
mod model_info;
pub use model_info::ModelInfoChunk;
mod model_chunk;
pub use model_chunk::{ModelChunk, UnknownChunk};
mod version;
pub use version::VersionChunk;

/// A value that represents a complete top-level chunk.
pub trait Chunk {
    /// Returns this value's chunk tag.
    fn tag(&self) -> Tag;

    /// Writes only the contents of the chunk, without its tag or size.
    fn encode_payload_to(&self, output: &mut Encoder<'_>) -> Result<(), WriteError>;
}

impl<T: Chunk + ?Sized> mdx::Write for T {
    fn write_mdx(&self, output: &mut Encoder<'_>) -> Result<(), WriteError> {
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
    fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, ReadError>;
}

impl<T: KnownChunk> mdx::Read for T {
    fn read_mdx(cursor: &mut Cursor<'_>) -> Result<Self, ReadError> {
        let mut next = *cursor;
        let offset = next.absolute_position();
        if next.remaining().len() < 8 {
            return Err(ReadError::TruncatedHeader { offset });
        }
        let tag: Tag = next.read_exact(4)?.try_into().expect("four-byte tag");
        if tag != T::TAG {
            return Err(ReadError::UnexpectedChunkTag {
                expected: T::TAG,
                actual: tag,
            });
        }
        let size = next.read()?;
        let mut payload = next
            .slice(size as usize)
            .map_err(|_| ReadError::TruncatedChunk { tag, offset, size })?;
        let decoded = T::decode_payload(&mut payload)?;
        payload.finish()?;
        *cursor = next;
        Ok(decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{mdx::Read, mdx::Write};

    #[test]
    fn chunks_encode_complete_headers_without_nesting() {
        let known = VersionChunk::<crate::model::V800>::new();
        let expected = [
            b"VERS".as_slice(),
            &4u32.to_le_bytes(),
            &800u32.to_le_bytes(),
        ]
        .concat();
        assert_eq!(known.encode_mdx().unwrap(), expected);
        assert_eq!(
            ModelChunk::from(known.clone()).encode_mdx().unwrap(),
            expected
        );
        assert_eq!(
            VersionChunk::<crate::model::V800>::decode_mdx(&expected).unwrap(),
            known
        );

        let raw = RawChunk::new(*b"FUTR", vec![1, 2, 3]);
        let raw_bytes = [b"FUTR".as_slice(), &3u32.to_le_bytes(), &[1, 2, 3]].concat();
        assert_eq!(raw.encode_mdx().unwrap(), raw_bytes);
        assert_eq!(
            ModelChunk::Unknown(UnknownChunk::<crate::model::V800>::new(raw).unwrap())
                .encode_mdx()
                .unwrap(),
            raw_bytes
        );
    }

    #[test]
    fn known_chunk_decoder_checks_header_and_boundaries() {
        let bytes = VersionChunk::<crate::model::V800>::new()
            .encode_mdx()
            .unwrap();
        let mut wrong = bytes.clone();
        wrong[..4].copy_from_slice(b"MODL");
        assert_eq!(
            VersionChunk::<crate::model::V800>::decode_mdx(&wrong),
            Err(ReadError::UnexpectedChunkTag {
                expected: *b"VERS",
                actual: *b"MODL",
            })
        );
        let mut short = bytes.clone();
        short[4..8].copy_from_slice(&5u32.to_le_bytes());
        assert!(matches!(
            VersionChunk::<crate::model::V800>::decode_mdx(&short),
            Err(ReadError::TruncatedChunk { .. })
        ));
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(matches!(
            VersionChunk::<crate::model::V800>::decode_mdx(&trailing),
            Err(ReadError::TrailingRecordBytes { .. })
        ));
        let joined = [bytes.clone(), bytes].concat();
        let mut cursor = Cursor::new(&joined);
        cursor.read::<VersionChunk<crate::model::V800>>().unwrap();
        cursor.read::<VersionChunk<crate::model::V800>>().unwrap();
        cursor.finish().unwrap();
    }

    #[test]
    fn raw_and_known_chunks_expose_their_tags() {
        let raw = RawChunk::new(*b"FUTR", vec![1, 2, 3]);
        assert_eq!(raw.tag(), *b"FUTR");

        let known = VersionChunk::<crate::model::V800>::new();
        assert_eq!(known.tag(), *b"VERS");
        assert_eq!(
            VersionChunk::<crate::model::V800>::decode_mdx(&known.encode_mdx().unwrap()).unwrap(),
            known
        );
    }
}
