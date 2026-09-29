//! RGBA image encoding for BLP1 and BLP2.
use super::super::{
    Blp, Blp1, Blp1Content, Blp1Header, Blp2, Blp2Content, Blp2Header, DxtFormat, MIPMAP_SLOTS,
    PALETTE_BYTES,
};
use color_quant::NeuQuant;
use image::{imageops::FilterType, RgbaImage};
use jpeg_encoder::{ColorType, Encoder};
use squish::{Format, Params};
use std::{error::Error, fmt};

/// Pixel encoding and BLP container version.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeFormat {
    Blp1Jpeg,
    Blp1Indexed,
    Blp2Jpeg,
    Blp2Indexed,
    Blp2Dxt(DxtFormat),
    Blp2Bgra,
}

/// Options for turning an RGBA image into a BLP container.
#[derive(Clone, Copy, Debug)]
pub struct EncodeOptions {
    pub format: EncodeFormat,
    /// Indexed: 0, 1, 4, or 8. JPEG: 0 or 8. DXT alpha depth is set by format.
    pub alpha_bits: u8,
    pub mipmaps: bool,
    /// JPEG quality in 1..=100.
    pub jpeg_quality: u8,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            format: EncodeFormat::Blp1Indexed,
            alpha_bits: 8,
            mipmaps: true,
            jpeg_quality: 90,
        }
    }
}

/// An image or option that cannot be encoded as the requested BLP format.
#[derive(Debug)]
pub enum EncodeError {
    InvalidDimensions,
    InvalidAlphaDepth,
    InvalidJpegQuality,
    InvalidMipmaps,
    LimitExceeded,
    Jpeg(String),
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => f.write_str("invalid BLP image dimensions"),
            Self::InvalidAlphaDepth => f.write_str("invalid alpha depth for BLP encoding"),
            Self::InvalidJpegQuality => f.write_str("JPEG quality must be 1 through 100"),
            Self::InvalidMipmaps => f.write_str("invalid BLP mipmap sequence"),
            Self::LimitExceeded => f.write_str("BLP image exceeds size limit"),
            Self::Jpeg(message) => write!(f, "BLP JPEG encoding failed: {message}"),
        }
    }
}
impl Error for EncodeError {}

fn levels(image: &RgbaImage, generate: bool) -> Vec<RgbaImage> {
    let mut levels = vec![image.clone()];
    if generate {
        while levels.len() < MIPMAP_SLOTS {
            let last = levels.last().expect("base image");
            if last.width() == 1 && last.height() == 1 {
                break;
            }
            levels.push(image::imageops::resize(
                last,
                (last.width() / 2).max(1),
                (last.height() / 2).max(1),
                FilterType::Triangle,
            ));
        }
    }
    levels
}

fn validate(images: &[RgbaImage], options: EncodeOptions) -> Result<(), EncodeError> {
    let first = images.first().ok_or(EncodeError::InvalidMipmaps)?;
    if first.width() == 0 || first.height() == 0 || first.width() > 65535 || first.height() > 65535
    {
        return Err(EncodeError::InvalidDimensions);
    }
    if images.len() > MIPMAP_SLOTS || (!options.mipmaps && images.len() != 1) {
        return Err(EncodeError::InvalidMipmaps);
    }
    for (level, image) in images.iter().enumerate() {
        if image.width() != (first.width() >> level).max(1)
            || image.height() != (first.height() >> level).max(1)
        {
            return Err(EncodeError::InvalidMipmaps);
        }
    }
    if options.mipmaps
        && images
            .last()
            .is_some_and(|image| image.width() != 1 || image.height() != 1)
    {
        return Err(EncodeError::InvalidMipmaps);
    }
    let alpha_valid = match options.format {
        EncodeFormat::Blp1Indexed | EncodeFormat::Blp2Indexed => {
            matches!(options.alpha_bits, 0 | 1 | 4 | 8)
        }
        EncodeFormat::Blp1Jpeg | EncodeFormat::Blp2Jpeg => matches!(options.alpha_bits, 0 | 8),
        EncodeFormat::Blp2Dxt(DxtFormat::Dxt1) => matches!(options.alpha_bits, 0 | 1),
        EncodeFormat::Blp2Dxt(DxtFormat::Dxt3) => options.alpha_bits == 4,
        EncodeFormat::Blp2Dxt(DxtFormat::Dxt5) | EncodeFormat::Blp2Bgra => options.alpha_bits == 8,
    };
    if !alpha_valid {
        return Err(EncodeError::InvalidAlphaDepth);
    }
    if matches!(
        options.format,
        EncodeFormat::Blp1Jpeg | EncodeFormat::Blp2Jpeg
    ) && !(1..=100).contains(&options.jpeg_quality)
    {
        return Err(EncodeError::InvalidJpegQuality);
    }
    Ok(())
}

