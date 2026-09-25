//! Typed texture animation tracks in `TXAN` chunks.

use crate::{AnimationTrack, Error, Model};

const TAG: [u8; 4] = *b"TXAN";

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

    /// Parses one inclusive-size record and its tracks.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let size_bytes = bytes.get(..4).ok_or(Error::MalformedRecord {
            tag: TAG,
            offset: 0,
        })?;
        if u32::from_le_bytes(size_bytes.try_into().expect("four-byte size")) as usize
            != bytes.len()
        {
            return Err(Error::MalformedRecord {
                tag: TAG,
                offset: 0,
            });
        }
        let mut tracks = Vec::new();
        let mut offset = 4;
        while offset < bytes.len() {
            let (track, consumed) = AnimationTrack::parse(bytes, offset)?;
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord { tag: TAG, offset });
            }
            tracks.push(track);
            offset += consumed;
        }
        Ok(Self { tracks })
    }

    /// Serializes the inclusive-size record.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        for track in &self.tracks {
            if !is_track_tag(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
            tag: TAG,
            size: bytes.len(),
        })?;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
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
                    tag: TAG,
                    offset: size,
                });
            }
            size = size
                .checked_add(track.to_bytes()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
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
        let mut data = Vec::new();
        for animation in animations {
            data.extend_from_slice(&animation.to_bytes()?);
            if data.len() > u32::MAX as usize {
                return Err(Error::ChunkTooLarge {
                    tag: TAG,
                    size: data.len(),
                });
            }
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
