//! Typed texture animation tracks in `TXAN` chunks.
use crate::model::ModelVersion;
use crate::model::{mdl, mdx};
crate::model::animation::track_group! {
    pub enum TextureAnimationTrack {
        Translation: TextureTranslation,
        Rotation: TextureRotation,
        Scaling: TextureScaling,
    }
}

use crate::model::KnownChunk;

use crate::model::Model;
use crate::model::TextureAnimationsChunk;

/// A texture animation containing translation, rotation, and scaling tracks.
/// MDL uses a `TVertexAnim` block inside `TextureAnims`. Its transform channels
/// are animation tracks only; there are no `static` transform properties.
#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = TextureAnimationsChunk::TAG))]
#[mdl(block = "TVertexAnim")]
pub struct TextureAnimation {
    #[mdl(
        tracks,
        channels(
            Translation = "TextureAnimationTrack::Translation",
            Rotation = "TextureAnimationTrack::Rotation",
            Scaling = "TextureAnimationTrack::Scaling"
        )
    )]
    pub tracks: Vec<TextureAnimationTrack>,
}

impl TextureAnimation {
    /// Creates an empty texture animation.
    pub fn new() -> Self {
        Self::default()
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all texture animations in `TXAN` chunks.
    pub fn texture_animations(&self) -> Vec<TextureAnimation> {
        self.collect_chunk_records::<TextureAnimationsChunk>()
    }

    /// Replaces texture animations in the first `TXAN` chunk.
    pub fn set_texture_animations(&mut self, animations: &[TextureAnimation]) {
        self.replace_chunk(TextureAnimationsChunk::new(animations.to_vec()));
    }
}
