//! Animated texture translation, rotation, and scaling.
use crate::model::KnownChunk;
use crate::model::Model;
use crate::model::ModelVersion;
use crate::model::TextureAnimationsChunk;
use crate::model::{mdl, mdx};
use crate::model::{Quaternion, Track, Vec3};

/// A texture animation containing translation, rotation, and scaling tracks.
/// MDL uses a `TVertexAnim` block inside `TextureAnims`. Its transform channels
/// are animation tracks only; there are no `static` transform properties.
#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = TextureAnimationsChunk::TAG))]
#[mdl(block = "TVertexAnim")]
pub struct TextureAnimation {
    #[mdx(tag = *b"KTAT")]
    #[mdl(property = "Translation")]
    /// Optional texture-coordinate translation track.
    pub translation: Option<Track<Vec3>>,
    #[mdx(tag = *b"KTAR")]
    #[mdl(property = "Rotation")]
    /// Optional texture-coordinate quaternion rotation track.
    pub rotation: Option<Track<Quaternion>>,
    #[mdx(tag = *b"KTAS")]
    #[mdl(property = "Scaling")]
    /// Optional texture-coordinate scale track.
    pub scaling: Option<Track<Vec3>>,
}

impl TextureAnimation {
    /// Creates an empty texture animation.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of texture animations in `TXAN` chunks.
    pub fn texture_animations(&self) -> Vec<TextureAnimation> {
        self.collect_chunk_records::<TextureAnimationsChunk>()
    }

    /// Replaces texture animations with one `TXAN` chunk, removing any duplicate chunks.
    pub fn set_texture_animations(&mut self, animations: &[TextureAnimation]) {
        self.replace_chunk(TextureAnimationsChunk::new(animations.to_vec()));
    }
}
