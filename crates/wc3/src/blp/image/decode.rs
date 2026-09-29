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
    MissingMipmap { level: usize },
    InvalidDimensions,
    InvalidData { field: &'static str },
    UnsupportedAlphaDepth { depth: u32 },
    LimitExceeded,
    Jpeg(String),
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

fn dimensions(width: u32, height: u32, level: usize) -> Result<(u32, u32), DecodeError> {
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

fn mip<'a>(mips: &[Option<&'a [u8]>; MIPMAP_SLOTS], level: usize) -> Result<&'a [u8], DecodeError> {
    mips.get(level)
        .and_then(|mip| *mip)
        .ok_or(DecodeError::MissingMipmap { level })
}

impl Blp1Ref<'_> {
    /// Decodes one present mipmap to an image-rs RGBA image.
    pub fn decode_mip(&self, level: usize) -> Result<RgbaImage, DecodeError> {
        let (width, height) = dimensions(self.header.width, self.header.height, level)?;
        let data = mip(&self.mipmaps, level)?;
        match self.content {
            Blp1ContentRef::Indexed { palette } => {
                indexed::decode(data, palette, self.header.alpha_bits, width, height)
            }
            Blp1ContentRef::Jpeg { shared_header } => jpeg::decode(
                data,
                shared_header,
                self.header.alpha_bits == 0,
                width,
                height,
            ),
        }
    }
}

impl Blp2Ref<'_> {
    /// Decodes one present mipmap to an image-rs RGBA image.
    pub fn decode_mip(&self, level: usize) -> Result<RgbaImage, DecodeError> {
        let (width, height) = dimensions(self.header.width, self.header.height, level)?;
        let data = mip(&self.mipmaps, level)?;
        match self.content {
            Blp2ContentRef::Indexed { palette, .. } => indexed::decode(
                data,
                palette,
                u32::from(self.header.alpha_bits),
                width,
                height,
            ),
            Blp2ContentRef::Jpeg { shared_header, .. } => jpeg::decode(
                data,
                shared_header,
                self.header.alpha_bits == 0,
                width,
                height,
            ),
            Blp2ContentRef::Dxt { format, .. } => {
                dxt::decode(data, format, self.header.alpha_bits != 0, width, height)
            }
            Blp2ContentRef::Bgra { .. } => bgra::decode(data, width, height),
        }
    }
}

impl BlpRef<'_> {
    /// Decodes one present mipmap to an image-rs RGBA image.
    pub fn decode_mip(&self, level: usize) -> Result<RgbaImage, DecodeError> {
        match self {
            Self::Blp1(value) => value.decode_mip(level),
            Self::Blp2(value) => value.decode_mip(level),
        }
    }
}

impl Blp {
    /// Decodes one present mipmap to an image-rs RGBA image.
    pub fn decode_mip(&self, level: usize) -> Result<RgbaImage, DecodeError> {
        self.as_ref().decode_mip(level)
    }
}
