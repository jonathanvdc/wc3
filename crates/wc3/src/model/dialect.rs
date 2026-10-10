//! Application dialects layered on standard Warcraft record layouts.
use crate::model::{mdl, mdx, Chunk, Cursor, Encoder, IoError, ModelVersion, Tag};
use std::{fmt::Debug, io::Write as IoWrite, marker::PhantomData};

/// A model dialect selects standard record layouts and application chunk values.
pub trait ModelDialect: Clone + Debug {
    /// Standard Warcraft record layout and numeric format version.
    type Version: ModelVersion;
    /// Application-defined chunk values.
    type Extension: Clone + Debug;
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
impl mdx::Extension for NoExtensions {
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
pub struct Extended<V: ModelVersion, E: Clone + Debug>(PhantomData<(V, E)>);
impl<V: ModelVersion, E: Clone + Debug> ModelDialect for Extended<V, E> {
    type Version = V;
    type Extension = E;
}

impl mdl::Extension for NoExtensions {
    fn read_extension(_: &str, _: &mut mdl::Parser<'_>) -> Result<Option<Self>, mdl::ReadError> {
        Ok(None)
    }
}
impl mdl::Write for NoExtensions {
    fn write_mdl<W: IoWrite>(
        &self,
        _: &mut mdl::Writer<W>,
    ) -> Result<(), IoError<mdl::WriteError>> {
        match *self {}
    }
}
