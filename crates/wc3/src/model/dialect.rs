//! Application dialects layered on standard Warcraft record layouts.
use crate::model::{mdx, Cursor, Encoder, ModelVersion, Tag, Version};
use std::{fmt::Debug, marker::PhantomData};

/// Application-defined chunks. Return `None` only for tags this codec does not own.
/// Recognized malformed payloads must return an error. The container checks that
/// successful decoding consumes the complete bounded payload.
pub trait ModelExtension: Clone + Debug {
    /// Four-byte tag identifying this extension value.
    fn tag(&self) -> Tag;
    /// Write a payload for the enclosing model's standard format version.
    fn encode_payload(
        &self,
        version: Version,
        output: &mut Encoder<'_>,
    ) -> Result<(), mdx::WriteError>;
    /// Decode an owned tag, or return `None` without consuming input.
    fn decode_payload(
        version: Version,
        tag: Tag,
        input: &mut Cursor<'_>,
    ) -> Result<Option<Self>, mdx::ReadError>;
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
impl ModelExtension for NoExtensions {
    fn tag(&self) -> Tag {
        match *self {}
    }
    fn encode_payload(&self, _: Version, _: &mut Encoder<'_>) -> Result<(), mdx::WriteError> {
        match *self {}
    }
    fn decode_payload(
        _: Version,
        _: Tag,
        _: &mut Cursor<'_>,
    ) -> Result<Option<Self>, mdx::ReadError> {
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
