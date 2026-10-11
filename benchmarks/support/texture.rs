use image::{Rgba, RgbaImage};

pub fn texture(size: u32) -> RgbaImage {
    // Flat areas, colored edges, fine detail, transparent holes, and gradients.
    RgbaImage::from_fn(size, size, |x, y| {
        let detail = ((x.wrapping_mul(73) ^ y.wrapping_mul(151)) & 31) as u8;
        Rgba([
            ((x / 16 * 37) % 256) as u8,
            ((y * 255 / size) as u8).saturating_add(detail),
            ((x ^ y) % 256) as u8,
            if (x / 8 + y / 8) % 5 == 0 {
                0
            } else {
                (128 + (x + y) % 128) as u8
            },
        ])
    })
}
