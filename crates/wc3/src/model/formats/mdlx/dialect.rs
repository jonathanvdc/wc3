//! Application dialects layered on standard Warcraft record layouts.
use crate::model::formats::mdlx::{mdl, mdx, Chunk, Cursor, Encoder, FormatVersion, IoError, Tag};
use std::{fmt::Debug, io::Write as IoWrite, marker::PhantomData};

/// A model dialect selects standard record layouts and application chunk values.
pub trait Dialect: Clone + Debug {
    /// Standard Warcraft record layout and numeric format version.
    type Version: FormatVersion;
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
impl<'data> mdx::Extension<'data> for NoExtensions {
    fn read_extension(_: Tag, _: &mut Cursor<'data>) -> Result<Option<Self>, mdx::ReadError> {
        Ok(None)
    }
}
impl<V: FormatVersion> Dialect for V {
    type Version = V;
    type Extension = NoExtensions;
}

/// A standard format version extended with an application's chunk enum.
#[derive(Clone, Debug)]
pub struct Extended<V: FormatVersion, E: Clone + Debug>(PhantomData<(V, E)>);
impl<V: FormatVersion, E: Clone + Debug> Dialect for Extended<V, E> {
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