fn indexed(images: &[RgbaImage], alpha_bits: u8) -> (Box<[u8; PALETTE_BYTES]>, Vec<Vec<u8>>) {
    let mut training = Vec::new();
    for image in images {
        for pixel in image.pixels() {
            training.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
        }
    }
    let quant = NeuQuant::new(10, 256, &training);
    let mut palette = Box::new([0; PALETTE_BYTES]);
    for (i, color) in quant.color_map_rgba().chunks_exact(4).enumerate() {
        palette[i * 4..i * 4 + 3].copy_from_slice(&[color[2], color[1], color[0]]);
    }
    let mipmaps = images
        .iter()
        .map(|image| {
            let count = image.width() as usize * image.height() as usize;
            let mut data = Vec::with_capacity(count + (count * alpha_bits as usize).div_ceil(8));
            for pixel in image.pixels() {
                data.push(quant.index_of(&[pixel[0], pixel[1], pixel[2], 255]) as u8);
            }
            match alpha_bits {
                1 => {
                    for pixels in image.as_raw().chunks(32) {
                        let mut byte = 0;
                        for (i, pixel) in pixels.chunks_exact(4).enumerate() {
                            if pixel[3] >= 128 {
                                byte |= 1 << i;
                            }
                        }
                        data.push(byte);
                    }
                }
                4 => {
                    for pixels in image.as_raw().chunks(8) {
                        let mut byte = 0;
                        for (i, pixel) in pixels.chunks_exact(4).enumerate() {
                            byte |= (pixel[3] / 17) << (i * 4);
                        }
                        data.push(byte);
                    }
                }
                8 => {
                    for pixel in image.pixels() {
                        data.push(pixel[3]);
                    }
                }
                _ => (),
            }
            data
        })
        .collect();
    (palette, mipmaps)
}

fn jpeg(image: &RgbaImage, alpha_bits: u8, quality: u8) -> Result<Vec<u8>, EncodeError> {
    let mut bgra = Vec::with_capacity(image.as_raw().len());
    for pixel in image.pixels() {
        // The CMYK JPEG path stores inverted samples. Invert input so the
        // decoder's raw four components retain B, G, R, A values.
        bgra.extend_from_slice(&[
            !pixel[2],
            !pixel[1],
            !pixel[0],
            if alpha_bits == 0 { 0 } else { !pixel[3] },
        ]);
    }
    let mut bytes = Vec::new();
    Encoder::new(&mut bytes, quality)
        .encode(
            &bgra,
            image.width() as u16,
            image.height() as u16,
            ColorType::Cmyk,
        )
        .map_err(|error| EncodeError::Jpeg(error.to_string()))?;
    Ok(bytes)
}

