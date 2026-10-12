//! Envelope detection and owned standard I/O adapters.
use super::{Envelope, Md33, Md34, Model, ReadError, WriteError};
use crate::model::common::IoError;
use std::io::{Read as IoRead, Write as IoWrite};

/// A native model whose reference envelope is detected from its signature.
#[derive(Clone, Debug)]
pub enum DynamicModel<'data> {
    /// Eight-byte section-reference envelope.
    Md33(Model<'data, Md33>),
    /// Twelve-byte section-reference envelope.
    Md34(Model<'data, Md34>),
}
impl<'data> DynamicModel<'data> {
    /// Detects the reversed native FourCC and validates the selected model graph.
    pub fn read(bytes: &'data [u8]) -> Result<Self, ReadError> {
        match bytes.get(..4) {
            Some(b"33DM") => Ok(Self::Md33(Model::read(bytes)?)),
            Some(b"43DM") => Ok(Self::Md34(Model::read(bytes)?)),
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
    pub fn into_owned(self) -> DynamicModel<'static> {
        match self {
            Self::Md33(model) => DynamicModel::Md33(model.into_owned()),
            Self::Md34(model) => DynamicModel::Md34(model.into_owned()),
        }
    }
}
/// Buffers and validates a native model under an explicitly selected envelope.
pub fn from_reader<E: Envelope>(
    mut reader: impl IoRead,
) -> Result<Model<'static, E>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(Model::read(&bytes).map_err(IoError::Codec)?.into_owned())
}
/// Buffers a model source and detects its reference envelope.
pub fn from_reader_dynamic(
    mut reader: impl IoRead,
) -> Result<DynamicModel<'static>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(DynamicModel::read(&bytes)
        .map_err(IoError::Codec)?
        .into_owned())
}
/// Encodes a model before writing it to a sink; does not flush the sink.
pub fn to_writer<E: Envelope>(
    mut writer: impl IoWrite,
    model: &Model<'_, E>,
) -> Result<(), IoError<WriteError>> {
    writer
        .write_all(&model.write().map_err(IoError::Codec)?)
        .map_err(IoError::Io)
}
