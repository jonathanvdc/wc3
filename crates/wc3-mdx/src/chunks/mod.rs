//! Raw and typed top-level MDX chunks.
use crate::EncodeError;
use crate::{Cursor, Encodable, Encoder, Readable, Tag};

use crate::DecodeError;

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
    fn decode_payload(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError>;
}

impl<T: KnownChunk> Readable for T {
    fn read_from(cursor: &mut Cursor<'_>) -> Result<Self, DecodeError> {
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
        let decoded = T::decode_payload(&mut payload)?;
        payload.finish()?;
        *cursor = next;
        Ok(decoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Encodable, Readable};

    #[test]
    fn chunks_encode_complete_headers_without_nesting() {
        let known = VersionChunk::<crate::V800>::new();
        let expected = [
            b"VERS".as_slice(),
            &4u32.to_le_bytes(),
            &800u32.to_le_bytes(),
        ]
        .concat();
        assert_eq!(known.encode().unwrap(), expected);
        assert_eq!(ModelChunk::from(known.clone()).encode().unwrap(), expected);
        assert_eq!(
            VersionChunk::<crate::V800>::decode(&expected).unwrap(),
            known
        );

        let raw = RawChunk::new(*b"FUTR", vec![1, 2, 3]);
        let raw_bytes = [b"FUTR".as_slice(), &3u32.to_le_bytes(), &[1, 2, 3]].concat();
        assert_eq!(raw.encode().unwrap(), raw_bytes);
        assert_eq!(
            ModelChunk::Unknown(UnknownChunk::<crate::V800>::new(raw).unwrap())
                .encode()
                .unwrap(),
            raw_bytes
        );
    }

    #[test]
    fn known_chunk_decoder_checks_header_and_boundaries() {
        let bytes = VersionChunk::<crate::V800>::new().encode().unwrap();
        let mut wrong = bytes.clone();
        wrong[..4].copy_from_slice(b"MODL");
        assert_eq!(
            VersionChunk::<crate::V800>::decode(&wrong),
            Err(DecodeError::UnexpectedChunkTag {
                expected: *b"VERS",
                actual: *b"MODL",
            })
        );
        let mut short = bytes.clone();
        short[4..8].copy_from_slice(&5u32.to_le_bytes());
        assert!(matches!(
            VersionChunk::<crate::V800>::decode(&short),
            Err(DecodeError::TruncatedChunk { .. })
        ));
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(matches!(
            VersionChunk::<crate::V800>::decode(&trailing),
            Err(DecodeError::TrailingRecordBytes { .. })
        ));
        let joined = [bytes.clone(), bytes].concat();
        let mut cursor = Cursor::new(&joined);
        cursor.read::<VersionChunk<crate::V800>>().unwrap();
        cursor.read::<VersionChunk<crate::V800>>().unwrap();
        cursor.finish().unwrap();
    }

    #[test]
    fn raw_and_known_chunks_expose_their_tags() {
        let raw = RawChunk::new(*b"FUTR", vec![1, 2, 3]);
        assert_eq!(raw.tag(), *b"FUTR");

        let known = VersionChunk::<crate::V800>::new();
        assert_eq!(known.tag(), *b"VERS");
        assert_eq!(
            VersionChunk::<crate::V800>::decode(&known.encode().unwrap()).unwrap(),
            known
        );
    }
}
