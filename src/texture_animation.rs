//! Typed texture animation tracks in `TXAN` chunks.

use crate::Record;
use crate::{AnimationTrack, Error, Model};

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
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut size = 4usize;
        for track in tracks {
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TextureAnimation::TAG,
                    offset: size,
                });
            }
            size = size
                .checked_add(track.encode()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TextureAnimation::TAG,
                    size: usize::MAX,
                })?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track_tag(tag: [u8; 4]) -> bool {
    matches!(&tag, b"KTAT" | b"KTAR" | b"KTAS")
}

impl Model {
    /// Decodes all texture animations in `TXAN` chunks.
    pub fn texture_animations(&self) -> Result<Vec<TextureAnimation>, Error> {
        self.collect_chunk_records::<crate::TextureAnimationsChunk>(|chunk| match chunk {
            crate::ModelChunk::TextureAnimations(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces texture animations in the first `TXAN` chunk.
    pub fn set_texture_animations(&mut self, animations: &[TextureAnimation]) -> Result<(), Error> {
        let mut data = Vec::new();
        for animation in animations {
            data.extend_from_slice(&animation.encode()?);
            if data.len() > u32::MAX as usize {
                return Err(Error::ChunkTooLarge {
                    tag: TextureAnimation::TAG,
                    size: data.len(),
                });
            }
        }
        self.replace_chunks(TextureAnimation::TAG, data)?;
        Ok(())
    }
}

impl Record for TextureAnimation {
    fn decode_one(bytes: &[u8], _version: u32) -> Result<(Self, usize), Error> {
        let length = crate::record::sized_record_len(bytes, Self::TAG, 4, u32::MAX, 0)?;
        let bytes = &bytes[..length];
        let value = {
            let size_bytes = bytes.get(..4).ok_or(Error::MalformedRecord {
                tag: TextureAnimation::TAG,
                offset: 0,
            })?;
            if u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize
                != bytes.len()
            {
                return Err(Error::MalformedRecord {
                    tag: TextureAnimation::TAG,
                    offset: 0,
                });
            }
            let mut tracks = Vec::new();
            let mut offset = 4;
            while offset < bytes.len() {
                let (track, consumed) = AnimationTrack::parse(bytes, offset)?;
                if !is_track_tag(track.tag) {
                    return Err(Error::MalformedRecord {
                        tag: TextureAnimation::TAG,
                        offset,
                    });
                }
                tracks.push(track);
                offset += consumed;
            }
            Ok(Self { tracks })
        }?;
        Ok((value, length))
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        for track in &self.tracks {
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TextureAnimation::TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.encode()?);
        }
        let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
            tag: TextureAnimation::TAG,
            size: bytes.len(),
        })?;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
    }
}

impl TextureAnimation {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"TXAN";
}
