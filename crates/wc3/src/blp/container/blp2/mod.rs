//! BLP2 container types.
use crate::blp::{MIPMAP_SLOTS, Palette};

mod codec;
mod ownership;
use std::borrow::Cow;

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

/// Supported BLP2 block compression formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DxtFormat {
    /// BC1 blocks with opaque or one-bit alpha.
    Dxt1,
    /// BC2 blocks with explicit four-bit alpha.
    Dxt3,
    /// BC3 blocks with interpolated alpha.
    Dxt5,
    /// BC5 blocks containing independent unsigned red and green channels.
    Bc5,
}

/// BLP2 encoding metadata with borrowed or owned byte storage. The unused region is retained
/// because BLP2 always has a 1024-byte content area.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blp2Content<'a> {
    /// JPEG payloads with a header prefix shared by all mipmaps.
    Jpeg {
        /// Raw alpha encoding selector retained from the BLP2 header.
        alpha_type: u8,
        /// JPEG prefix prepended to each mipmap payload before decoding.
        shared_header: Cow<'a, [u8]>,
        /// Unused bytes retained from the fixed-size content region.
        unused: Cow<'a, [u8]>,
    },
    /// Palette indices with a separate packed alpha plane.
    Indexed {
        /// Raw alpha encoding selector retained from the BLP2 header.
        alpha_type: u8,
        /// The 256-entry palette, stored as four BGRA bytes per entry.
        palette: Palette<'a>,
    },
    /// Block-compressed pixel payloads.
    Dxt {
        /// Block compression format used by every mipmap.
        format: DxtFormat,
        /// Unused palette-sized region, preserved during round trips.
        palette_region: Palette<'a>,
    },
    /// Uncompressed BGRA pixel payloads.
    Bgra {
        /// Raw BLP2 pixel encoding selector.
        encoding: u8,
        /// Raw alpha encoding selector retained from the BLP2 header.
        alpha_type: u8,
        /// Unused palette-sized region, preserved during round trips.
        palette_region: Palette<'a>,
    },
}

/// A BLP2 container with copy-on-write payloads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp2<'a> {
    /// Dimensions and encoding-independent header fields.
    pub header: Blp2Header,
    /// Encoding metadata shared by all mipmaps.
    pub content: Blp2Content<'a>,
    /// Encoded mipmap payloads, with full-resolution level zero first; `None` denotes an absent level.
    pub mipmaps: [Option<Cow<'a, [u8]>>; MIPMAP_SLOTS],
}
