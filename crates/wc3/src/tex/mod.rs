//! Diablo texture containers, with native storage separate from pixel conversion.
//!
//! [`d3::Texture`](crate::tex::d3::Texture) reads revision 47 embedded mipmaps. [`d4::Texture`](crate::tex::d4::Texture) reads
//! hash F9CD83E7 metadata and resolves external payload ranges explicitly. Both
//! retain unknown fields and borrow encoded buffers. Writers relocate references;
//! they do not preserve arbitrary original padding or aliasing between ranges.
//!
//! Pixel conversion is enabled by `tex-decode` and `tex-encode`. Block codecs
//! reuse `squish` and `texture2ddecoder`. Unsupported pixel conversions return
//! an error while native encoded data remains editable.
//!
//! See the [texture guide](https://github.com/jonathanvdc/wc3/blob/main/crates/wc3/docs/tex.md)
//! for supported formats, external payload handling, and evidence limitations.
pub mod d3;
pub mod d4;
mod pixels;

use std::{error::Error as StdError, fmt};

/// Invalid native storage or unsupported pixel conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// A required byte range is outside the supplied input.
    Truncated,
    /// Magic does not identify a Diablo SNO file.
    InvalidMagic,
    /// The native revision does not match the selected typed layout.
    UnsupportedRevision(u32),
    /// Native data violates a structural constraint.
    Invalid(&'static str),
    /// Arithmetic or serialized offsets exceed supported limits.
    LimitExceeded,
    /// Pixel conversion is unavailable for this native format identifier.
    UnsupportedPixelFormat(u32),
    /// A requested face or mip does not exist or has no attached payload.
    MissingMip,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl StdError for Error {}

pub(crate) fn range(bytes: &[u8], offset: usize, len: usize) -> Result<&[u8], Error> {
    bytes
        .get(offset..offset.checked_add(len).ok_or(Error::LimitExceeded)?)
        .ok_or(Error::Truncated)
}
pub(crate) fn u32_at(bytes: &[u8], at: usize) -> Result<u32, Error> {
    Ok(u32::from_le_bytes(range(bytes, at, 4)?.try_into().unwrap()))
}
pub(crate) fn put_u32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn narrow(value: usize) -> Result<u32, Error> {
    value.try_into().map_err(|_| Error::LimitExceeded)
}
pub(crate) const MAGIC: u32 = 0xdeadbeef;
