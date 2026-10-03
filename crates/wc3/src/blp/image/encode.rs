//! RGBA image encoding for BLP1 and BLP2.
use super::{assemble, bgra, dxt, indexed, jpeg, mipmaps};
use crate::blp::{Blp, DxtFormat, MIPMAP_SLOTS};
use image::RgbaImage;
use std::{error::Error, fmt};

/// BLP container version for formats available in both versions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlpVersion {
    /// The original BLP1 container format.
    Blp1,
    /// The BLP2 container format.
    Blp2,
}

/// Alpha storage supported by indexed BLP images.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexedAlpha {
    /// No alpha storage; every pixel is opaque.
    Opaque,
    /// One alpha bit per pixel, representing transparency or opacity.
    Bit1,
    /// Four alpha bits per pixel.
    Bit4,
    /// Eight alpha bits per pixel.
    Bit8,
}

impl IndexedAlpha {
    /// Returns the number of stored alpha bits per pixel.
    pub const fn bits(self) -> u8 {
        match self {
            Self::Opaque => 0,
            Self::Bit1 => 1,
            Self::Bit4 => 4,
            Self::Bit8 => 8,
        }
    }
}

/// Pixel encoding, with only the settings supported by that encoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeFormat {
    /// Palette-indexed pixels with packed alpha storage.
    Indexed {
        /// Container version to produce.
        version: BlpVersion,
        /// Alpha precision for the separate alpha plane.
        alpha: IndexedAlpha,
    },
    /// JPEG-compressed color and optional alpha components.
    Jpeg {
        /// Container version to produce.
        version: BlpVersion,
        /// Whether to preserve alpha instead of forcing opaque pixels.
        alpha: bool,
        /// JPEG quality from 1 (lowest) through 100 (highest).
        quality: u8,
    },
    /// BC1 block compression in a BLP2 container.
    Dxt1 {
        /// Whether to preserve alpha instead of forcing opaque pixels.
        alpha: bool,
    },
    /// BC2 block compression with explicit alpha in a BLP2 container.
    Dxt3,
    /// BC3 block compression with interpolated alpha in a BLP2 container.
    Dxt5,
    /// Uncompressed BGRA pixels in a BLP2 container.
    Bgra,
}

impl EncodeFormat {
    /// Returns the alpha depth recorded for this encoding, in bits per pixel.
    pub const fn alpha_bits(self) -> u8 {
        match self {
            Self::Indexed { alpha, .. } => alpha.bits(),
            Self::Jpeg { alpha, .. } => {
                if alpha {
                    8
                } else {
                    0
                }
            }
            Self::Dxt1 { alpha } => {
                if alpha {
                    1
                } else {
                    0
                }
            }
            Self::Dxt3 => 4,
            Self::Dxt5 | Self::Bgra => 8,
        }
    }
}

/// Options for turning RGBA pixels into a BLP container.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EncodeOptions {
    /// Container version, pixel encoding, and encoding-specific settings.
    pub format: EncodeFormat,
    /// Generate mipmaps when encoding an image; validate a complete chain when encoding supplied levels.
    pub mipmaps: bool,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            format: EncodeFormat::Indexed {
                version: BlpVersion::Blp1,
                alpha: IndexedAlpha::Bit8,
            },
            mipmaps: true,
        }
    }
}

/// An image or option that cannot be encoded as the requested BLP format.
#[derive(Debug)]
pub enum EncodeError {
    /// A supplied image has zero or unsupported dimensions.
    InvalidDimensions,
    /// JPEG quality is outside the inclusive range 1 through 100.
    InvalidJpegQuality,
    /// The mipmap count or dimensions do not form the requested chain.
    InvalidMipmaps,
    /// The encoded image exceeds the supported size limit.
    LimitExceeded,
    /// The JPEG encoder failed; the contained string describes its failure.
    Jpeg(String),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => f.write_str("invalid BLP image dimensions"),
            Self::InvalidJpegQuality => f.write_str("JPEG quality must be 1 through 100"),
            Self::InvalidMipmaps => f.write_str("invalid BLP mipmap sequence"),
            Self::LimitExceeded => f.write_str("BLP image exceeds size limit"),
            Self::Jpeg(message) => write!(f, "BLP JPEG encoding failed: {message}"),
        }
    }
}
impl Error for EncodeError {}

fn validate(images: &[RgbaImage], options: EncodeOptions) -> Result<(), EncodeError> {
    mipmaps::validate(images, options.mipmaps)?;
    if let EncodeFormat::Jpeg { quality, .. } = options.format {
        if !(1..=100).contains(&quality) {
            return Err(EncodeError::InvalidJpegQuality);
        }
    }
    Ok(())
}

impl Blp {
    /// Encodes an RGBA image, optionally generating mipmaps through 1×1.
    pub fn encode_image(image: &RgbaImage, options: EncodeOptions) -> Result<Self, EncodeError> {
        let images = mipmaps::generate(image, options.mipmaps);
        Self::encode_mipmaps(&images, options)
    }

    /// Encodes a caller-supplied mipmap sequence, with level zero first.
    ///
    /// Each subsequent level must halve both dimensions, clamped to one pixel.
    /// When `options.mipmaps` is enabled, the chain must end at 1×1; otherwise
    /// exactly one image is required. At most 16 levels are accepted.
    /// Returns an error for invalid dimensions, chains, or JPEG settings.
    pub fn encode_mipmaps(
        images: &[RgbaImage],
        options: EncodeOptions,
    ) -> Result<Self, EncodeError> {
        validate(images, options)?;
        let mut mips: [Option<Vec<u8>>; MIPMAP_SLOTS] = std::array::from_fn(|_| None);
        let palette = if matches!(options.format, EncodeFormat::Indexed { .. }) {
            let (palette, encoded) = indexed::encode(images, options.format.alpha_bits());
            for (slot, mip) in mips.iter_mut().zip(encoded) {
                *slot = Some(mip);
            }
            Some(palette)
        } else {
            for (slot, image) in mips.iter_mut().zip(images) {
                *slot = Some(match options.format {
                    EncodeFormat::Jpeg { alpha, quality, .. } => {
                        jpeg::encode(image, u8::from(alpha) * 8, quality)?
                    }
                    EncodeFormat::Dxt1 { alpha } => {
                        dxt::encode(image, DxtFormat::Dxt1, u8::from(alpha))
                    }
                    EncodeFormat::Dxt3 => dxt::encode(image, DxtFormat::Dxt3, 4),
                    EncodeFormat::Dxt5 => dxt::encode(image, DxtFormat::Dxt5, 8),
                    EncodeFormat::Bgra => bgra::encode(image),
                    EncodeFormat::Indexed { .. } => unreachable!(),
                });
            }
            None
        };
        Ok(assemble::container(
            images[0].width(),
            images[0].height(),
            options.format,
            palette,
            mips,
        ))
    }
}
