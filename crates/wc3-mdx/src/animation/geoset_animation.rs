//! Geoset animation records in `GEOA` chunks.
use crate::ModelVersion;
crate::animation::track_group! {
    pub enum GeosetTrack {
        Alpha: GeosetAlpha,
        Color: GeosetColor,
    }
}

use crate::Color;
use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;

use crate::{Cursor, GeosetAnimationsChunk};
use crate::{DecodeError, Model};
use crate::{Readable, Writable};

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
    tracks: Vec<GeosetTrack>,
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
    pub fn tracks(&self) -> &[GeosetTrack] {
        &self.tracks
    }
    /// Replaces alpha and color tracks.
    pub fn set_tracks(&mut self, tracks: &[GeosetTrack]) {
        self.tracks = tracks.to_vec();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all `GEOA` records in file order.
    pub fn geoset_animations(&self) -> Vec<GeosetAnimation> {
        self.collect_chunk_records::<GeosetAnimationsChunk>()
    }

    /// Replaces geoset animations in the first `GEOA` chunk.
    pub fn set_geoset_animations(&mut self, animations: &[GeosetAnimation]) {
        self.replace_chunk(GeosetAnimationsChunk::new(animations.to_vec()));
    }
}

impl Readable for GeosetAnimation {
    fn read_from(source: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let mut cursor = source.slice_u32_sized()?;
        let alpha = cursor.read()?;
        let raw_flags = cursor.read()?;
        let color = cursor.read()?;
        let geoset_id = cursor.read()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            tracks.push(cursor.read::<GeosetTrack>()?);
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

impl Writable for &GeosetAnimation {
    fn write_to(self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let marker = bytes.begin_sized();
        bytes.write(self.alpha)?;
        bytes.write(self.raw_flags)?;
        for value in self.color {
            bytes.write(value)?;
        }
        bytes.write(self.geoset_id)?;
        for track in &self.tracks {
            bytes.write(track)?;
        }
        bytes.finish_sized(marker, GeosetAnimationsChunk::TAG)?;
        Ok(())
    }
}
