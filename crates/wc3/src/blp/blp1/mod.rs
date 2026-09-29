//! BLP1 container types.
use super::{MIPMAP_SLOTS, PALETTE_BYTES};

mod codec;
mod owned;

/// BLP1 fields shared by both content encodings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Blp1Header {
    pub width: u32,
    pub height: u32,
    pub alpha_bits: u32,
    pub extra: u32,
    pub has_mipmaps: u32,
}

/// BLP1 content borrowing its original bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Blp1ContentRef<'a> {
    Jpeg { shared_header: &'a [u8] },
    Indexed { palette: &'a [u8; PALETTE_BYTES] },
}

/// Editable BLP1 content.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blp1Content {
    Jpeg { shared_header: Vec<u8> },
    Indexed { palette: Box<[u8; PALETTE_BYTES]> },
}

/// Borrowed BLP1 container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp1Ref<'a> {
    pub header: Blp1Header,
    pub content: Blp1ContentRef<'a>,
    pub mipmaps: [Option<&'a [u8]>; MIPMAP_SLOTS],
}

/// Editable BLP1 container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Blp1 {
    pub header: Blp1Header,
    pub content: Blp1Content,
    pub mipmaps: [Option<Vec<u8>>; MIPMAP_SLOTS],
}
