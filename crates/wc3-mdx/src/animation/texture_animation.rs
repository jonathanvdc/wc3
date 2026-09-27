//! Typed texture animation tracks in `TXAN` chunks.
use crate::ModelVersion;
crate::animation::track_group! {
    pub enum TextureAnimationTrack {
        Translation: TextureTranslation,
        Rotation: TextureRotation,
        Scaling: TextureScaling,
    }
}

use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;

use crate::{Cursor, TextureAnimationsChunk};
use crate::{Decodable, Encodable};
use crate::{DecodeError, Model};

/// A texture animation containing translation, rotation, and scaling tracks.
#[derive(Clone, Debug, Default, PartialEq)]
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

impl Decodable for TextureAnimation {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, DecodeError> {
        let mut cursor = source.slice_u32_sized()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            tracks.push(cursor.read::<TextureAnimationTrack>()?);
        }
        cursor.finish()?;
        Ok(Self { tracks })
    }
}

impl Encodable for TextureAnimation {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let marker = bytes.begin_sized();
        for track in &self.tracks {
            bytes.write(track);
        }
        bytes.finish_sized(marker, TextureAnimationsChunk::TAG)?;
        Ok(())
    }
}
