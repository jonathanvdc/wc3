//! Read and write BLP1 and BLP2 texture containers.
//!
//! [`BlpRef::read`] borrows encoded mipmap data from an input byte slice.
//! Call [`BlpRef::to_owned`] to edit the container, or write the borrowed view
//! directly. Writing lays out mipmaps in level order and recalculates offsets;
//! it does not reproduce arbitrary source padding or mipmap placement.
//! Container types retain format fields and encoded bytes, including unused
//! regions. Image encoding validates format settings, dimensions, and mipmaps;
//! reading a container does not assert that every mipmap can be decoded.
//! With `blp-decode`, `decode_mip` converts encoded mipmaps to
//! [`image::RgbaImage`]. With `blp-encode`, `encode_image` creates a container
//! from RGBA pixels.

mod container;
mod error;
#[cfg(any(feature = "blp-decode", feature = "blp-encode"))]
mod image;

pub use container::blp1::{Blp1, Blp1Content, Blp1ContentRef, Blp1Header, Blp1Ref};
pub use container::blp2::{Blp2, Blp2Content, Blp2ContentRef, Blp2Header, Blp2Ref, DxtFormat};
pub use error::{ReadError, ReadErrorKind, WriteError};
#[cfg(feature = "blp-decode")]
pub use image::DecodeError;
#[cfg(feature = "blp-encode")]
pub use image::{BlpVersion, EncodeError, EncodeFormat, EncodeOptions, IndexedAlpha};

/// Number of slots in a BLP mipmap location table.
pub const MIPMAP_SLOTS: usize = 16;
/// Size of a BLP palette or BLP2 content region.
pub const PALETTE_BYTES: usize = 1024;

/// A borrowed BLP1 or BLP2 texture container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlpRef<'a> {
    Blp1(Blp1Ref<'a>),
    Blp2(Blp2Ref<'a>),
}

/// An editable BLP1 or BLP2 texture container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blp {
    Blp1(Blp1),
    Blp2(Blp2),
}

impl<'a> BlpRef<'a> {
    /// Parses a complete BLP1 or BLP2 file, borrowing its content bytes.
    pub fn read(bytes: &'a [u8]) -> Result<Self, ReadError> {
        match container::io::slice(bytes, 0, 4)? {
            b"BLP1" => Blp1Ref::read(bytes).map(Self::Blp1),
            b"BLP2" => Blp2Ref::read(bytes).map(Self::Blp2),
            _ => Err(ReadError::new(0, ReadErrorKind::InvalidMagic)),
        }
    }

    /// Copies encoded content into an editable container.
    pub fn to_owned(&self) -> Blp {
        match self {
            Self::Blp1(value) => Blp::Blp1(value.to_owned()),
            Self::Blp2(value) => Blp::Blp2(value.to_owned()),
        }
    }

    /// Writes a canonical layout without copying mipmaps into an owned `Blp`.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        match self {
            Self::Blp1(value) => value.write(),
            Self::Blp2(value) => value.write(),
        }
    }
}

impl Blp {
    /// Borrows the current encoded content and mipmaps.
    pub fn as_ref(&self) -> BlpRef<'_> {
        match self {
            Self::Blp1(value) => BlpRef::Blp1(value.as_ref()),
            Self::Blp2(value) => BlpRef::Blp2(value.as_ref()),
        }
    }

    /// Writes the texture with recalculated mipmap offsets and sizes.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        self.as_ref().write()
    }
}
