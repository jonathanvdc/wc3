//! Typed texture animation tracks in `TXAN` chunks.
use crate::Encoder;
use crate::Tag;
use crate::ValueError;

use crate::Record;
use crate::{AnimationTrack, Error, Model};
use crate::{Cursor, ModelChunk, TextureAnimationsChunk};

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
                    record: TextureAnimation::TAG,
                    track: track.tag,
                });
            }
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track_tag(tag: Tag) -> bool {
    matches!(&tag, b"KTAT" | b"KTAR" | b"KTAS")
}

impl Model {
    /// Decodes all texture animations in `TXAN` chunks.
    pub fn texture_animations(&self) -> Result<Vec<TextureAnimation>, Error> {
        self.collect_chunk_records::<TextureAnimationsChunk>(|chunk| match chunk {
            ModelChunk::TextureAnimations(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces texture animations in the first `TXAN` chunk.
    pub fn set_texture_animations(&mut self, animations: &[TextureAnimation]) {
        self.replace_chunk(ModelChunk::TextureAnimations(TextureAnimationsChunk::new(
            animations.to_vec(),
        )));
    }
}

impl Record for TextureAnimation {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Self::TAG,
                    offset,
                });
            }

            tracks.push(track);
        }
        cursor.finish()?;
        Ok(Self { tracks })
    }

    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        for track in &self.tracks {
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TextureAnimation::TAG,
                    offset: bytes.position() - start,
                });
            }
            track.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, TextureAnimation::TAG)?;
        Ok(())
    }
}

impl TextureAnimation {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"TXAN";
}
