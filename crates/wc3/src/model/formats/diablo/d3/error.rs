use std::{error::Error, fmt};
/// Failure decoding a supported native Diablo III document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadError {
    /// The SNO header signature differs from `DEADBEEF`.
    Magic,
    /// The explicitly selected family does not support this revision.
    Revision {
        /// Required header revision.
        expected: u32,
        /// Found header revision.
        actual: u32,
    },
    /// A referenced byte range exceeds the payload.
    Range {
        /// Payload-relative start.
        offset: usize,
        /// Required number of bytes.
        size: usize,
    },
    /// A descriptor is negative or has a partial record.
    Descriptor {
        /// Descriptor's payload-relative location.
        offset: usize,
    },
    /// A fixed-width value is malformed.
    Value {
        /// Payload-relative location.
        offset: usize,
    },
    /// A stored count contradicts a resolved array.
    Count {
        /// Payload-relative location.
        offset: usize,
        /// Resolved array length.
        expected: usize,
        /// Stored count.
        actual: i32,
    },
    /// A packed vertex representation has not been established for this revision.
    VertexFormat(u32),
}
impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Diablo III decode: {self:?}")
    }
}
impl Error for ReadError {}
/// Failure rebuilding a native Diablo III document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WriteError {
    /// The authored vertex flag selects an unsupported packed layout.
    VertexFormat(u32),
    /// A payload or descriptor cannot fit the native signed 32-bit width.
    Size,
    /// Fixed storage was not reserved at this offset.
    Range {
        /// Payload-relative location.
        offset: usize,
    },
}
impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Diablo III encode: {self:?}")
    }
}
impl Error for WriteError {}
