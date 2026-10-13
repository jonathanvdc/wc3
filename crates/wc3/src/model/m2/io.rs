//! Composable record codec traits and standard I/O adapters.
use super::records::ModelFlags;
use super::{
    codec::{Decoder, Encoder},
    Dialect, DynamicModel, Model, ReadError, WriteError,
};
use crate::model::common::IoError;
use std::io::{Read as IoRead, Write as IoWrite};

/// A value decoded within an M2 offset address space.
pub trait Read<'data>: Sized {
    /// Reads inline metadata and resolves locally stored payloads.
    fn read(decoder: &mut Decoder<'data>) -> Result<Self, ReadError>;
}
/// A value encoded within an M2 offset address space.
pub trait Write {
    /// Writes inline metadata and allocates locally stored payloads.
    fn write(&self, encoder: &mut Encoder) -> Result<(), WriteError>;
}
/// The inline metadata width of a record, excluding offset-addressed payloads.
pub trait RecordLayout {
    /// Returns the metadata width for the selected context.
    fn size(version: u32, flags: u32) -> usize;
}

/// Buffers a standard I/O source and returns an independent typed model.
pub fn from_reader<D: Dialect>(
    mut reader: impl IoRead,
) -> Result<Model<'static, D>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    let model = Model::<D>::decode_m2(&bytes).map_err(IoError::Codec)?;
    Ok(model.into_owned())
}
/// Buffers and dynamically detects an independent model from standard I/O.
pub fn from_reader_dynamic(
    mut reader: impl IoRead,
) -> Result<DynamicModel<'static>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(DynamicModel::decode_m2(&bytes)
        .map_err(IoError::Codec)?
        .into_owned())
}
/// Encodes a self-contained value before writing; the sink is not flushed.
///
/// Models select their own dialect and flags. Context-dependent record values
/// should instead use an explicitly configured [`Encoder`].
pub fn to_writer(mut writer: impl IoWrite, value: &impl Write) -> Result<(), IoError<WriteError>> {
    let mut encoder = Encoder::new(0, ModelFlags::default());
    value.write(&mut encoder).map_err(IoError::Codec)?;
    let bytes = encoder.finish();
    writer.write_all(&bytes).map_err(IoError::Io)
}
