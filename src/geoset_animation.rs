//! Geoset animation records in `GEOA` chunks.

use crate::Record;
use crate::{AnimationTrack, Error, Model};
use crate::{Cursor, GeosetAnimationsChunk, ModelChunk};

pub(crate) const HEADER_SIZE: usize = 28;

/// Geoset animation rendering flags, retaining unknown bits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GeosetAnimationFlags(u32);

impl GeosetAnimationFlags {
    pub const DROP_SHADOW: Self = Self(1);
    pub const COLOR: Self = Self(2);
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
    pub const fn bits(self) -> u32 {
        self.0
    }
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub fn set(&mut self, other: Self, enabled: bool) {
        if enabled {
            self.0 |= other.0;
        } else {
            self.0 &= !other.0;
        }
    }
}

/// A geoset animation with decoded alpha and color tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct GeosetAnimation {
    alpha: f32,
    raw_flags: u32,
    color: [f32; 3],
    geoset_id: u32,
    tracks: Vec<AnimationTrack>,
}

impl GeosetAnimation {
    /// Creates a geoset animation with opaque white color and full alpha.
    pub fn new(geoset_id: u32) -> Self {
        Self {
            alpha: 1.0,
            raw_flags: 0,
            color: [1.0; 3],
            geoset_id,
            tracks: Vec::new(),
        }
    }

    /// Returns the base alpha.
    pub fn alpha(&self) -> f32 {
        self.alpha
    }
    /// Changes the base alpha.
    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha;
    }
    /// Returns decoded rendering flags.
    pub fn flags(&self) -> GeosetAnimationFlags {
        GeosetAnimationFlags::from_bits(self.raw_flags)
    }
    /// Returns exact rendering bits.
    pub fn raw_flags(&self) -> u32 {
        self.raw_flags
    }
    /// Changes decoded rendering bits.
    pub fn set_flags(&mut self, flags: GeosetAnimationFlags) {
        self.raw_flags = flags.bits();
    }
    /// Changes exact rendering bits.
    pub fn set_raw_flags(&mut self, flags: u32) {
        self.raw_flags = flags;
    }
    /// Returns base RGB color.
    pub fn color(&self) -> [f32; 3] {
        self.color
    }
    /// Changes base RGB color.
    pub fn set_color(&mut self, color: [f32; 3]) {
        self.color = color;
    }
    /// Returns the referenced geoset index.
    pub fn geoset_id(&self) -> u32 {
        self.geoset_id
    }
    /// Changes the referenced geoset index.
    pub fn set_geoset_id(&mut self, id: u32) {
        self.geoset_id = id;
    }
    /// Borrows alpha and color tracks without reparsing.
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }
    /// Replaces alpha and color tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut size = HEADER_SIZE;
        for track in tracks {
            if !matches!(&track.tag, b"KGAO" | b"KGAC") {
                return Err(Error::MalformedRecord {
                    tag: GeosetAnimation::TAG,
                    offset: size,
                });
            }
            size = size
                .checked_add(track.encode()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: GeosetAnimation::TAG,
                    size: usize::MAX,
                })?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

impl Model {
    /// Decodes all `GEOA` records in file order.
    pub fn geoset_animations(&self) -> Result<Vec<GeosetAnimation>, Error> {
        self.collect_chunk_records::<GeosetAnimationsChunk>(|chunk| match chunk {
            ModelChunk::GeosetAnimations(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces geoset animations in the first `GEOA` chunk.
    pub fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]) -> Result<(), Error> {
        self.replace_chunk(ModelChunk::GeosetAnimations(GeosetAnimationsChunk::new(
            animations.to_vec(),
        )))
    }
}

impl Record for GeosetAnimation {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let alpha = cursor.read_f32()?;
        let raw_flags = cursor.read_u32()?;
        let color = cursor.read_vec3()?;
        let geoset_id = cursor.read_u32()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !matches!(&track.tag, b"KGAO" | b"KGAC") {
                return Err(Error::MalformedRecord {
                    tag: Self::TAG,
                    offset,
                });
            }

            tracks.push(track);
        }
        cursor.finish()?;
        Ok(Self {
            alpha,
            raw_flags,
            color,
            geoset_id,
            tracks,
        })
    }

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; HEADER_SIZE];
        bytes[4..8].copy_from_slice(&self.alpha.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.raw_flags.to_le_bytes());
        for (index, value) in self.color.into_iter().enumerate() {
            bytes[12 + index * 4..16 + index * 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[24..28].copy_from_slice(&self.geoset_id.to_le_bytes());
        for track in &self.tracks {
            if !matches!(&track.tag, b"KGAO" | b"KGAC") {
                return Err(Error::MalformedRecord {
                    tag: GeosetAnimation::TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.encode()?);
        }
        let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
            tag: GeosetAnimation::TAG,
            size: bytes.len(),
        })?;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
    }
}

impl GeosetAnimation {
    /// The tag of the chunk containing this record.
    pub const TAG: [u8; 4] = *b"GEOA";
}
