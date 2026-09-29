#[cfg(feature = "blp-decode")]
use super::{image, output, DecodeError};
use image::RgbaImage;

#[cfg(feature = "blp-decode")]
pub(super) fn decode(
    data: &[u8],
    palette: &[u8; 1024],
    alpha_bits: u32,
    width: u32,
    height: u32,
) -> Result<RgbaImage, DecodeError> {
    if !matches!(alpha_bits, 0 | 1 | 4 | 8) {
        return Err(DecodeError::UnsupportedAlphaDepth { depth: alpha_bits });
    }
    let mut pixels = output(width, height)?;
    let count = pixels.len() / 4;
    let alpha_len = count
        .checked_mul(alpha_bits as usize)
        .and_then(|bits| bits.checked_add(7))
        .ok_or(DecodeError::LimitExceeded)?
        / 8;
    if data.len() != count + alpha_len {
        return Err(DecodeError::InvalidData {
            field: "indexed mipmap size",
        });
    }
    let (indices, alpha) = data.split_at(count);
    for (pixel, &index) in pixels.chunks_exact_mut(4).zip(indices) {
        let entry = &palette[usize::from(index) * 4..usize::from(index) * 4 + 4];
        pixel[..3].copy_from_slice(&[entry[2], entry[1], entry[0]]);
    }
    for (i, pixel) in pixels.chunks_exact_mut(4).enumerate() {
        pixel[3] = match alpha_bits {
            0 => 255,
            1 => {
                if alpha[i / 8] & (1 << (i % 8)) == 0 {
                    0
                } else {
                    255
                }
            }
            4 => {
                let nibble = (alpha[i / 2] >> ((i % 2) * 4)) & 0x0f;
                nibble * 17
            }
            8 => alpha[i],
            _ => unreachable!(),
        };
    }
    image(width, height, pixels)
}

#[cfg(feature = "blp-encode")]
pub(super) fn encode(
    images: &[RgbaImage],
    alpha_bits: u8,
) -> (Box<[u8; PALETTE_BYTES]>, Vec<Vec<u8>>) {
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

#[cfg(feature = "blp-encode")]
#[cfg(feature = "blp-encode")]
use crate::blp::PALETTE_BYTES;
#[cfg(feature = "blp-encode")]
use color_quant::NeuQuant;
