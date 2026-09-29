#[cfg(feature = "blp-decode")]
use super::{DecodeError, MAX_DECODE_BYTES};
#[cfg(feature = "blp-encode")]
use image::RgbaImage;
#[cfg(feature = "blp-decode")]
use zune_jpeg::{
    zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions},
    JpegDecoder,
};

#[cfg(feature = "blp-decode")]
pub(super) fn decode_into(
    mip: &[u8],
    shared_header: &[u8],
    alpha_opaque: bool,
    width: u32,
    height: u32,
    pixels: &mut [u8],
) -> Result<(), DecodeError> {
    let len = shared_header
        .len()
        .checked_add(mip.len())
        .ok_or(DecodeError::LimitExceeded)?;
    if len > MAX_DECODE_BYTES {
        return Err(DecodeError::LimitExceeded);
    }
    let mut jpeg = Vec::new();
    jpeg.try_reserve_exact(len)
        .map_err(|_| DecodeError::LimitExceeded)?;
    jpeg.extend_from_slice(shared_header);
    jpeg.extend_from_slice(mip);
    let options = DecoderOptions::default()
        .set_max_width(65535)
        .set_max_height(65535)
        .jpeg_set_out_colorspace(ColorSpace::CMYK);
    let mut decoder = JpegDecoder::new_with_options(ZCursor::new(&jpeg), options);
    decoder
        .decode_headers()
        .map_err(|error| DecodeError::Jpeg(error.to_string()))?;
    if decoder.input_colorspace() != Some(ColorSpace::CMYK)
        || decoder.output_colorspace() != Some(ColorSpace::CMYK)
    {
        return Err(DecodeError::UnsupportedJpegColor);
    }
    if decoder
        .output_buffer_size()
        .ok_or(DecodeError::InvalidDimensions)?
        > MAX_DECODE_BYTES
    {
        return Err(DecodeError::LimitExceeded);
    }
    let (jpeg_width, jpeg_height) = decoder.dimensions().ok_or(DecodeError::InvalidDimensions)?;
    let raw = decoder
        .decode()
        .map_err(|error| DecodeError::Jpeg(error.to_string()))?;
    let source_width = jpeg_width;
    let source_height = jpeg_height;
    if raw.len()
        != source_width
            .checked_mul(source_height)
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(DecodeError::LimitExceeded)?
    {
        return Err(DecodeError::InvalidData {
            field: "JPEG output size",
        });
    }
    pixels.fill(0);
    let width_usize = width as usize;
    for y in 0..height as usize {
        if y >= source_height {
            break;
        }
        for x in 0..width_usize.min(source_width) {
            let src = (y * source_width + x) * 4;
            let dst = (y * width_usize + x) * 4;
            pixels[dst..dst + 4].copy_from_slice(&[
                raw[src + 2],
                raw[src + 1],
                raw[src],
                if alpha_opaque { 255 } else { raw[src + 3] },
            ]);
        }
    }
    Ok(())
}

#[cfg(feature = "blp-encode")]
pub(super) fn encode(
    image: &RgbaImage,
    alpha_bits: u8,
    quality: u8,
) -> Result<Vec<u8>, EncodeError> {
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

#[cfg(feature = "blp-encode")]
use super::EncodeError;
#[cfg(feature = "blp-encode")]
use jpeg_encoder::{ColorType, Encoder};
