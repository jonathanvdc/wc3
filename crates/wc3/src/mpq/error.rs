use std::error::Error as StdError;
use std::fmt;
use std::io;

/// An archive format, lookup, codec, or I/O failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The underlying archive source or destination failed an I/O operation.
    Io(io::Error),
    /// The archive layout is invalid; the contained string identifies the failed check.
    InvalidArchive(&'static str),
    /// An unsupported on-disk header version was encountered.
    UnsupportedVersion(u16),
    /// The entry uses unsupported or inconsistent raw file flags.
    UnsupportedFlags(u32),
    /// The entry uses an unsupported compression mask.
    UnsupportedCompression(u8),
    /// The operation requires the Cargo feature named by the contained string.
    FeatureDisabled(&'static str),
    /// A resource limit was exceeded; the contained string identifies the limit.
    LimitExceeded(&'static str),
    /// No entry matches the requested name, locale, platform, or block index.
    FileNotFound,
    /// The filename is empty or contains NUL, CR, or LF.
    InvalidName,
    /// The filename hashes and locale/platform conflict with an existing entry.
    DuplicateFile,
    /// The bytes supplied to an entry writer differ from its declared decoded size.
    SizeMismatch,
    /// A prior failure or unfinished entry prevents archive completion.
    WriterFailed,
    /// A sector checksum differs from its stored value; the number is the zero-based sector index.
    ChecksumMismatch(u32),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => e.fmt(f),
            Self::InvalidArchive(reason) => write!(f, "invalid MPQ archive: {reason}"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported MPQ header version {v}"),
            Self::UnsupportedFlags(v) => write!(f, "unsupported MPQ file flags {v:#010x}"),
            Self::UnsupportedCompression(v) => write!(f, "unsupported MPQ compression {v:#04x}"),
            Self::FeatureDisabled(v) => write!(f, "MPQ operation requires feature {v}"),
            Self::LimitExceeded(v) => write!(f, "MPQ limit exceeded: {v}"),
            Self::FileNotFound => f.write_str("MPQ file not found"),
            Self::InvalidName => {
                f.write_str("MPQ names must be nonempty and contain no NUL, CR, or LF")
            }
            Self::DuplicateFile => f.write_str("duplicate MPQ filename hashes and locale"),
            Self::SizeMismatch => f.write_str("MPQ entry length differs from its declared size"),
            Self::WriterFailed => f.write_str("MPQ writer has an unfinished or failed entry"),
            Self::ChecksumMismatch(sector) => write!(f, "MPQ sector {sector} checksum mismatch"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        // Recover errors carried through the standard streaming traits so the
        // convenience archive methods retain their structured MPQ failures.
        if value.get_ref().is_some_and(|inner| inner.is::<Self>()) {
            *value.into_inner().unwrap().downcast::<Self>().unwrap()
        } else {
            Self::Io(value)
        }
    }
}

impl From<Error> for io::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Io(e) => e,
            Error::FileNotFound => Self::new(io::ErrorKind::NotFound, value),
            _ => Self::new(io::ErrorKind::InvalidData, value),
        }
    }
}
