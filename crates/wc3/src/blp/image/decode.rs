//! Optional conversion of encoded BLP mipmaps to and from RGBA pixels.
use super::super::{Blp, Blp1ContentRef, Blp1Ref, Blp2ContentRef, Blp2Ref, BlpRef, MIPMAP_SLOTS};

use super::{bgra, dxt, indexed, jpeg};
use image::RgbaImage;
use std::{error::Error, fmt};

/// Maximum decoded RGBA buffer size, including JPEG decoder output.
pub(super) const MAX_DECODE_BYTES: usize = 512 * 1024 * 1024;

/// A failure while converting an encoded mipmap to RGBA pixels.
#[derive(Debug)]
pub enum DecodeError {
    /// A required or requested mipmap level is absent.
    MissingMipmap {
        /// Zero-based mipmap level, with level zero at full resolution.
        level: usize,
    },
    /// An image dimension is zero or the mipmap dimensions are invalid.
    InvalidDimensions,
    /// Encoded pixels or the supplied output buffer have an invalid layout.
    InvalidData {
        /// Name of the invalid field.
        field: &'static str,
    },
    /// The requested alpha depth is unsupported by the pixel decoder.
    UnsupportedAlphaDepth {
        /// Unsupported alpha depth, in bits per pixel.
        depth: u32,
    },
    /// The image exceeds the allocation or size limit.
    LimitExceeded,
    /// The JPEG codec failed; the contained string describes its failure.
    Jpeg(String),
    /// The JPEG stream does not have the four unconverted components required by BLP.
    UnsupportedJpegColor,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMipmap { level } => write!(f, "missing BLP mipmap {level}"),
            Self::InvalidDimensions => f.write_str("invalid BLP mipmap dimensions"),
            Self::InvalidData { field } => write!(f, "invalid BLP {field}"),
            Self::UnsupportedAlphaDepth { depth } => {
                write!(f, "unsupported BLP alpha depth {depth}")
            }
            Self::LimitExceeded => f.write_str("BLP decoded image exceeds the size limit"),
            Self::Jpeg(message) => write!(f, "BLP JPEG decode failed: {message}"),
            Self::UnsupportedJpegColor => {
                f.write_str("BLP JPEG does not contain four unconverted components")
            }
        }
    }
}

impl Error for DecodeError {}

pub(super) fn dimensions(width: u32, height: u32, level: usize) -> Result<(u32, u32), DecodeError> {
    if width == 0 || height == 0 || level >= MIPMAP_SLOTS {
        return Err(DecodeError::InvalidDimensions);
    }
    let width = (width >> level).max(1);
    let height = (height >> level).max(1);
    rgba_len(width, height)?;
    Ok((width, height))
}

fn rgba_len(width: u32, height: u32) -> Result<usize, DecodeError> {
    let len = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(DecodeError::LimitExceeded)?;
    if len > MAX_DECODE_BYTES {
        return Err(DecodeError::LimitExceeded);
    }
    Ok(len)
}

pub(super) fn output(width: u32, height: u32) -> Result<Vec<u8>, DecodeError> {
    let len = rgba_len(width, height)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(len)
        .map_err(|_| DecodeError::LimitExceeded)?;
    pixels.resize(len, 0);
    Ok(pixels)
}

pub(super) fn image(width: u32, height: u32, pixels: Vec<u8>) -> Result<RgbaImage, DecodeError> {
    RgbaImage::from_raw(width, height, pixels).ok_or(DecodeError::InvalidDimensions)
}

fn check_output(pixels: &[u8], width: u32, height: u32) -> Result<(), DecodeError> {
    if pixels.len() != rgba_len(width, height)? {
        return Err(DecodeError::InvalidData {
            field: "RGBA output size",
        });
    }
    Ok(())
}

fn mip<'a>(mips: &[Option<&'a [u8]>; MIPMAP_SLOTS], level: usize) -> Result<&'a [u8], DecodeError> {
    mips.get(level)
        .and_then(|mip| *mip)
        .ok_or(DecodeError::MissingMipmap { level })
}

