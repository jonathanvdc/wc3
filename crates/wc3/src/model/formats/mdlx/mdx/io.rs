//! Adapters for standard I/O sources and sinks.
use super::{ReadOwned, Write};
use crate::model::formats::mdlx::mdx;
use crate::model::formats::mdlx::IoError;
use crate::model::formats::mdlx::{DynamicDocument, Version};
use std::fmt::Debug;
use std::io::{Read as IoRead, Write as IoWrite};

/// Buffers the source through EOF, then decodes exactly one value.
/// For concatenated records, use `Cursor::read` instead.
pub fn from_reader<T: ReadOwned>(mut reader: impl IoRead) -> Result<T, IoError<mdx::ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    T::decode_owned(&bytes).map_err(IoError::Codec)
}

/// Buffers a complete model and selects its version, using the fallback when
/// the file has no version chunk. `E` selects the application extension codec.
pub fn from_reader_with_version<E: Clone + Debug + 'static + for<'input> mdx::Extension<'input>>(
    mut reader: impl IoRead,
    default_version: Version,
) -> Result<DynamicDocument<'static, E>, IoError<mdx::ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    DynamicDocument::<E>::decode_mdx(&bytes, default_version)
        .map(|model| model.into_owned())
        .map_err(IoError::Codec)
}

/// Encodes into a temporary buffer before writing all bytes to the sink.
/// Encoding errors leave the sink untouched; I/O errors may leave partial
/// output. Does not flush the sink. No seeking is required.
pub fn to_writer<T: Write + ?Sized>(
    mut writer: impl IoWrite,
    value: &T,
) -> Result<(), IoError<mdx::WriteError>> {
    let bytes = value.encode_mdx().map_err(IoError::Codec)?;
    writer.write_all(&bytes).map_err(IoError::Io)
}
