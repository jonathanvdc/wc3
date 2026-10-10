//! Application dialects layered on standard Warcraft record layouts.
use crate::model::{mdx, Chunk, Cursor, Encoder, KnownChunk, ModelVersion, Tag};
use std::{fmt::Debug, marker::PhantomData};

/// Dispatches application chunk tags to their payload codecs.
///
/// The container supplies a bounded payload cursor and checks that decoded values
/// consume the complete payload and retain its tag. Fixed-tag types implementing
/// `KnownChunk`, `Clone`, and `Debug` receive an automatic implementation.
pub trait ModelExtension: Chunk + Clone + Debug {
    /// Decodes the payload for a recognized tag.
    ///
    /// Returns `Some` for a decoded value, `None` without consuming input for an
    /// unrecognized tag, and an error for a malformed recognized payload.
    fn read_extension(tag: Tag, input: &mut Cursor<'_>) -> Result<Option<Self>, mdx::ReadError>;
}

/// Fixed-tag chunks automatically participate in extension dispatch.
impl<T: KnownChunk + Clone + Debug> ModelExtension for T {
    fn read_extension(tag: Tag, input: &mut Cursor<'_>) -> Result<Option<Self>, mdx::ReadError> {
        if tag == T::TAG {
            T::decode_payload(input).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// An MDX dialect selects standard record layouts and application chunks.
pub trait ModelDialect: Clone + Debug {
    /// Standard Warcraft record layout and numeric format version.
    type Version: ModelVersion;
    /// Application-defined chunk values and their binary codec.
    type Extension: ModelExtension;
}

/// Standard Warcraft dialects do not interpret application chunks.
#[derive(Clone, Debug)]
pub enum NoExtensions {}
impl Chunk for NoExtensions {
    fn tag(&self) -> Tag {
        match *self {}
    }
    fn encode_payload_to(&self, _: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        match *self {}
    }
}
impl ModelExtension for NoExtensions {
    fn read_extension(_: Tag, _: &mut Cursor<'_>) -> Result<Option<Self>, mdx::ReadError> {
        Ok(None)
    }
}
impl<V: ModelVersion> ModelDialect for V {
    type Version = V;
    type Extension = NoExtensions;
}

/// A standard format version extended with an application's chunk enum.
#[derive(Clone, Debug)]
pub struct Extended<V: ModelVersion, E: ModelExtension>(PhantomData<(V, E)>);
impl<V: ModelVersion, E: ModelExtension> ModelDialect for Extended<V, E> {
    type Version = V;
    type Extension = E;
}
