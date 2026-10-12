//! Owned adapters for standard byte streams.
use super::{Dialect, Document, Options, RawDocument, ReadError, WriteError};
use crate::model::common::IoError;
use std::io::{Read as IoRead, Write as IoWrite};

/// Buffers a stream and checks its selected document generation.
pub fn from_reader<D: Dialect>(
    reader: impl IoRead,
) -> Result<Document<'static, D>, IoError<ReadError>> {
    from_reader_with_options(reader, Options::default())
}
/// Buffers a stream with explicit native producer information.
pub fn from_reader_with_options<D: Dialect>(
    mut reader: impl IoRead,
    options: Options,
) -> Result<Document<'static, D>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(Document::read_with_options(&bytes, options)
        .map_err(IoError::Codec)?
        .into_owned())
}
/// Buffers an index without requiring a supported document generation.
pub fn raw_from_reader(
    mut reader: impl IoRead,
    options: Options,
) -> Result<RawDocument<'static>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(RawDocument::read_with_options(&bytes, options)
        .map_err(IoError::Codec)?
        .into_owned())
}
/// Encodes a checked document before writing it to a sink.
pub fn to_writer<D: Dialect>(
    mut writer: impl IoWrite,
    document: &Document<'_, D>,
) -> Result<(), IoError<WriteError>> {
    writer
        .write_all(&document.write().map_err(IoError::Codec)?)
        .map_err(IoError::Io)
}
