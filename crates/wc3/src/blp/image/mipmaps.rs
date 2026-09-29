//! Mipmap generation and dimension validation.
use super::EncodeError;
use crate::blp::MIPMAP_SLOTS;
use image::{imageops::FilterType, RgbaImage};

pub(super) fn generate(image: &RgbaImage, enabled: bool) -> Vec<RgbaImage> {
    let mut levels = vec![image.clone()];
    if enabled {
        while levels.len() < MIPMAP_SLOTS {
            let last = levels.last().expect("base image");
            if last.width() == 1 && last.height() == 1 {
                break;
            }
            levels.push(image::imageops::resize(
                last,
                (last.width() / 2).max(1),
                (last.height() / 2).max(1),
                FilterType::Triangle,
            ));
        }
    }
    levels
}

pub(super) fn validate(images: &[RgbaImage], enabled: bool) -> Result<(), EncodeError> {
    let first = images.first().ok_or(EncodeError::InvalidMipmaps)?;
    if first.width() == 0 || first.height() == 0 || first.width() > 65535 || first.height() > 65535
    {
        return Err(EncodeError::InvalidDimensions);
    }
    if images.len() > MIPMAP_SLOTS || (!enabled && images.len() != 1) {
        return Err(EncodeError::InvalidMipmaps);
    }
    for (level, image) in images.iter().enumerate() {
        if image.width() != (first.width() >> level).max(1)
            || image.height() != (first.height() >> level).max(1)
        {
            return Err(EncodeError::InvalidMipmaps);
        }
    }
    if enabled
        && images
            .last()
            .is_some_and(|image| image.width() != 1 || image.height() != 1)
    {
        return Err(EncodeError::InvalidMipmaps);
    }
    Ok(())
}
