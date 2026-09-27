//! Typed texture animation tracks in `TXAN` chunks.
use crate::ModelVersion;
crate::animation::track_group! {
    pub enum TextureAnimationTrack {
        Translation: TextureTranslation,
        Rotation: TextureRotation,
        Scaling: TextureScaling,
    }
}

use crate::KnownChunk;

use crate::Model;
use crate::TextureAnimationsChunk;
use crate::{Readable, Writable};

/// A texture animation containing translation, rotation, and scaling tracks.
#[derive(Clone, Debug, Default, PartialEq, Readable, Writable)]
#[mdx(sized(tag = TextureAnimationsChunk::TAG))]
pub struct TextureAnimation {
    tracks: Vec<TextureAnimationTrack>,
}

impl TextureAnimation {
    /// Creates an empty texture animation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrows decoded tracks without reparsing.
    pub fn tracks(&self) -> &[TextureAnimationTrack] {
        &self.tracks
    }

    /// Replaces texture animation tracks.
    pub fn set_tracks(&mut self, tracks: &[TextureAnimationTrack]) {
        self.tracks = tracks.to_vec();
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
