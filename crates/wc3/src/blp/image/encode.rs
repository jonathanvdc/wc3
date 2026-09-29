//! RGBA image encoding for BLP1 and BLP2.
use super::{assemble, bgra, dxt, indexed, jpeg, mipmaps};
use crate::blp::{Blp, DxtFormat, MIPMAP_SLOTS};
use image::RgbaImage;
use std::{error::Error, fmt};

/// BLP container version for formats available in both versions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlpVersion {
    Blp1,
    Blp2,
}

/// Alpha storage supported by indexed BLP images.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IndexedAlpha {
    Opaque,
    Bit1,
    Bit4,
    Bit8,
}

impl IndexedAlpha {
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
    Indexed {
        version: BlpVersion,
        alpha: IndexedAlpha,
    },
    Jpeg {
        version: BlpVersion,
        alpha: bool,
        quality: u8,
    },
    Dxt1 {
        alpha: bool,
    },
    Dxt3,
    Dxt5,
    Bgra,
}

impl EncodeFormat {
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
    pub format: EncodeFormat,
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
    InvalidDimensions,
    InvalidJpegQuality,
    InvalidMipmaps,
    LimitExceeded,
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
