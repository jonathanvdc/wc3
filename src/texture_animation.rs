//! Texture animation records in `TXAN` chunks.

use crate::{AnimationTrack, Error, Model};

const TAG: [u8; 4] = *b"TXAN";

/// A texture animation containing translation, rotation, and scaling tracks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextureAnimation {
    bytes: Vec<u8>,
}

impl Default for TextureAnimation {
    fn default() -> Self {
        Self {
            bytes: 4u32.to_le_bytes().to_vec(),
        }
    }
}

impl TextureAnimation {
    /// Creates an empty texture animation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Wraps one inclusive-size record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 4 {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let size = u32::from_le_bytes(bytes[..4].try_into().expect("four-byte size")) as usize;
        if size != bytes.len() {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Decodes `KTAT`, `KTAR`, and `KTAS` tracks.
    pub fn tracks(&self) -> Result<Vec<AnimationTrack>, Error> {
        let mut tracks = Vec::new();
        let mut offset = 4;
        while offset < self.bytes.len() {
            let (track, consumed) = AnimationTrack::parse(&self.bytes, offset)?;
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord { tag: TAG, offset });
            }
            tracks.push(track);
            offset += consumed;
        }
        Ok(tracks)
    }

    /// Replaces tracks and updates the inclusive size.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut bytes = vec![0; 4];
        for track in tracks {
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        self.bytes = bytes;
        Ok(())
    }
}

fn is_track_tag(tag: [u8; 4]) -> bool {
    matches!(&tag, b"KTAT" | b"KTAR" | b"KTAS")
}

impl Model {
    /// Decodes all texture animations in `TXAN` chunks.
    pub fn texture_animations(&self) -> Result<Vec<TextureAnimation>, Error> {
        let mut animations = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            let mut offset = 0;
            while offset < chunk.data.len() {
                let size_bytes = chunk
                    .data
                    .get(offset..offset.saturating_add(4))
                    .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                let size =
                    u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize;
                if size < 4 {
                    return Err(Error::MalformedRecord { tag: TAG, offset });
                }
                let end = offset
                    .checked_add(size)
                    .filter(|&end| end <= chunk.data.len())
                    .ok_or(Error::MalformedRecord { tag: TAG, offset })?;
                animations.push(TextureAnimation::from_bytes(&chunk.data[offset..end])?);
                offset = end;
            }
        }
        Ok(animations)
    }

    /// Replaces texture animations in the first `TXAN` chunk.
    pub fn set_texture_animations(&mut self, animations: &[TextureAnimation]) -> Result<(), Error> {
        let size = animations.iter().try_fold(0usize, |sum, animation| {
            sum.checked_add(animation.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for animation in animations {
            data.extend_from_slice(animation.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
