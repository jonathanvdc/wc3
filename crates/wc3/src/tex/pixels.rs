#[cfg(any(feature = "tex-decode", feature = "tex-encode"))]
use super::Error;
#[cfg(any(feature = "tex-decode", feature = "tex-encode"))]
use image::RgbaImage;
#[cfg(any(feature = "tex-decode", feature = "tex-encode"))]
use squish::Format;
#[cfg(feature = "tex-encode")]
use squish::Params;
#[cfg(feature = "tex-decode")]
use texture2ddecoder::{decode_bc2, decode_bc4, decode_bc5, decode_bc7};

#[cfg(any(feature = "tex-decode", feature = "tex-encode"))]
fn bc_format(format: u32) -> Option<Format> {
    match format {
        9 | 10 => Some(Format::Bc1),
        11 => Some(Format::Bc2),
        12 => Some(Format::Bc3),
        _ => None,
    }
}
/// Converts the byte planes specified by the D3 TEX format.
#[cfg(any(feature = "tex-decode", feature = "tex-encode"))]
fn shuffle(format: u32, data: &[u8], encode: bool) -> Result<Vec<u8>, Error> {
    let lanes: &[usize] = match format {
        9 | 10 => &[4, 4],
        11 => &[8, 4, 4],
        12 | 43 => &[2, 6, 4, 4],
        _ => return Ok(data.to_vec()),
    };
    let stride: usize = lanes.iter().sum();
    if !data.len().is_multiple_of(stride) {
        return Err(Error::Invalid(
            "BC byte count is not a multiple of the block size",
        ));
    }
    let blocks = data.len() / stride;
    let mut result = vec![0; data.len()];
    let mut plane_start = 0;
    let mut lane_start = 0;
    for &width in lanes {
        for block in 0..blocks {
            let planar = plane_start + block * width;
            let interleaved = block * stride + lane_start;
            let (source, target) = if encode {
                (interleaved, planar)
            } else {
                (planar, interleaved)
            };
            result[target..target + width].copy_from_slice(&data[source..source + width]);
        }
        plane_start += blocks * width;
        lane_start += width;
    }
    Ok(result)
}