fn dxt(image: &RgbaImage, format: DxtFormat, alpha_bits: u8) -> Vec<u8> {
    let format = match format {
        DxtFormat::Dxt1 => Format::Bc1,
        DxtFormat::Dxt3 => Format::Bc2,
        DxtFormat::Dxt5 => Format::Bc3,
    };
    let mut pixels = image.as_raw().clone();
    if alpha_bits == 0 {
        for pixel in pixels.chunks_exact_mut(4) {
            pixel[3] = 255;
        }
    }
    let (width, height) = (image.width() as usize, image.height() as usize);
    let mut bytes = vec![0; format.compressed_size(width, height)];
    format.compress(&pixels, width, height, Params::default(), &mut bytes);
    bytes
}

fn bgra(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(image.as_raw().len());
    for pixel in image.pixels() {
        bytes.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
    bytes
}

impl Blp {
    /// Encodes an RGBA image, optionally generating all mipmaps through 1×1.
    pub fn encode_image(image: &RgbaImage, options: EncodeOptions) -> Result<Self, EncodeError> {
        let images = levels(image, options.mipmaps);
        Self::encode_mipmaps(&images, options)
    }

    /// Encodes a full caller-supplied mipmap sequence, with level zero first.
    pub fn encode_mipmaps(
        images: &[RgbaImage],
        options: EncodeOptions,
    ) -> Result<Self, EncodeError> {
        validate(images, options)?;
        let first = &images[0];
        let mut mipmaps: [Option<Vec<u8>>; MIPMAP_SLOTS] = std::array::from_fn(|_| None);
        let indexed = if matches!(
            options.format,
            EncodeFormat::Blp1Indexed | EncodeFormat::Blp2Indexed
        ) {
            Some(indexed(images, options.alpha_bits))
        } else {
            None
        };
        for (level, image) in images.iter().enumerate() {
            mipmaps[level] = Some(match options.format {
                EncodeFormat::Blp1Indexed | EncodeFormat::Blp2Indexed => {
                    indexed.as_ref().expect("indexed palette").1[level].clone()
                }
                EncodeFormat::Blp1Jpeg | EncodeFormat::Blp2Jpeg => {
                    jpeg(image, options.alpha_bits, options.jpeg_quality)?
                }
                EncodeFormat::Blp2Dxt(format) => dxt(image, format, options.alpha_bits),
                EncodeFormat::Blp2Bgra => bgra(image),
            });
        }
        let width = first.width();
        let height = first.height();
        let has_mipmaps = images.len() > 1;
        Ok(match options.format {
            EncodeFormat::Blp1Jpeg | EncodeFormat::Blp1Indexed => Self::Blp1(Blp1 {
                header: Blp1Header {
                    width,
                    height,
                    alpha_bits: options.alpha_bits.into(),
                    extra: 5,
                    has_mipmaps: u32::from(has_mipmaps),
                },
                content: match indexed {
                    Some((palette, _)) => Blp1Content::Indexed { palette },
                    None => Blp1Content::Jpeg {
                        shared_header: Vec::new(),
                    },
                },
                mipmaps,
            }),
            _ => Self::Blp2(Blp2 {
                header: Blp2Header {
                    width,
                    height,
                    alpha_bits: options.alpha_bits,
                    mipmap_flags: u8::from(has_mipmaps),
                },
                content: match options.format {
                    EncodeFormat::Blp2Indexed => Blp2Content::Indexed {
                        alpha_type: options.alpha_bits,
                        palette: indexed.expect("indexed palette").0,
                    },
                    EncodeFormat::Blp2Jpeg => Blp2Content::Jpeg {
                        alpha_type: 0,
                        shared_header: Vec::new(),
                        unused: vec![0; PALETTE_BYTES - 4],
                    },
                    EncodeFormat::Blp2Dxt(format) => Blp2Content::Dxt {
                        format,
                        palette_region: Box::new([0; PALETTE_BYTES]),
                    },
                    EncodeFormat::Blp2Bgra => Blp2Content::Bgra {
                        encoding: 3,
                        alpha_type: 0,
                        palette_region: Box::new([0; PALETTE_BYTES]),
                    },
                    _ => unreachable!(),
                },
                mipmaps,
            }),
        })
    }
}
