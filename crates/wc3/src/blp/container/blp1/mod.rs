//! BLP1 container types.
use crate::blp::{MIPMAP_SLOTS, PALETTE_BYTES};

mod codec;
mod owned;

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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blp1ContentRef<'a> {
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

/// Editable BLP1 content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blp1Content {
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

/// Borrowed BLP1 container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp1Ref<'a> {
    /// Dimensions and encoding-independent header fields.
    pub header: Blp1Header,
    /// Borrowed encoding metadata shared by all mipmaps.
    pub content: Blp1ContentRef<'a>,
    /// Encoded mipmap payloads, with full-resolution level zero first; `None` denotes an absent level.
    pub mipmaps: [Option<&'a [u8]>; MIPMAP_SLOTS],
}

/// Editable BLP1 container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp1 {
    /// Dimensions and encoding-independent header fields.
    pub header: Blp1Header,
    /// Owned encoding metadata shared by all mipmaps.
    pub content: Blp1Content,
    /// Encoded mipmap payloads, with full-resolution level zero first; `None` denotes an absent level.
    pub mipmaps: [Option<Vec<u8>>; MIPMAP_SLOTS],
}
