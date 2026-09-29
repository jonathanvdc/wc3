use super::{image, output, DecodeError, MAX_DECODE_BYTES};
use image::RgbaImage;
use zune_jpeg::{
    zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions},
    JpegDecoder,
};

pub(super) fn decode(
    mip: &[u8],
    shared_header: &[u8],
    alpha_opaque: bool,
    width: u32,
    height: u32,
) -> Result<RgbaImage, DecodeError> {
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
    let mut pixels = output(width, height)?;
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
    image(width, height, pixels)
}
