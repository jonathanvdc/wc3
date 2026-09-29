#[cfg(feature = "blp-decode")]
use super::{image, output, DecodeError};
use crate::blp::DxtFormat;
use image::RgbaImage;

#[cfg(feature = "blp-decode")]
fn rgb565(value: u16) -> [u8; 3] {
    let r = ((value >> 11) & 31) as u8;
    let g = ((value >> 5) & 63) as u8;
    let b = (value & 31) as u8;
    [
        (r << 3) | (r >> 2),
        (g << 2) | (g >> 4),
        (b << 3) | (b >> 2),
    ]
}

#[cfg(feature = "blp-decode")]
fn colors(bytes: &[u8], four_color: bool, alpha_enabled: bool) -> [[u8; 4]; 4] {
    let c0 = u16::from_le_bytes([bytes[0], bytes[1]]);
    let c1 = u16::from_le_bytes([bytes[2], bytes[3]]);
    let a = rgb565(c0);
    let b = rgb565(c1);
    let mut colors = [[0u8; 4]; 4];
    colors[0] = [a[0], a[1], a[2], 255];
    colors[1] = [b[0], b[1], b[2], 255];
    if four_color || c0 > c1 {
        for channel in 0..3 {
            colors[2][channel] = ((2 * u16::from(a[channel]) + u16::from(b[channel])) / 3) as u8;
            colors[3][channel] = ((u16::from(a[channel]) + 2 * u16::from(b[channel])) / 3) as u8;
        }
        colors[2][3] = 255;
        colors[3][3] = 255;
    } else {
        for channel in 0..3 {
            colors[2][channel] = ((u16::from(a[channel]) + u16::from(b[channel])) / 2) as u8;
        }
        colors[2][3] = 255;
        colors[3][3] = if alpha_enabled { 0 } else { 255 };
    }
    colors
}

#[cfg(feature = "blp-decode")]
fn alpha_bc3(bytes: &[u8], pixel: usize) -> u8 {
    let a0 = bytes[0];
    let a1 = bytes[1];
    let mut table = [0u8; 8];
    table[0] = a0;
    table[1] = a1;
    if a0 > a1 {
        for (i, alpha) in table.iter_mut().enumerate().skip(2) {
            *alpha = (((8 - i) * usize::from(a0) + (i - 1) * usize::from(a1)) / 7) as u8;
        }
    } else {
        for (i, alpha) in table.iter_mut().enumerate().take(6).skip(2) {
            *alpha = (((6 - i) * usize::from(a0) + (i - 1) * usize::from(a1)) / 5) as u8;
        }
        table[6] = 0;
        table[7] = 255;
    }
    let bits = u64::from_le_bytes([
        bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], 0, 0,
    ]);
    table[((bits >> (3 * pixel)) & 7) as usize]
}

#[cfg(feature = "blp-decode")]
pub(super) fn decode(
    data: &[u8],
    format: DxtFormat,
    alpha_enabled: bool,
    width: u32,
    height: u32,
) -> Result<RgbaImage, DecodeError> {
    let mut pixels = output(width, height)?;
    let blocks_wide = usize::try_from(width.div_ceil(4)).map_err(|_| DecodeError::LimitExceeded)?;
    let blocks_high =
        usize::try_from(height.div_ceil(4)).map_err(|_| DecodeError::LimitExceeded)?;
    let block_size = if format == DxtFormat::Dxt1 { 8 } else { 16 };
    let expected = blocks_wide
        .checked_mul(blocks_high)
        .and_then(|blocks| blocks.checked_mul(block_size))
        .ok_or(DecodeError::LimitExceeded)?;
    let data = &data[..data.len().min(expected)];
    let width = width as usize;
    let height = height as usize;
    for block_y in 0..blocks_high {
        for block_x in 0..blocks_wide {
            let start = (block_y * blocks_wide + block_x) * block_size;
            let mut block = [0u8; 16];
            if let Some(source) = data.get(start..) {
                let len = source.len().min(block_size);
                block[..len].copy_from_slice(&source[..len]);
            }
            let color_start = if format == DxtFormat::Dxt1 { 0 } else { 8 };
            let colors = colors(
                &block[color_start..color_start + 4],
                format != DxtFormat::Dxt1,
                alpha_enabled,
            );
            let indices = u32::from_le_bytes(
                block[color_start + 4..color_start + 8]
                    .try_into()
                    .expect("four bytes"),
            );
            for local_y in 0..4 {
                for local_x in 0..4 {
                    let x = block_x * 4 + local_x;
                    let y = block_y * 4 + local_y;
                    if x >= width || y >= height {
                        continue;
                    }
                    let index = local_y * 4 + local_x;
                    let color = colors[((indices >> (2 * index)) & 3) as usize];
                    let dst = (y * width + x) * 4;
                    pixels[dst..dst + 4].copy_from_slice(&color);
                    if format == DxtFormat::Dxt3 {
                        pixels[dst + 3] = ((block[index / 2] >> ((index % 2) * 4)) & 15) * 17;
                    } else if format == DxtFormat::Dxt5 {
                        pixels[dst + 3] = alpha_bc3(&block[..8], index);
                    }
                }
            }
        }
    }
    image(width as u32, height as u32, pixels)
}

#[cfg(feature = "blp-encode")]
pub(super) fn encode(image: &RgbaImage, format: DxtFormat, alpha_bits: u8) -> Vec<u8> {
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

#[cfg(feature = "blp-encode")]
use squish::{Format, Params};
