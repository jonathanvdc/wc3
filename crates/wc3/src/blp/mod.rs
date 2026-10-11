//! Read and write BLP0, BLP1, and BLP2 texture containers.
//!
//! [`BlpRef::read`](crate::blp::BlpRef::read) borrows encoded content from an input byte slice. BLP1 and
//! BLP2 include mipmaps in that slice; BLP0 requires external payloads, described
//! below. Call [`BlpRef::to_owned`](crate::blp::BlpRef::to_owned) to edit the container, or serialize the
//! borrowed view directly. Writing BLP1 and BLP2 lays out mipmaps in level order
//! and recalculates offsets; writing any version discards arbitrary source
//! padding rather than reproducing the original file layout.
//! Container types retain format fields and encoded bytes, including unused
//! regions. Image encoding validates format settings, dimensions, and mipmaps;
//! reading a container does not assert that every mipmap can be decoded.
//! With `blp-decode`, `decode_mip` converts encoded mipmaps to
//! `image::RgbaImage`. With `blp-encode`, `encode_image` creates a container
//! from RGBA pixels.
//!
//! # External BLP0 mipmaps
//!
//! BLP0 stores its palette or shared JPEG prefix in the root `.blp` file and
//! its encoded mipmaps in sibling files named `texture.b00`, `texture.b01`,
//! and so on. Root parsing leaves all mipmap slots empty. Applications resolve
//! and attach only the levels they need, whether from files, archives, or memory:
//!
//! ```
//! # fn example(root: &[u8], base: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
//! use wc3::blp::BlpRef;
//!
//! let mut texture = BlpRef::read(root)?;
//! if let BlpRef::Blp0(value) = &mut texture {
//!     value.mipmaps[0] = Some(base);
//! }
//! # #[cfg(feature = "blp-decode")]
//! let rgba = texture.decode_mip(0)?;
//! let output = texture.write_files()?;
//! std::fs::write("texture.blp", output.root)?;
//! for (level, payload) in output.mipmaps.iter().enumerate() {
//!     if let Some(bytes) = payload {
//!         std::fs::write(format!("texture.b{level:02}"), bytes)?;
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! `write_files` also supports root-only metadata editing without attached
//! mipmaps. Single-file `write` rejects BLP0 to prevent silently discarding
//! external payloads. For BLP1 and BLP2, `write_files` embeds mipmaps in the root
//! and leaves the external slots empty. BLP0 JPEG parsing uses the declared
//! shared-prefix length and ignores trailing root padding. Compatibility with
//! real beta-game BLP0 textures remains unverified.
//!
//! # `image` crate adapters
//!
//! With `blp-decode`, `BlpDecoder` implements `image::ImageDecoder`. It
//! selects level zero by default; `BlpDecoder::with_mip` selects
//! another level. It writes RGBA8 pixels into the buffer supplied by `image`.
//! Use `BlpRef::decode_mip_into` when you supply the pixel buffer yourself.
//! JPEG decoding still uses a temporary CMYK buffer inside the JPEG decoder.
//!
//! ```
//! # #[cfg(feature = "blp-decode")]
//! # fn example(bytes: &[u8]) -> Result<image::DynamicImage, image::ImageError> {
//! use wc3::blp::BlpDecoder;
//! image::DynamicImage::from_decoder(BlpDecoder::new(bytes)?)
//! # }
//! ```
//! Call `register_decoding_hook` once to let `image` load `.blp` paths and
//! detect BLP0/BLP1/BLP2 signatures. The hook cannot load external BLP0 payloads
//! and reports a missing mipmap; use `BlpDecoder::from_blp` after attaching them.
//!
//! With `blp-encode`, `BlpEncoder` implements `image::ImageEncoder` for
//! RGBA8 pixels. Use `BlpEncoder::with_options` to select the BLP encoding
//! format; `BlpEncoder::new` uses default options. This single-writer adapter
//! rejects BLP0. Use `Blp::encode_image` and `write_files` for BLP0 output.
//!
//! `EncodeOptions::mipmap_filter` selects the resampling filter for generated
//! levels, defaulting to `MipmapFilter::Triangle`. Supplied mipmaps are used
//! as authored. `EncodeOptions::dxt_algorithm` selects `DxtAlgorithm` for
//! DXT1/3/5 encoding: range fit prioritizes speed, cluster fit is the default,
//! and iterative cluster fit spends more time refining the color fit. This
//! setting has no effect on indexed, JPEG, or BGRA encoding.
//!
//! ```
//! # #[cfg(feature = "blp-encode")]
//! # fn example(image: &image::RgbaImage) -> Result<Vec<u8>, image::ImageError> {
//! use image::{ExtendedColorType, ImageEncoder};
//! use wc3::blp::BlpEncoder;
//!
//! let mut bytes = Vec::new();
//! BlpEncoder::new(&mut bytes).write_image(
//!     image.as_raw(), image.width(), image.height(), ExtendedColorType::Rgba8,
//! )?;
//! Ok(bytes)
//! # }
//! ```

mod container;
mod error;
#[cfg(any(feature = "blp-decode", feature = "blp-encode"))]
mod image;

