//! Adapter for image-rs's decoder interface.
use crate::blp::{BlpRef, DecodeError, ReadError, MIPMAP_SLOTS};
use image::error::{DecodingError, ImageFormatHint};
use image::{ColorType, ImageDecoder, ImageError, ImageResult};

/// Decodes one BLP mipmap through the `image` crate's general decoder API.
pub struct BlpDecoder<'a> {
    blp: BlpRef<'a>,
    level: usize,
    dimensions: (u32, u32),
}

fn image_error(error: impl std::error::Error + Send + Sync + 'static) -> ImageError {
    ImageError::Decoding(DecodingError::new(
        ImageFormatHint::Name("BLP".into()),
        error,
    ))
}

impl<'a> BlpDecoder<'a> {
    /// Reads a BLP file and selects its largest mipmap.
    pub fn new(bytes: &'a [u8]) -> ImageResult<Self> {
        Self::with_mip(bytes, 0)
    }

    /// Reads a BLP file and selects a mipmap by level.
    pub fn with_mip(bytes: &'a [u8], level: usize) -> ImageResult<Self> {
        let blp = BlpRef::read(bytes).map_err(|error: ReadError| image_error(error))?;
        let (width, height, present) = match &blp {
            BlpRef::Blp1(value) => (value.header.width, value.header.height, &value.mipmaps),
            BlpRef::Blp2(value) => (value.header.width, value.header.height, &value.mipmaps),
        };
        if width == 0 || height == 0 || level >= MIPMAP_SLOTS {
            return Err(image_error(DecodeError::InvalidDimensions));
        }
        if present[level].is_none() {
            return Err(image_error(DecodeError::MissingMipmap { level }));
        }
        let dimensions = ((width >> level).max(1), (height >> level).max(1));
        let bytes = u64::from(dimensions.0) * u64::from(dimensions.1) * 4;
        if bytes > super::decode::MAX_DECODE_BYTES as u64 {
            return Err(image_error(DecodeError::LimitExceeded));
        }
        Ok(Self {
            blp,
            level,
            dimensions,
        })
    }
}

impl ImageDecoder for BlpDecoder<'_> {
    fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    fn color_type(&self) -> ColorType {
        ColorType::Rgba8
    }

    fn read_image(self, buf: &mut [u8]) -> ImageResult<()> {
        assert_eq!(buf.len() as u64, self.total_bytes());
        self.blp
            .decode_mip_into(self.level, buf)
            .map_err(image_error)
    }

    fn read_image_boxed(self: Box<Self>, buf: &mut [u8]) -> ImageResult<()> {
        (*self).read_image(buf)
    }
}
