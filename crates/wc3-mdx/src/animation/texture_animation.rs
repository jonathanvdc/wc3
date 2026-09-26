//! Typed texture animation tracks in `TXAN` chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::ValueError;

use crate::{AnimationTrack, DecodeError, Model, TrackTag};
use crate::{Cursor, TextureAnimationsChunk};
use crate::{Decodable, Encodable};

/// A texture animation containing translation, rotation, and scaling tracks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureAnimation {
    tracks: Vec<AnimationTrack>,
}

impl TextureAnimation {
    /// Creates an empty texture animation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrows decoded tracks without reparsing.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }

    /// Replaces texture animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), ValueError> {
        for track in tracks {
            if !is_track_tag(track.tag) {
                return Err(ValueError::InvalidTrackTag {
                    record: TextureAnimationsChunk::TAG,
                    track: track.tag.bytes(),
                });
            }
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track_tag(tag: TrackTag) -> bool {
    matches!(
        tag,
        TrackTag::TextureTranslation | TrackTag::TextureRotation | TrackTag::TextureScaling
    )
}

impl Model {
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
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !is_track_tag(track.tag) {
                return Err(DecodeError::MalformedRecord {
                    tag: TextureAnimationsChunk::TAG,
                    offset,
                });
            }

            tracks.push(track);
        }
        cursor.finish()?;
        Ok(Self { tracks })
    }
}

impl Encodable for TextureAnimation {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        for track in &self.tracks {
            if !is_track_tag(track.tag) {
                return Err(EncodeError::MalformedRecord {
                    tag: TextureAnimationsChunk::TAG,
                    offset: bytes.position() - start,
                });
            }
            track.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, TextureAnimationsChunk::TAG)?;
        Ok(())
    }
}
