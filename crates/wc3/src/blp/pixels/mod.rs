//! Optional conversion of encoded BLP mipmaps to and from RGBA pixels.
#[cfg(feature = "blp-decode")]
mod decode;
#[cfg(feature = "blp-encode")]
mod encode;

#[cfg(feature = "blp-decode")]
pub use decode::DecodeError;
#[cfg(feature = "blp-encode")]
pub use encode::{EncodeError, EncodeFormat, EncodeOptions};
