#[cfg(feature = "blp-decode")]
use super::{image, output, DecodeError};
use image::RgbaImage;

#[cfg(feature = "blp-decode")]
pub(super) fn decode(data: &[u8], width: u32, height: u32) -> Result<RgbaImage, DecodeError> {
    let mut pixels = output(width, height)?;
    if data.len() != pixels.len() {
        return Err(DecodeError::InvalidData {
            field: "BGRA mipmap size",
        });
    }
    for (source, destination) in data.chunks_exact(4).zip(pixels.chunks_exact_mut(4)) {
        destination.copy_from_slice(&[source[2], source[1], source[0], source[3]]);
    }
    image(width, height, pixels)
}

#[cfg(feature = "blp-encode")]
pub(super) fn encode(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(image.as_raw().len());
    for pixel in image.pixels() {
        bytes.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
    bytes
}
