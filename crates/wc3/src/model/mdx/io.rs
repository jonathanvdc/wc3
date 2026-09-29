//! Adapters for standard I/O sources and sinks.
use super::{Read, ReadError, Write, WriteError};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read as IoRead, Write as IoWrite};

/// A source I/O failure or a malformed complete input.
#[derive(Debug)]
pub enum FromReaderError {
    Io(io::Error),
    Decode(ReadError),
}

impl Display for FromReaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => Display::fmt(error, f),
            Self::Decode(error) => Display::fmt(error, f),
        }
    }
}

impl Error for FromReaderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Decode(error) => Some(error),
        }
    }
}

use crate::model::{DynamicModel, Version};

/// Buffers the source through EOF, then decodes exactly one value.
/// For concatenated records, use `Cursor::read` instead.
pub fn from_reader<T: Read>(mut reader: impl IoRead) -> Result<T, FromReaderError> {
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(FromReaderError::Io)?;
    T::decode_mdx(&bytes).map_err(FromReaderError::Decode)
}

/// Buffers a complete model and selects its version, using the fallback when
/// the file has no version chunk.
pub fn from_reader_with_version(
    mut reader: impl IoRead,
    default_version: Version,
) -> Result<DynamicModel, FromReaderError> {
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .map_err(FromReaderError::Io)?;
    DynamicModel::decode_mdx(&bytes, default_version).map_err(FromReaderError::Decode)
}

/// A value encoding failure or a sink I/O failure.
#[derive(Debug)]
pub enum ToWriterError {
    Encode(WriteError),
    Io(io::Error),
}

impl Display for ToWriterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(error) => Display::fmt(error, f),
            Self::Io(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ToWriterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Encode(error) => Some(error),
            Self::Io(error) => Some(error),
        }
    }
}

/// Encodes into a temporary buffer before writing all bytes to the sink.
/// Encoding errors leave the sink untouched; I/O errors may leave partial
/// output. Does not flush the sink. No seeking is required.
pub fn to_writer<T: Write + ?Sized>(
    mut writer: impl IoWrite,
    value: &T,
) -> Result<(), ToWriterError> {
    let bytes = value.encode_mdx().map_err(ToWriterError::Encode)?;
    writer.write_all(&bytes).map_err(ToWriterError::Io)
}
