use super::{image, output, DecodeError};
use image::RgbaImage;

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
