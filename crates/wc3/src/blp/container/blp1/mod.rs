//! BLP1 container types.
use crate::blp::{Palette, MIPMAP_SLOTS};

mod codec;
mod ownership;
use std::borrow::Cow;

/// BLP1 fields shared by both content encodings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Blp1Header {
    /// Width of mipmap level zero, in pixels.
    pub width: u32,
    /// Height of mipmap level zero, in pixels.
    pub height: u32,
    /// Alpha depth recorded in the container header, in bits per pixel.
    pub alpha_bits: u32,
    /// Encoding-specific extra header word, preserved during round trips.
    pub extra: u32,
    /// Raw mipmap presence flag from the BLP1 header.
    pub has_mipmaps: u32,
}

/// BLP1 content borrowing its original bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blp1Content<'a> {
    /// JPEG payloads with a header prefix shared by all mipmaps.
    Jpeg {
        /// JPEG prefix prepended to each mipmap payload before decoding.
        shared_header: Cow<'a, [u8]>,
    },
    /// Palette indices with a separate packed alpha plane.
    Indexed {
        /// The 256-entry palette, stored as four BGRA bytes per entry.
        palette: Palette<'a>,
    },
}

/// Borrowed BLP1 container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp1<'a> {
    /// Dimensions and encoding-independent header fields.
    pub header: Blp1Header,
    /// Borrowed encoding metadata shared by all mipmaps.
    pub content: Blp1Content<'a>,
    /// Encoded mipmap payloads, with full-resolution level zero first; `None` denotes an absent level.
    pub mipmaps: [Option<Cow<'a, [u8]>>; MIPMAP_SLOTS],
}
