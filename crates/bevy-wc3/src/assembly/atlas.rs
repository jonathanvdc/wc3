use bevy::asset::RenderAssetUsages;
use bevy::prelude::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::Arc;

/// Decoded, tightly packed, top-to-bottom straight-alpha sRGB pixels.
/// Clones share immutable pixel storage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaImage {
    width: u32,
    height: u32,
    pixels: Arc<Vec<u8>>,
}
impl RgbaImage {
    /// Checks dimensions and storage before accepting decoded pixels.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, AtlasError> {
        let len = pixel_len(width, height)?;
        if pixels.len() != len {
            return Err(AtlasError::PixelCount);
        }
        Ok(Self {
            width,
            height,
            pixels: Arc::new(pixels),
        })
    }
    /// Pixel width.
    pub fn width(&self) -> u32 {
        self.width
    }
    /// Pixel height.
    pub fn height(&self) -> u32 {
        self.height
    }
    /// Tightly packed RGBA bytes.
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    /// Creates a Bevy color image with sRGB sampling and CPU/GPU asset usage.
    pub fn into_image(self) -> Image {
        Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            self.into_pixels(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        )
    }
    /// Takes the tightly packed RGBA bytes, reusing uniquely owned storage.
    /// When another image shares the pixels, this returns an independent copy.
    pub fn into_pixels(self) -> Vec<u8> {
        Arc::unwrap_or_clone(self.pixels)
    }
}
/// A checked rectangle uses top-left image coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelRect {
    /// Left edge in pixels.
    pub x: u32,
    /// Top edge in pixels.
    pub y: u32,
    /// Positive pixel width.
    pub width: u32,
    /// Positive pixel height.
    pub height: u32,
}
/// Pixel compositing in the encoded color domain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AtlasBlend {
    /// Normalized straight-alpha source-over, including destination alpha.
    #[default]
    SourceOver,
    /// Copy source RGB and its opacity-adjusted alpha.
    Replace,
    /// Source-alpha RGB interpolation; alpha is Sa² + Da(1-Sa).
    Alpha,
    /// Source-alpha RGB interpolation; alpha is Sa + Da(1-Sa).
    AlphaStraight,
    /// Multiply source and destination RGBA, then apply `Alpha`.
    Multiply,
    /// Source-conditioned overlay RGB, then apply `Alpha` with source alpha.
    Overlay,
    /// Screen RGB, then apply `Alpha` with source alpha.
    Screen,
}
/// One resolved atlas layer. Input order is back-to-front composition order.
pub struct AtlasLayer<'a> {
    /// Decoded source image.
    pub image: &'a RgbaImage,
    /// Selected source image region.
    pub source: PixelRect,
    /// Destination region, explicitly supplied by the client-build database.
    pub destination: PixelRect,
    /// Additional opacity, with 255 meaning unchanged source alpha.
    pub opacity: u8,
    /// Encoded-color compositing equation.
    pub blend: AtlasBlend,
}
/// Output dimensions and opaque/transparent clear color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Atlas {
    /// Positive pixel width.
    pub width: u32,
    /// Positive pixel height.
    pub height: u32,
    /// Initial straight-alpha sRGB pixel.
    pub clear: [u8; 4],
}
/// Invalid decoded input or atlas placement.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtlasError {
    /// Dimensions are zero or exceed addressable byte storage.
    Dimensions,
    /// Pixel byte count differs from width times height times four.
    PixelCount,
    /// A source or destination rectangle is empty or outside its image.
    Rectangle,
}
impl Display for AtlasError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str(match self {
            Self::Dimensions => "invalid atlas image dimensions",
            Self::PixelCount => "RGBA pixel byte count does not match dimensions",
            Self::Rectangle => "atlas rectangle lies outside its image",
        })
    }
}
impl Error for AtlasError {}
pub(super) fn pixel_len(width: u32, height: u32) -> Result<usize, AtlasError> {
    if width == 0 || height == 0 {
        return Err(AtlasError::Dimensions);
    }
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .filter(|n| *n <= isize::MAX as usize)
        .ok_or(AtlasError::Dimensions)
}
pub(super) fn valid(rect: PixelRect, width: u32, height: u32) -> bool {
    rect.width > 0
        && rect.height > 0
        && rect
            .x
            .checked_add(rect.width)
            .is_some_and(|end| end <= width)
        && rect
            .y
            .checked_add(rect.height)
            .is_some_and(|end| end <= height)
}
/// Composes layers with deterministic nearest-neighbor scaling and explicit blending.
///
/// Blending takes place in the decoded sRGB byte domain, as a texture compositor;
/// the resulting color texture receives its usual sRGB decode when rendered.
/// Rectangles are validated before allocation or processing; inputs are unchanged.
pub fn compose_atlas(atlas: Atlas, layers: &[AtlasLayer<'_>]) -> Result<RgbaImage, AtlasError> {
    let len = pixel_len(atlas.width, atlas.height)?;
    for layer in layers {
        if !valid(layer.source, layer.image.width, layer.image.height)
            || !valid(layer.destination, atlas.width, atlas.height)
        {
            return Err(AtlasError::Rectangle);
        }
    }
    let mut pixels = vec![0; len];
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.copy_from_slice(&atlas.clear);
    }
    for layer in layers {
        if layer.opacity == 0 && layer.blend != AtlasBlend::Replace {
            continue;
        }
        for y in 0..layer.destination.height {
            let sy = layer.source.y
                + (u64::from(y) * u64::from(layer.source.height)
                    / u64::from(layer.destination.height)) as u32;
            for x in 0..layer.destination.width {
                let sx = layer.source.x
                    + (u64::from(x) * u64::from(layer.source.width)
                        / u64::from(layer.destination.width)) as u32;
                let si = (sy as usize * layer.image.width as usize + sx as usize) * 4;
                let di = ((y + layer.destination.y) as usize * atlas.width as usize
                    + (x + layer.destination.x) as usize)
                    * 4;
                let source = &layer.image.pixels[si..si + 4];
                let target = &mut pixels[di..di + 4];
                blend_pixel(target, source, layer.opacity, layer.blend);
            }
        }
    }
    RgbaImage::new(atlas.width, atlas.height, pixels)
}

