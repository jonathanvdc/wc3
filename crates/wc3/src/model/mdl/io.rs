//! Adapters for standard I/O sources and sinks.
use super::{Dialect, Writer};
use super::{Read, Write};
use crate::model::mdl;
use crate::model::IoError;
use std::io::{Read as IoRead, Write as IoWrite};

/// Buffers UTF-8 text through EOF, then parses exactly one value.
/// Invalid UTF-8 is an I/O `InvalidData` error. For source-based diagnostics,
/// retain the text yourself and use `Read::decode_mdl` instead.
pub fn from_reader<T: Read>(mut reader: impl IoRead) -> Result<T, IoError<mdl::ReadError>> {
    let mut source = String::new();
    reader.read_to_string(&mut source).map_err(IoError::Io)?;
    T::decode_mdl(&source).map_err(IoError::Codec)
}

/// Streams one value using the default Warcraft III dialect and checks block
/// balance. Errors may leave partial output. Does not flush the sink.
pub fn to_writer<T: Write + ?Sized>(
    writer: impl IoWrite,
    value: &T,
) -> Result<(), IoError<mdl::WriteError>> {
    to_writer_with_dialect(writer, value, Dialect::Warcraft3)
}

/// Streams one value using the selected dialect and checks block balance.
/// Errors may leave partial output. Does not flush the sink.
pub fn to_writer_with_dialect<T: Write + ?Sized>(
    writer: impl IoWrite,
    value: &T,
    dialect: Dialect,
) -> Result<(), IoError<mdl::WriteError>> {
    let mut writer = Writer::with_dialect(writer, dialect);
    writer.write(value)?;
    writer.finish()?;
    Ok(())
}
