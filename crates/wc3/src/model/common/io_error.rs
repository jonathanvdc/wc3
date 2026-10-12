//! Transport failures separated from format-specific codec errors.
use std::{error::Error as StdError, fmt, io};

/// A transport failure or an error produced by a codec.
#[derive(Debug)]
pub enum IoError<E> {
    /// An underlying stream operation failed.
    Io(io::Error),
    /// The format codec rejected the data.
    Codec(E),
}
impl<E> From<io::Error> for IoError<E> {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
impl<E: fmt::Display> fmt::Display for IoError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::Codec(error) => error.fmt(f),
        }
    }
}
impl<E: StdError + 'static> StdError for IoError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match self {
            Self::Io(error) => error,
            Self::Codec(error) => error,
        })
    }
}
