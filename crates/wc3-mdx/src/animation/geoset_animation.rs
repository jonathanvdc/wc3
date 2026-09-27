//! Geoset animation records in `GEOA` chunks.
use crate::ModelVersion;
use bitfield::bitfield;
crate::animation::track_group! {
    pub enum GeosetTrack {
        Alpha: GeosetAlpha,
        Color: GeosetColor,
    }
}

use crate::Color;
use crate::KnownChunk;

use crate::GeosetAnimationsChunk;
use crate::Model;
use crate::{Readable, Writable};

bitfield! {
    /// Geoset animation rendering flags, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct GeosetAnimationFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `DROP_SHADOW` bit.
    pub drop_shadow, set_drop_shadow: 0;
    /// Returns or changes the `COLOR` bit.
    pub color, set_color: 1;
}

/// A geoset animation with decoded alpha and color tracks.
#[derive(Clone, Debug, PartialEq, Readable, Writable)]
#[mdx(sized(tag = GeosetAnimationsChunk::TAG))]
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
        GeosetAnimationFlags(self.raw_flags)
    }
    /// Changes decoded rendering bits.
    pub fn set_flags(&mut self, flags: GeosetAnimationFlags) {
        self.raw_flags = flags.bits();
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
