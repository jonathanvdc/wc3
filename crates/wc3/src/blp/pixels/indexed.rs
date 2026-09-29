use super::{image, output, DecodeError};
use image::RgbaImage;

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
