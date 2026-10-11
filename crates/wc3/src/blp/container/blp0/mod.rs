//! BLP0 container types.
use crate::blp::{MIPMAP_SLOTS, PALETTE_BYTES};

mod codec;
mod owned;

/// BLP0 fields shared by both content encodings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Blp0Header {
    /// Width of mipmap level zero, in pixels.
    pub width: u32,
    /// Height of mipmap level zero, in pixels.
    pub height: u32,
    /// Alpha depth recorded in the container header, in bits per pixel.
    pub alpha_bits: u32,
    /// Encoding-specific extra header word, preserved during round trips.
    pub extra: u32,
    /// Raw mipmap presence flag from the BLP0 header.
    pub has_mipmaps: u32,
}

/// BLP0 content borrowing its original bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blp0ContentRef<'a> {
    /// JPEG payloads with a header prefix shared by all mipmaps.
    Jpeg {
        /// JPEG prefix prepended to each mipmap payload before decoding.
        shared_header: &'a [u8],
    },
    /// Palette indices with a separate packed alpha plane.
    Indexed {
        /// The 256-entry palette, stored as four BGRA bytes per entry.
        palette: &'a [u8; PALETTE_BYTES],
    },
}

/// Editable BLP0 content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blp0Content {
    /// JPEG payloads with a header prefix shared by all mipmaps.
    Jpeg {
        /// JPEG prefix prepended to each mipmap payload before decoding.
        shared_header: Vec<u8>,
    },
    /// Palette indices with a separate packed alpha plane.
    Indexed {
        /// The 256-entry palette, stored as four BGRA bytes per entry.
        palette: Box<[u8; PALETTE_BYTES]>,
    },
}

/// Borrowed BLP0 root metadata and externally supplied mipmaps.
///
/// [`Self::read`] leaves all mipmap slots empty. Attach payloads from sibling
/// `.b00`, `.b01`, and subsequent files before decoding those levels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp0Ref<'a> {
    /// Dimensions and encoding-independent header fields.
    pub header: Blp0Header,
    /// Borrowed encoding metadata shared by all mipmaps.
    pub content: Blp0ContentRef<'a>,
    /// Encoded mipmap payloads, with full-resolution level zero first; `None` denotes an absent level.
    pub mipmaps: [Option<&'a [u8]>; MIPMAP_SLOTS],
}

/// Editable BLP0 root metadata and externally supplied mipmaps.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp0 {
    /// Dimensions and encoding-independent header fields.
    pub header: Blp0Header,
    /// Owned encoding metadata shared by all mipmaps.
    pub content: Blp0Content,
    /// Encoded mipmap payloads, with full-resolution level zero first; `None` denotes an absent level.
    pub mipmaps: [Option<Vec<u8>>; MIPMAP_SLOTS],
}
