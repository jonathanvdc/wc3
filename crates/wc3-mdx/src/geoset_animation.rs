//! Geoset animation records in `GEOA` chunks.
use crate::EncodeError;
use crate::Encoder;
use crate::ValueError;
use crate::{Color, Tag};

use crate::{AnimationTrack, DecodeError, Model};
use crate::{Cursor, GeosetAnimationsChunk};
use crate::{Decodable, Encodable};

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
    color: Color,
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
    pub fn color(&self) -> Color {
        self.color
    }
    /// Changes base RGB color.
    pub fn set_color(&mut self, color: Color) {
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
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), ValueError> {
        for track in tracks {
            if !matches!(&track.tag, b"KGAO" | b"KGAC") {
                return Err(ValueError::InvalidTrackTag {
                    record: GeosetAnimation::TAG,
                    track: track.tag,
                });
            }
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

impl Model {
    /// Decodes all `GEOA` records in file order.
    pub fn geoset_animations(&self) -> Vec<GeosetAnimation> {
        self.collect_chunk_records::<GeosetAnimationsChunk>()
    }

    /// Replaces geoset animations in the first `GEOA` chunk.
    pub fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]) {
        self.replace_chunk(GeosetAnimationsChunk::new(animations.to_vec()));
    }
}

impl Decodable for GeosetAnimation {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, DecodeError> {
        let mut cursor = source.slice_u32_sized()?;
        let alpha = cursor.read()?;
        let raw_flags = cursor.read()?;
        let color = cursor.read()?;
        let geoset_id = cursor.read()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !matches!(&track.tag, b"KGAO" | b"KGAC") {
                return Err(DecodeError::MalformedRecord {
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
}

impl Encodable for GeosetAnimation {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        bytes.write(self.alpha);
        bytes.write(self.raw_flags);
        for value in self.color {
            bytes.write(value);
        }
        bytes.write(self.geoset_id);
        for track in &self.tracks {
            if !matches!(&track.tag, b"KGAO" | b"KGAC") {
                return Err(EncodeError::MalformedRecord {
                    tag: GeosetAnimation::TAG,
                    offset: bytes.position() - start,
                });
            }
            track.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, GeosetAnimation::TAG)?;
        Ok(())
    }
}

impl GeosetAnimation {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"GEOA";
}
