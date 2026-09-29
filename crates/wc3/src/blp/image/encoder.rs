//! Adapter for image-rs's encoder interface.
use crate::blp::{Blp, EncodeError, EncodeOptions};
use image::error::{EncodingError, ImageFormatHint, UnsupportedError, UnsupportedErrorKind};
use image::{ExtendedColorType, ImageEncoder, ImageError, ImageResult, RgbaImage};
use std::io::Write;

/// Encodes RGBA pixels as a BLP file through the `image` crate's encoder API.
pub struct BlpEncoder<W> {
    writer: W,
    options: EncodeOptions,
}

impl<W: Write> BlpEncoder<W> {
    /// Creates an encoder with the default BLP encoding options.
    pub fn new(writer: W) -> Self {
        Self::with_options(writer, EncodeOptions::default())
    }

    /// Creates an encoder with explicit BLP encoding options.
    pub fn with_options(writer: W, options: EncodeOptions) -> Self {
        Self { writer, options }
    }
}

impl<W: Write> ImageEncoder for BlpEncoder<W> {
    fn write_image(
        mut self,
        buf: &[u8],
        width: u32,
        height: u32,
        color_type: ExtendedColorType,
    ) -> ImageResult<()> {
        if color_type != ExtendedColorType::Rgba8 {
            return Err(ImageError::Unsupported(
                UnsupportedError::from_format_and_kind(
                    ImageFormatHint::Name("BLP".into()),
                    UnsupportedErrorKind::Color(color_type),
                ),
            ));
        }
        let image = RgbaImage::from_raw(width, height, buf.to_vec()).ok_or_else(|| {
            ImageError::Encoding(EncodingError::new(
                ImageFormatHint::Name("BLP".into()),
                EncodeError::InvalidDimensions,
            ))
        })?;
        let blp = Blp::encode_image(&image, self.options).map_err(|error| {
            ImageError::Encoding(EncodingError::new(
                ImageFormatHint::Name("BLP".into()),
                error,
            ))
        })?;
        let bytes = blp.write().map_err(|error| {
            ImageError::Encoding(EncodingError::new(
                ImageFormatHint::Name("BLP".into()),
                error,
            ))
        })?;
        self.writer.write_all(&bytes).map_err(ImageError::IoError)
    }
}
