//! Animated texture translation, rotation, and scaling.
use crate::model::KnownChunk;
use crate::model::Model;
use crate::model::ModelDialect;
use crate::model::ModelOwnership;
use crate::model::TextureAnimationsChunk;
use crate::model::{mdl, mdx};
use crate::model::{Quaternion, Track, Vec3};

/// A texture animation containing translation, rotation, and scaling tracks.
/// MDL uses a `TVertexAnim` block inside `TextureAnims`. Its transform channels
/// are animation tracks only; there are no `static` transform properties.
#[derive(
    Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write, ModelOwnership,
)]
#[mdx(sized(tag = TextureAnimationsChunk::TAG))]
#[mdl(block = "TVertexAnim")]
pub struct TextureAnimation<'data> {
    #[mdx(tag = *b"KTAT")]
    #[mdl(property = "Translation")]
    /// Optional texture-coordinate translation track.
    pub translation: Option<Track<'data, Vec3>>,
    #[mdx(tag = *b"KTAR")]
    #[mdl(property = "Rotation")]
    /// Optional texture-coordinate quaternion rotation track.
    pub rotation: Option<Track<'data, Quaternion>>,
    #[mdx(tag = *b"KTAS")]
    #[mdl(property = "Scaling")]
    /// Optional texture-coordinate scale track.
    pub scaling: Option<Track<'data, Vec3>>,
}

impl<'data> TextureAnimation<'data> {
    /// Creates an empty texture animation.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn texture_animations<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'s TextureAnimation<'data>> + use<'data, 's, D> {
        self.records::<TextureAnimationsChunk<'_>>()
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn texture_animations_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut TextureAnimation<'data>> + use<'data, 's, D> {
        self.records_mut::<TextureAnimationsChunk<'_>>()
    }

    /// Replaces texture animations with one `TXAN` chunk, removing any duplicate chunks.
    pub fn set_texture_animations(
        &mut self,
        animations: impl IntoIterator<Item = TextureAnimation<'data>>,
    ) {
        self.replace_chunk(TextureAnimationsChunk::new(
            animations.into_iter().collect(),
        ));
    }
}
