#[cfg(feature = "blp-decode")]
use super::DecodeError;
#[cfg(feature = "blp-encode")]
use image::RgbaImage;

#[cfg(feature = "blp-decode")]
pub(super) fn decode_into(data: &[u8], pixels: &mut [u8]) -> Result<(), DecodeError> {
    let (source_pixels, source_remainder) = data.as_chunks::<4>();
    let (destination_pixels, destination_remainder) = pixels.as_chunks_mut::<4>();
    if !source_remainder.is_empty()
        || !destination_remainder.is_empty()
        || source_pixels.len() != destination_pixels.len()
    {
        return Err(DecodeError::InvalidData {
            field: "BGRA mipmap size",
        });
    }
    for (source, destination) in source_pixels.iter().zip(destination_pixels) {
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