impl Blp1Ref<'_> {
    /// Decodes one present mipmap to an image-rs RGBA image.
    ///
    /// `level` is zero-based, with level zero at full resolution. Returns an
    /// error for missing levels, malformed pixels, or exceeded allocation limits.
    pub fn decode_mip(&self, level: usize) -> Result<RgbaImage, DecodeError> {
        let (width, height) = dimensions(self.header.width, self.header.height, level)?;
        let mut pixels = output(width, height)?;
        self.decode_mip_into(level, &mut pixels)?;
        image(width, height, pixels)
    }

    /// Decodes one present mipmap into a caller-provided RGBA8 buffer.
    ///
    /// `level` is zero-based. The buffer must contain exactly four bytes per
    /// pixel at that level, in row-major red, green, blue, alpha order.
    /// Invalid dimensions, missing levels, malformed pixels, or an incorrectly
    /// sized buffer return an error.
    pub fn decode_mip_into(&self, level: usize, pixels: &mut [u8]) -> Result<(), DecodeError> {
        let (width, height) = dimensions(self.header.width, self.header.height, level)?;
        check_output(pixels, width, height)?;
        let data = mip(&self.mipmaps, level)?;
        match self.content {
            Blp1ContentRef::Indexed { palette } => {
                indexed::decode_into(data, palette, self.header.alpha_bits, pixels)
            }
            Blp1ContentRef::Jpeg { shared_header } => jpeg::decode_into(
                data,
                shared_header,
                self.header.alpha_bits == 0,
                width,
                height,
                pixels,
            ),
        }
    }
}

impl Blp2Ref<'_> {
    /// Decodes one present mipmap to an image-rs RGBA image.
    ///
    /// `level` is zero-based, with level zero at full resolution. Returns an
    /// error for missing levels, malformed pixels, or exceeded allocation limits.
    pub fn decode_mip(&self, level: usize) -> Result<RgbaImage, DecodeError> {
        let (width, height) = dimensions(self.header.width, self.header.height, level)?;
        let mut pixels = output(width, height)?;
        self.decode_mip_into(level, &mut pixels)?;
        image(width, height, pixels)
    }

    /// Decodes one present mipmap into a caller-provided RGBA8 buffer.
    ///
    /// `level` is zero-based. The buffer must contain exactly four bytes per
    /// pixel at that level, in row-major red, green, blue, alpha order.
    /// Invalid dimensions, missing levels, malformed pixels, or an incorrectly
    /// sized buffer return an error.
    pub fn decode_mip_into(&self, level: usize, pixels: &mut [u8]) -> Result<(), DecodeError> {
        let (width, height) = dimensions(self.header.width, self.header.height, level)?;
        check_output(pixels, width, height)?;
        let data = mip(&self.mipmaps, level)?;
        match self.content {
            Blp2ContentRef::Indexed { palette, .. } => {
                indexed::decode_into(data, palette, u32::from(self.header.alpha_bits), pixels)
            }
            Blp2ContentRef::Jpeg { shared_header, .. } => jpeg::decode_into(
                data,
                shared_header,
                self.header.alpha_bits == 0,
                width,
                height,
                pixels,
            ),
            Blp2ContentRef::Dxt { format, .. } => dxt::decode_into(
                data,
                format,
                self.header.alpha_bits != 0,
                width,
                height,
                pixels,
            ),
            Blp2ContentRef::Bgra { .. } => bgra::decode_into(data, pixels),
        }
    }
}

impl BlpRef<'_> {
    /// Decodes one present mipmap to an image-rs RGBA image.
    ///
    /// `level` is zero-based, with level zero at full resolution. Returns an
    /// error for missing levels, malformed pixels, or exceeded allocation limits.
    pub fn decode_mip(&self, level: usize) -> Result<RgbaImage, DecodeError> {
        match self {
            Self::Blp1(value) => value.decode_mip(level),
            Self::Blp2(value) => value.decode_mip(level),
        }
    }

    /// Decodes one present mipmap into a caller-provided RGBA8 buffer.
    ///
    /// `level` is zero-based. The buffer must contain exactly four bytes per
    /// pixel at that level, in row-major red, green, blue, alpha order.
    /// Invalid dimensions, missing levels, malformed pixels, or an incorrectly
    /// sized buffer return an error.
    pub fn decode_mip_into(&self, level: usize, pixels: &mut [u8]) -> Result<(), DecodeError> {
        match self {
            Self::Blp1(value) => value.decode_mip_into(level, pixels),
            Self::Blp2(value) => value.decode_mip_into(level, pixels),
        }
    }
}

impl Blp {
    /// Decodes one present mipmap to an image-rs RGBA image.
    ///
    /// `level` is zero-based, with level zero at full resolution. Returns an
    /// error for missing levels, malformed pixels, or exceeded allocation limits.
    pub fn decode_mip(&self, level: usize) -> Result<RgbaImage, DecodeError> {
        self.as_ref().decode_mip(level)
    }

    /// Decodes one present mipmap into a caller-provided RGBA8 buffer.
    ///
    /// `level` is zero-based. The buffer must contain exactly four bytes per
    /// pixel at that level, in row-major red, green, blue, alpha order.
    /// Invalid dimensions, missing levels, malformed pixels, or an incorrectly
    /// sized buffer return an error.
    pub fn decode_mip_into(&self, level: usize, pixels: &mut [u8]) -> Result<(), DecodeError> {
        self.as_ref().decode_mip_into(level, pixels)
    }
}
