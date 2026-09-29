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

use super::{Dialect, Writer};

/// Buffers UTF-8 text through EOF, then parses exactly one value.
/// Invalid UTF-8 is an I/O `InvalidData` error. For source-based diagnostics,
/// retain the text yourself and use `Read::decode_mdl` instead.
pub fn from_reader<T: Read>(mut reader: impl IoRead) -> Result<T, FromReaderError> {
    let mut source = String::new();
    reader
        .read_to_string(&mut source)
        .map_err(FromReaderError::Io)?;
    T::decode_mdl(&source).map_err(FromReaderError::Decode)
}

/// Streams one value using the default Warcraft III dialect and checks block
/// balance. Errors may leave partial output. Does not flush the sink.
pub fn to_writer<T: Write + ?Sized>(writer: impl IoWrite, value: &T) -> Result<(), WriteError> {
    to_writer_with_dialect(writer, value, Dialect::Warcraft3)
}

/// Streams one value using the selected dialect and checks block balance.
/// Errors may leave partial output. Does not flush the sink.
pub fn to_writer_with_dialect<T: Write + ?Sized>(
    writer: impl IoWrite,
    value: &T,
    dialect: Dialect,
) -> Result<(), WriteError> {
    let mut writer = Writer::with_dialect(writer, dialect);
    writer.write(value)?;
    writer.finish()?;
    Ok(())
}
