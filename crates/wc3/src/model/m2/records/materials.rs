//! Material flags, texture references, and animated material properties.
use super::Track;
use super::{BlendMode, MaterialFlags, TextureFlags, TextureSource};
use crate::model::{common::Ownership, m2};
use m2::ModelLayout;
use zerovec::{ule::AsULE, ZeroVec};

/// Render flags and blending operation shared by draw batches.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct Material {
    /// Render-state flags controlling lighting, fog, culling, depth, and shadow behavior.
    pub flags: MaterialFlags,
    /// M2 blend-mode code used to select the color and alpha blending operation.
    pub blending_mode: BlendMode,
}
/// Texture source and sampler wrapping settings.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct Texture<'data> {
    /// Texture source type: zero uses a file reference; other values select runtime replacements.
    pub kind: TextureSource,
    /// Sampler wrapping flags: `0x01` wraps U and `0x02` wraps V.
    pub flags: TextureFlags,
    /// NUL-terminated texture path for source type zero; TXID file IDs can replace this path.
    pub filename: ZeroVec<'data, u8>,
}
/// Animated RGB and opacity modulation referenced by draw batches.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct ColorAnimation<'data, V: ModelLayout> {
    /// Animated RGB multiplier, normally with components between zero and one.
    pub color: Track<'data, [f32; 3], V>,
    /// Animated opacity: zero is transparent and `0x7fff` is fully opaque.
    pub alpha: Track<'data, i16, V>,
}
/// Animated opacity referenced through the texture-weight lookup.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct TextureWeight<'data, V: ModelLayout> {
    /// Animated texture opacity: zero is transparent and `0x7fff` is fully opaque.
    pub weight: Track<'data, i16, V>,
}
/// Legacy texture-frame animation.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct TextureFlipbook<'data, V: ModelLayout> {
    /// Legacy animated texture-frame selection; frame interpretation depends on the texture.
    pub frames: Track<'data, u16, V>,
}
/// Animated UV translation, rotation, and scale.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Default,
    Ownership,
    m2::Read,
    m2::Write,
    m2::RecordLayout,
    m2::Validate,
    m2::VisitArrays,
)]
pub struct TextureTransform<'data, V: ModelLayout> {
    /// Animated translation of texture coordinates.
    pub translation: Track<'data, [f32; 3], V>,
    /// Animated `[x, y, z, w]` quaternion rotation about the UV center `(0.5, 0.5)`.
    pub rotation: Track<'data, [f32; 4], V>,
    /// Animated scale of texture coordinates.
    pub scaling: Track<'data, [f32; 3], V>,
}
impl AsULE for Material {
    type ULE = <[u8; 4] as AsULE>::ULE;
    fn to_unaligned(self) -> Self::ULE {
        let flags = self.flags.bits().to_le_bytes();
        let blending = self.blending_mode.raw().to_le_bytes();
        [flags[0], flags[1], blending[0], blending[1]].to_unaligned()
    }
    fn from_unaligned(unaligned: Self::ULE) -> Self {
        let bytes = <[u8; 4]>::from_unaligned(unaligned);
        Self {
            flags: MaterialFlags::from_bits_retain(u16::from_le_bytes([bytes[0], bytes[1]])),
            blending_mode: BlendMode::from_raw(u16::from_le_bytes([bytes[2], bytes[3]])),
        }
    }
}