fn blend_pixel(target: &mut [u8], source: &[u8], opacity: u8, blend: AtlasBlend) {
    let mul = |a: u32, b: u32| (a * b + 127) / 255;
    let sa = mul(u32::from(source[3]), u32::from(opacity));
    let da = u32::from(target[3]);
    if blend == AtlasBlend::Replace {
        target[..3].copy_from_slice(&source[..3]);
        target[3] = sa as u8;
        return;
    }
    if blend == AtlasBlend::SourceOver {
        let alpha = sa * 255 + da * (255 - sa);
        if alpha == 0 {
            target.fill(0);
            return;
        }
        for c in 0..3 {
            let color = u32::from(source[c]) * sa * 255 + u32::from(target[c]) * da * (255 - sa);
            target[c] = ((color + alpha / 2) / alpha) as u8;
        }
        target[3] = ((alpha + 127) / 255) as u8;
        return;
    }
    let mut sa = f32::from(source[3]) / 255. * f32::from(opacity) / 255.;
    let da = da as f32 / 255.;
    if blend == AtlasBlend::Multiply {
        sa *= da;
    }
    for c in 0..3 {
        let s = f32::from(source[c]) / 255.;
        let d = f32::from(target[c]) / 255.;
        let color = match blend {
            AtlasBlend::Multiply => s * d,
            AtlasBlend::Overlay if s < 0.5 => 2. * s * d,
            AtlasBlend::Overlay => 1. - 2. * (1. - s) * (1. - d),
            AtlasBlend::Screen => 1. - (1. - s) * (1. - d),
            _ => s,
        };
        target[c] = ((color * sa + d * (1. - sa)) * 255.).round() as u8;
    }
    let alpha = if blend == AtlasBlend::AlphaStraight {
        1.
    } else {
        sa
    };
    target[3] = ((alpha * sa + da * (1. - sa)) * 255.).round() as u8;
}

#[cfg(test)]
#[path = "atlas_tests.rs"]
mod tests;
