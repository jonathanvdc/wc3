//! Geoset animation records in `GEOA` chunks.
use crate::model::ModelVersion;
use crate::model::{mdl, mdx};
use bitfield::bitfield;
crate::model::animation::track_group! {
    pub enum GeosetTrack {
        Alpha: GeosetAlpha,
        Color: GeosetColor,
    }
}

use crate::model::Color;
use crate::model::KnownChunk;

use crate::model::GeosetAnimationsChunk;
use crate::model::Model;

bitfield! {
    /// Geoset animation rendering flags, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write)]
    pub struct GeosetAnimationFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `DROP_SHADOW` bit.
    pub drop_shadow, set_drop_shadow: 0;
    /// Returns or changes the `COLOR` bit.
    pub color, set_color: 1;
}

/// A geoset animation with decoded alpha and color tracks.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = GeosetAnimationsChunk::TAG))]
#[mdl(
    block = "GeosetAnim",
    default,
    write_order(alpha, flags, geoset_id, color, tracks)
)]
pub struct GeosetAnimation {
    #[mdl(animatable = "Alpha", track = "GeosetTrack::Alpha")]
    alpha: f32,
    #[mdl(flags(DropShadow = 1), allow_bits = 2)]
    flags: GeosetAnimationFlags,
    #[mdl(
        animatable = "Color",
        track = "GeosetTrack::Color",
        enabled_if = "Self::uses_color",
        enable_with = "Self::enable_color"
    )]
    color: Color,
    #[mdl(property = "GeosetId", required)]
    geoset_id: u32,
    #[mdl(tracks)]
    tracks: Vec<GeosetTrack>,
}

impl Default for GeosetAnimation {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            flags: GeosetAnimationFlags::default(),
            color: [1.0; 3],
            geoset_id: 0,
            tracks: Vec::new(),
        }
    }
}

impl GeosetAnimation {
    /// Creates a geoset animation with opaque white color and full alpha.
    pub fn new(geoset_id: u32) -> Self {
        Self {
            geoset_id,
            ..Self::default()
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
        self.flags
    }
    /// Changes decoded rendering bits.
    pub fn set_flags(&mut self, flags: GeosetAnimationFlags) {
        self.flags = flags;
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

impl GeosetAnimation {
    fn uses_color(&self) -> bool {
        self.flags.color()
    }
    fn enable_color(&mut self) {
        self.flags.set_color(true);
    }
}
