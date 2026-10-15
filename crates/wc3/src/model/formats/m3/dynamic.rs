//! Envelope detection and owned standard I/O adapters.
use super::{Document, Envelope, Md33, Md34, ReadError, WriteError};
use crate::model::common::IoError;
use std::io::{Read as IoRead, Write as IoWrite};

/// A native model whose reference envelope is detected from its signature.
#[derive(Clone, Debug)]
pub enum DynamicDocument<'data> {
    /// Eight-byte section-reference envelope.
    Md33(Document<'data, Md33>),
    /// Twelve-byte section-reference envelope.
    Md34(Document<'data, Md34>),
}
impl<'data> DynamicDocument<'data> {
    /// Detects the reversed native FourCC and validates the selected model graph.
    pub fn read(bytes: &'data [u8]) -> Result<Self, ReadError> {
        match bytes.get(..4) {
            Some(b"33DM") => Ok(Self::Md33(Document::read(bytes)?)),
            Some(b"43DM") => Ok(Self::Md34(Document::read(bytes)?)),
            _ => Err(ReadError::invalid(0, "envelope magic")),
        }
    }
    /// Encodes the selected envelope without converting any record revisions.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        match self {
            Self::Md33(model) => model.write(),
            Self::Md34(model) => model.write(),
        }
    }
    /// Detaches all source buffers.
    pub fn into_owned(self) -> DynamicDocument<'static> {
        match self {
            Self::Md33(model) => DynamicDocument::Md33(model.into_owned()),
            Self::Md34(model) => DynamicDocument::Md34(model.into_owned()),
        }
    }
}
/// Buffers and validates a native model under an explicitly selected envelope.
pub fn from_reader<E: Envelope>(
    mut reader: impl IoRead,
) -> Result<Document<'static, E>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(Document::read(&bytes).map_err(IoError::Codec)?.into_owned())
}
/// Buffers a model source and detects its reference envelope.
pub fn from_reader_dynamic(
    mut reader: impl IoRead,
) -> Result<DynamicDocument<'static>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(DynamicDocument::read(&bytes)
        .map_err(IoError::Codec)?
        .into_owned())
}
/// Encodes a model before writing it to a sink; does not flush the sink.
pub fn to_writer<E: Envelope>(
    mut writer: impl IoWrite,
    model: &Document<'_, E>,
) -> Result<(), IoError<WriteError>> {
    writer
        .write_all(&model.write().map_err(IoError::Codec)?)
        .map_err(IoError::Io)
}
