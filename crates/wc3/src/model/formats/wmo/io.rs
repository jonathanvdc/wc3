//! Composable codecs for fixed WMO records and standard I/O sinks.
use super::codec::{Decoder, Encoder};
use super::{Dialect, DynamicFile, Group, Profile, ReadError, Root, WriteError};
use crate::model::common::IoError;
use std::io::{Read as IoRead, Write as IoWrite};
/// A value decoded from a bounded WMO payload.
pub trait Read<'data>: Sized {
    /// Reads fields in native wire order.
    fn read(input: &mut Decoder<'data>) -> Result<Self, ReadError>;
}
/// A value encoded in native WMO wire order.
pub trait Write {
    /// Appends the encoded fields.
    fn write(&self, output: &mut Encoder) -> Result<(), WriteError>;
}
/// Fixed inline byte width of a native WMO record.
pub trait RecordLayout {
    /// Returns the encoded byte width.
    fn size() -> usize;
}
/// Optional semantic validation, separate from bounded decoding.
pub trait Validate {
    /// Checks locally available field relationships.
    fn validate(&self) -> Result<(), ReadError>;
}
/// Encodes a value before writing it; the sink is not flushed.
pub fn to_writer(mut writer: impl IoWrite, value: &impl Write) -> Result<(), IoError<WriteError>> {
    let mut output = Encoder::new();
    value.write(&mut output).map_err(IoError::Codec)?;
    writer.write_all(&output.finish()).map_err(IoError::Io)
}

/// Buffers a WMO root source and detaches all borrowed payload storage.
pub fn from_reader<D: Dialect>(
    mut reader: impl IoRead,
) -> Result<Root<'static, D>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(Root::<D>::decode(&bytes)
        .map_err(IoError::Codec)?
        .into_owned())
}
/// Buffers a WMO group source and detaches all borrowed payload storage.
pub fn from_reader_group<D: Dialect>(
    mut reader: impl IoRead,
) -> Result<Group<'static, D>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(Group::<D>::decode(&bytes)
        .map_err(IoError::Codec)?
        .into_owned())
}

/// Buffers a root or group file using an explicit client-layout profile.
///
/// MVER17 does not identify the profile, so it cannot be detected from the
/// numeric version alone.
pub fn from_reader_dynamic(
    mut reader: impl IoRead,
    profile: Profile,
) -> Result<DynamicFile<'static>, IoError<ReadError>> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(IoError::Io)?;
    Ok(DynamicFile::decode(&bytes, profile)
        .map_err(IoError::Codec)?
        .into_owned())
}
