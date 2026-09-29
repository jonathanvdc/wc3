//! BLP2 container types.
use crate::blp::{MIPMAP_SLOTS, PALETTE_BYTES};

mod codec;
mod owned;

/// BLP2 fields shared by all content encodings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Blp2Header {
    pub width: u32,
    pub height: u32,
    pub alpha_bits: u8,
    pub mipmap_flags: u8,
}

/// Supported BLP2 DXT block formats.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DxtFormat {
    Dxt1,
    Dxt3,
    Dxt5,
}

/// BLP2 content borrowing its original bytes. The unused region is retained
/// because BLP2 always has a 1024-byte content area.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blp2ContentRef<'a> {
    Jpeg {
        alpha_type: u8,
        shared_header: &'a [u8],
        unused: &'a [u8],
    },
    Indexed {
        alpha_type: u8,
        palette: &'a [u8; PALETTE_BYTES],
    },
    Dxt {
        format: DxtFormat,
        palette_region: &'a [u8; PALETTE_BYTES],
    },
    Bgra {
        encoding: u8,
        alpha_type: u8,
        palette_region: &'a [u8; PALETTE_BYTES],
    },
}

/// Editable BLP2 content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blp2Content {
    Jpeg {
        alpha_type: u8,
        shared_header: Vec<u8>,
        unused: Vec<u8>,
    },
    Indexed {
        alpha_type: u8,
        palette: Box<[u8; PALETTE_BYTES]>,
    },
    Dxt {
        format: DxtFormat,
        palette_region: Box<[u8; PALETTE_BYTES]>,
    },
    Bgra {
        encoding: u8,
        alpha_type: u8,
        palette_region: Box<[u8; PALETTE_BYTES]>,
    },
}

/// Borrowed BLP2 container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp2Ref<'a> {
    pub header: Blp2Header,
    pub content: Blp2ContentRef<'a>,
    pub mipmaps: [Option<&'a [u8]>; MIPMAP_SLOTS],
}

/// Editable BLP2 container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp2 {
    pub header: Blp2Header,
    pub content: Blp2Content,
    pub mipmaps: [Option<Vec<u8>>; MIPMAP_SLOTS],
}
