#[cfg(feature = "blp-decode")]
use super::DecodeError;
#[cfg(feature = "blp-encode")]
use image::RgbaImage;

#[cfg(feature = "blp-decode")]
pub(super) fn decode_into(data: &[u8], pixels: &mut [u8]) -> Result<(), DecodeError> {
    if data.len() != pixels.len() {
        return Err(DecodeError::InvalidData {
            field: "BGRA mipmap size",
        });
    }
    for (source, destination) in data.chunks_exact(4).zip(pixels.chunks_exact_mut(4)) {
        destination.copy_from_slice(&[source[2], source[1], source[0], source[3]]);
    }
    Ok(())
}

#[cfg(feature = "blp-encode")]
pub(super) fn encode(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(image.as_raw().len());
    for pixel in image.pixels() {
        bytes.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
    bytes
}