pub use container::blp0::{Blp0, Blp0Content, Blp0ContentRef, Blp0Header, Blp0Ref};
pub use container::blp1::{Blp1, Blp1Content, Blp1ContentRef, Blp1Header, Blp1Ref};
pub use container::blp2::{Blp2, Blp2Content, Blp2ContentRef, Blp2Header, Blp2Ref, DxtFormat};
pub use error::{ReadError, ReadErrorKind, WriteError};
#[cfg(feature = "blp-encode")]
pub use image::BlpEncoder;
#[cfg(feature = "blp-decode")]
pub use image::DecodeError;
#[cfg(feature = "blp-decode")]
pub use image::{register_decoding_hook, BlpDecoder};
#[cfg(feature = "blp-encode")]
pub use image::{
    BlpVersion, DxtAlgorithm, EncodeError, EncodeFormat, EncodeOptions, IndexedAlpha, MipmapFilter,
};

/// Number of slots in a BLP mipmap location table.
pub const MIPMAP_SLOTS: usize = 16;
/// Size of a BLP palette or BLP2 content region.
pub const PALETTE_BYTES: usize = 1024;

/// A borrowed BLP texture container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BlpRef<'a> {
    /// A BLP0 container with externally supplied mipmaps.
    Blp0(Blp0Ref<'a>),
    /// A borrowed BLP1 container.
    Blp1(Blp1Ref<'a>),
    /// A borrowed BLP2 container.
    Blp2(Blp2Ref<'a>),
}

/// An editable BLP texture container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Blp {
    /// A BLP0 container with externally supplied mipmaps.
    Blp0(Blp0),
    /// An owned BLP1 container.
    Blp1(Blp1),
    /// An owned BLP2 container.
    Blp2(Blp2),
}

impl<'a> BlpRef<'a> {
    /// Parses a BLP root file, borrowing its encoded content.
    ///
    /// BLP1 and BLP2 include their mipmaps. BLP0 leaves every mipmap slot empty;
    /// callers attach external payloads before decoding. No external I/O is performed.
    pub fn read(bytes: &'a [u8]) -> Result<Self, ReadError> {
        match container::io::slice(bytes, 0, 4)? {
            b"BLP0" => Blp0Ref::read(bytes).map(Self::Blp0),
            b"BLP1" => Blp1Ref::read(bytes).map(Self::Blp1),
            b"BLP2" => Blp2Ref::read(bytes).map(Self::Blp2),
            _ => Err(ReadError::new(0, ReadErrorKind::InvalidMagic)),
        }
    }

    /// Copies encoded content into an editable container.
    pub fn to_owned(&self) -> Blp {
        match self {
            Self::Blp0(value) => Blp::Blp0(value.to_owned()),
            Self::Blp1(value) => Blp::Blp1(value.to_owned()),
            Self::Blp2(value) => Blp::Blp2(value.to_owned()),
        }
    }

    /// Writes a canonical single-file layout for BLP1 or BLP2.
    ///
    /// Returns [`WriteError::ExternalMipmapsRequired`] for BLP0; use
    /// [`Self::write_files`] to serialize its root and external payloads.
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        match self {
            Self::Blp0(value) => value.write(),
            Self::Blp1(value) => value.write(),
            Self::Blp2(value) => value.write(),
        }
    }
}

impl Blp {
    /// Borrows the current encoded content and mipmaps.
    pub fn as_ref(&self) -> BlpRef<'_> {
        match self {
            Self::Blp0(value) => BlpRef::Blp0(value.as_ref()),
            Self::Blp1(value) => BlpRef::Blp1(value.as_ref()),
            Self::Blp2(value) => BlpRef::Blp2(value.as_ref()),
        }
    }

    /// Writes BLP1 or BLP2 with recalculated mipmap offsets and sizes.
    ///
    /// Returns [`WriteError::ExternalMipmapsRequired`] for BLP0; use [`Self::write_files`].
    pub fn write(&self) -> Result<Vec<u8>, WriteError> {
        self.as_ref().write()
    }
}

/// Serialized root file and any external mipmap payloads.
///
/// For BLP0, payload slot `level` belongs in a sibling file named with a `.bNN`
/// extension, such as `texture.b00` for level zero of `texture.blp`. File naming
/// and storage are application-owned. For BLP1 and BLP2, the root contains all
/// mipmaps and every external slot is empty.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlpOutput {
    /// Bytes of the root `.blp` file.
    pub root: Vec<u8>,
    /// External payloads by level; empty for BLP1 and BLP2.
    pub mipmaps: [Option<Vec<u8>>; MIPMAP_SLOTS],
}

impl BlpRef<'_> {
    /// Serializes the root and external mipmap payloads.
    ///
    /// BLP0 returns only the attached payloads; empty slots remain empty. BLP1
    /// and BLP2 embed their mipmaps in `root` and return empty external slots.
    pub fn write_files(&self) -> Result<BlpOutput, WriteError> {
        match self {
            Self::Blp0(value) => value.write_files(),
            _ => Ok(BlpOutput {
                root: self.write()?,
                mipmaps: std::array::from_fn(|_| None),
            }),
        }
    }
}

impl Blp {
    /// Serializes the root and external mipmap payloads.
    ///
    /// BLP0 returns only the attached payloads; empty slots remain empty. BLP1
    /// and BLP2 embed their mipmaps in `root` and return empty external slots.
    pub fn write_files(&self) -> Result<BlpOutput, WriteError> {
        self.as_ref().write_files()
    }
}
