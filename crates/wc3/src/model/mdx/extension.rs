//! Application chunk dispatch.
use crate::model::{
    chunks::{Chunk, KnownChunk},
    mdx, Cursor, Tag,
};

/// Dispatches application chunk tags to their payload codecs.
///
/// The container supplies a bounded payload cursor and checks that decoded values
/// consume the complete payload and retain its tag. Fixed-tag types implementing
/// `KnownChunk` receive an automatic implementation.
pub trait Extension: Chunk + Sized {
    /// Decodes the payload for a recognized tag.
    ///
    /// Returns `Some` for a decoded value, `None` without consuming input for an
    /// unrecognized tag, and an error for a malformed recognized payload.
    fn read_extension(tag: Tag, input: &mut Cursor<'_>) -> Result<Option<Self>, mdx::ReadError>;
}

/// Fixed-tag chunks automatically participate in extension dispatch.
impl<T: KnownChunk> Extension for T {
    fn read_extension(tag: Tag, input: &mut Cursor<'_>) -> Result<Option<Self>, mdx::ReadError> {
        if tag == T::TAG {
            T::decode_payload(input).map(Some)
        } else {
            Ok(None)
        }
    }
}