#[cfg(feature = "tex-decode")]
fn decode(format: u32, data: &[u8], width: u32, height: u32) -> Result<RgbaImage, Error> {
    let count = (width as usize)
        .checked_mul(height as usize)
        .ok_or(Error::LimitExceeded)?;
    let size = count.checked_mul(4).ok_or(Error::LimitExceeded)?;
    // Constrain allocations independently from file-controlled dimensions.
    if size > 512 * 1024 * 1024 {
        return Err(Error::LimitExceeded);
    }
    let mut pixels = vec![0; size];
    if let Some(bc) = bc_format(format) {
        if data.len() != bc.compressed_size(width as usize, height as usize) {
            return Err(Error::Invalid("compressed mip size"));
        }
        if bc == Format::Bc2 {
            let mut decoded = vec![0; count];
            decode_bc2(data, width as usize, height as usize, &mut decoded)
                .map_err(Error::Invalid)?;
            for (value, pixel) in decoded
                .into_iter()
                .zip(pixels.as_chunks_mut::<4>().0.iter_mut())
            {
                let [b, g, r, a] = value.to_le_bytes();
                pixel.copy_from_slice(&[r, g, b, a]);
            }
        } else {
            bc.decompress(data, width as usize, height as usize, &mut pixels);
        }
    } else if matches!(format, 41 | 42 | 44) {
        let block_size = if format == 41 { 8 } else { 16 };
        let expected = (width as usize)
            .div_ceil(4)
            .checked_mul((height as usize).div_ceil(4))
            .and_then(|n| n.checked_mul(block_size))
            .ok_or(Error::LimitExceeded)?;
        if data.len() != expected {
            return Err(Error::Invalid("compressed mip size"));
        }
        let mut decoded = vec![0; count];
        let decoder = match format {
            41 => decode_bc4,
            42 => decode_bc5,
            _ => decode_bc7,
        };
        decoder(data, width as usize, height as usize, &mut decoded).map_err(Error::Invalid)?;
        for (value, pixel) in decoded
            .into_iter()
            .zip(pixels.as_chunks_mut::<4>().0.iter_mut())
        {
            let [b, g, r, a] = value.to_le_bytes();
            pixel.copy_from_slice(&[r, g, b, if format == 44 { a } else { 255 }]);
        }
    } else {
        let stride = match format {
            0 | 1 => 4,
            4 => 2,
            7 | 23 => 1,
            _ => return Err(Error::UnsupportedPixelFormat(format)),
        };
        if data.len() != count.checked_mul(stride).ok_or(Error::LimitExceeded)? {
            return Err(Error::Invalid("pixel mip size"));
        }
        for (src, dst) in data
            .chunks_exact(stride)
            .zip(pixels.as_chunks_mut::<4>().0.iter_mut())
        {
            let rgba = match format {
                0 => [src[2], src[1], src[0], src[3]],
                1 => [src[0], src[1], src[2], src[3]],
                4 => {
                    let p = u16::from_le_bytes([src[0], src[1]]);
                    [
                        ((p >> 8) & 15) as u8 * 17,
                        ((p >> 4) & 15) as u8 * 17,
                        (p & 15) as u8 * 17,
                        (p >> 12) as u8 * 17,
                    ]
                }
                7 => [src[0], src[0], src[0], 255],
                23 => [0, 0, 0, src[0]],
                _ => unreachable!(),
            };
            dst.copy_from_slice(&rgba);
        }
    }
    RgbaImage::from_raw(width, height, pixels).ok_or(Error::LimitExceeded)
}
#[cfg(feature = "tex-decode")]
pub(super) fn decode_d3(
    format: u32,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<RgbaImage, Error> {
    let normalized;
    let data = if bc_format(format).is_some() || format == 43 {
        normalized = shuffle(format, data, false)?;
        normalized.as_slice()
    } else {
        data
    };
    decode(if format == 43 { 42 } else { format }, data, width, height)
}
#[cfg(feature = "tex-decode")]
pub(super) fn decode_d4(
    format: u32,
    data: &[u8],
    width: u32,
    height: u32,
) -> Result<RgbaImage, Error> {
    let mapped = match format {
        0 | 13 | 45 | 59 => 0,
        1 | 14 | 21 | 53 | 61 | 67 => 1,
        7 | 20 => 7,
        23 | 64 => 23,
        9 | 10 | 46 | 47 => 9,
        11 | 48 => 11,
        12 | 49 => 12,
        41 => 41,
        42 => 42,
        44 | 50 => 44,
        _ => return Err(Error::UnsupportedPixelFormat(format)),
    };
    let bc = bc_format(mapped);
    let compressed = bc.is_some() || matches!(mapped, 41 | 42 | 44);
    let row = if let Some(bc) = bc {
        (width as usize).div_ceil(4) * if bc == Format::Bc1 { 8 } else { 16 }
    } else if matches!(mapped, 41 | 42 | 44) {
        (width as usize).div_ceil(4) * if mapped == 41 { 8 } else { 16 }
    } else {
        (width as usize) * if matches!(mapped, 0 | 1) { 4 } else { 1 }
    };
    let rows = if compressed {
        (height as usize).div_ceil(4)
    } else {
        height as usize
    };
    let pitch = row.checked_add(255).ok_or(Error::LimitExceeded)? / 256 * 256;
    let size = row.checked_mul(rows).ok_or(Error::LimitExceeded)?;
    if size > 512 * 1024 * 1024 {
        return Err(Error::LimitExceeded);
    }
    let mut packed = Vec::with_capacity(size);
    for y in 0..rows {
        packed.extend_from_slice(super::range(
            data,
            y.checked_mul(pitch).ok_or(Error::LimitExceeded)?,
            row,
        )?);
    }
    decode(mapped, &packed, width, height)
}
#[cfg(feature = "tex-encode")]
pub(super) fn encode_d3(format: u32, image: &RgbaImage) -> Result<Vec<u8>, Error> {
    if image.width() == 0 || image.height() == 0 {
        return Err(Error::Invalid("zero texture dimensions"));
    }
    if let Some(bc) = bc_format(format) {
        let mut blocks =
            vec![0; bc.compressed_size(image.width() as usize, image.height() as usize)];
        bc.compress(
            image.as_raw(),
            image.width() as usize,
            image.height() as usize,
            Params::default(),
            &mut blocks,
        );
        let mut out = vec![0; 16];
        out.extend(shuffle(format, &blocks, true)?);
        return Ok(out);
    }
    let mut out = Vec::new();
    for p in image.pixels() {
        match format {
            0 => out.extend_from_slice(&[p[2], p[1], p[0], p[3]]),
            4 => {
                let v = (u16::from(p[3] >> 4) << 12)
                    | (u16::from(p[0] >> 4) << 8)
                    | (u16::from(p[1] >> 4) << 4)
                    | u16::from(p[2] >> 4);
                out.extend_from_slice(&v.to_le_bytes());
            }
            7 => out.push(p[0]),
            23 => out.push(p[3]),
            _ => return Err(Error::UnsupportedPixelFormat(format)),
        }
    }
    Ok(out)
}
#[cfg(feature = "tex-encode")]
pub(super) fn encode_d4(format: u32, image: &RgbaImage) -> Result<Vec<u8>, Error> {
    if image.width() == 0 || image.height() == 0 {
        return Err(Error::Invalid("zero texture dimensions"));
    }
    let mapped = match format {
        0 | 13 | 45 | 59 => 0,
        1 | 14 | 21 | 53 | 61 | 67 => 1,
        7 | 20 => 7,
        23 | 64 => 23,
        9 | 10 | 46 | 47 => 9,
        11 | 48 => 11,
        12 | 49 => 12,
        41 => 41,
        42 => 42,
        44 | 50 => 44,
        _ => return Err(Error::UnsupportedPixelFormat(format)),
    };
    let bc = bc_format(mapped);
    let packed = if let Some(bc) = bc {
        let mut data = vec![0; bc.compressed_size(image.width() as usize, image.height() as usize)];
        bc.compress(
            image.as_raw(),
            image.width() as usize,
            image.height() as usize,
            Params::default(),
            &mut data,
        );
        data
    } else if mapped == 1 {
        image.as_raw().clone()
    } else {
        encode_d3(mapped, image)?
    };
    let row = if let Some(bc) = bc {
        (image.width() as usize).div_ceil(4) * if bc == Format::Bc1 { 8 } else { 16 }
    } else {
        image.width() as usize * if matches!(mapped, 0 | 1) { 4 } else { 1 }
    };
    if row == 0 {
        return Err(Error::Invalid("zero texture dimensions"));
    }
    let pitch = row.checked_add(255).ok_or(Error::LimitExceeded)? / 256 * 256;
    let rows = packed.len() / row;
    let mut out = vec![0; pitch.checked_mul(rows).ok_or(Error::LimitExceeded)?];
    for (src, dst) in packed.chunks_exact(row).zip(out.chunks_exact_mut(pitch)) {
        dst[..row].copy_from_slice(src);
    }
    Ok(out)
}

#[cfg(all(test, any(feature = "tex-decode", feature = "tex-encode")))]
#[path = "pixels_tests.rs"]
mod tests;
