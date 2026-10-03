//! BLP2 container types.
use crate::blp::{MIPMAP_SLOTS, PALETTE_BYTES};

mod codec;
mod owned;

/// BLP2 fields shared by all content encodings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Blp2Header {
    /// Width of mipmap level zero, in pixels.
    pub width: u32,
    /// Height of mipmap level zero, in pixels.
    pub height: u32,
    /// Alpha depth recorded in the container header, in bits per pixel.
    pub alpha_bits: u8,
    /// Raw mipmap flags from the BLP2 header.
    pub mipmap_flags: u8,
}

/// Supported BLP2 DXT block formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DxtFormat {
    /// BC1 blocks with opaque or one-bit alpha.
    Dxt1,
    /// BC2 blocks with explicit four-bit alpha.
    Dxt3,
    /// BC3 blocks with interpolated alpha.
    Dxt5,
}

/// BLP2 content borrowing its original bytes. The unused region is retained
/// because BLP2 always has a 1024-byte content area.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blp2ContentRef<'a> {
    /// JPEG payloads with a header prefix shared by all mipmaps.
    Jpeg {
        /// Raw alpha encoding selector retained from the BLP2 header.
        alpha_type: u8,
        /// JPEG prefix prepended to each mipmap payload before decoding.
        shared_header: &'a [u8],
        /// Unused bytes retained from the fixed-size content region.
        unused: &'a [u8],
    },
    /// Palette indices with a separate packed alpha plane.
    Indexed {
        /// Raw alpha encoding selector retained from the BLP2 header.
        alpha_type: u8,
        /// The 256-entry palette, stored as four BGRA bytes per entry.
        palette: &'a [u8; PALETTE_BYTES],
    },
    /// Block-compressed pixel payloads.
    Dxt {
        /// Block compression format used by every mipmap.
        format: DxtFormat,
        /// Unused palette-sized region, preserved during round trips.
        palette_region: &'a [u8; PALETTE_BYTES],
    },
    /// Uncompressed BGRA pixel payloads.
    Bgra {
        /// Raw BLP2 pixel encoding selector.
        encoding: u8,
        /// Raw alpha encoding selector retained from the BLP2 header.
        alpha_type: u8,
        /// Unused palette-sized region, preserved during round trips.
        palette_region: &'a [u8; PALETTE_BYTES],
    },
}

/// Editable BLP2 content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blp2Content {
    /// JPEG payloads with a header prefix shared by all mipmaps.
    Jpeg {
        /// Raw alpha encoding selector retained from the BLP2 header.
        alpha_type: u8,
        /// JPEG prefix prepended to each mipmap payload before decoding.
        shared_header: Vec<u8>,
        /// Unused bytes retained from the fixed-size content region.
        unused: Vec<u8>,
    },
    /// Palette indices with a separate packed alpha plane.
    Indexed {
        /// Raw alpha encoding selector retained from the BLP2 header.
        alpha_type: u8,
        /// The 256-entry palette, stored as four BGRA bytes per entry.
        palette: Box<[u8; PALETTE_BYTES]>,
    },
    /// Block-compressed pixel payloads.
    Dxt {
        /// Block compression format used by every mipmap.
        format: DxtFormat,
        /// Unused palette-sized region, preserved during round trips.
        palette_region: Box<[u8; PALETTE_BYTES]>,
    },
    /// Uncompressed BGRA pixel payloads.
    Bgra {
        /// Raw BLP2 pixel encoding selector.
        encoding: u8,
        /// Raw alpha encoding selector retained from the BLP2 header.
        alpha_type: u8,
        /// Unused palette-sized region, preserved during round trips.
        palette_region: Box<[u8; PALETTE_BYTES]>,
    },
}

/// Borrowed BLP2 container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp2Ref<'a> {
    /// Dimensions and encoding-independent header fields.
    pub header: Blp2Header,
    /// Borrowed encoding metadata shared by all mipmaps.
    pub content: Blp2ContentRef<'a>,
    /// Encoded mipmap payloads, with full-resolution level zero first; `None` denotes an absent level.
    pub mipmaps: [Option<&'a [u8]>; MIPMAP_SLOTS],
}

/// Editable BLP2 container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp2 {
    /// Dimensions and encoding-independent header fields.
    pub header: Blp2Header,
    /// Owned encoding metadata shared by all mipmaps.
    pub content: Blp2Content,
    /// Encoded mipmap payloads, with full-resolution level zero first; `None` denotes an absent level.
    pub mipmaps: [Option<Vec<u8>>; MIPMAP_SLOTS],
}
