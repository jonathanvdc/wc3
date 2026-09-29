//! Optional conversion of encoded BLP mipmaps to and from RGBA pixels.
#[cfg(feature = "blp-encode")]
mod assemble;
#[cfg(any(feature = "blp-decode", feature = "blp-encode"))]
mod bgra;
#[cfg(feature = "blp-decode")]
mod decode;
#[cfg(any(feature = "blp-decode", feature = "blp-encode"))]
mod dxt;
#[cfg(feature = "blp-encode")]
mod encode;
#[cfg(any(feature = "blp-decode", feature = "blp-encode"))]
mod indexed;
#[cfg(any(feature = "blp-decode", feature = "blp-encode"))]
mod jpeg;
#[cfg(feature = "blp-encode")]
mod mipmaps;

#[cfg(feature = "blp-decode")]
pub use decode::DecodeError;
#[cfg(feature = "blp-encode")]
pub use encode::{BlpVersion, EncodeError, EncodeFormat, EncodeOptions, IndexedAlpha};

#[cfg(feature = "blp-decode")]
use decode::{image, output, MAX_DECODE_BYTES};
