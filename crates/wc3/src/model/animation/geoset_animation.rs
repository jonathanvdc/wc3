//! Animated opacity and color for individual geosets.
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

/// Opacity and optional color animation applied to one geoset.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = GeosetAnimationsChunk::TAG))]
#[mdl(
    block = "GeosetAnim",
    default,
    write_order(alpha, flags, geoset_id, color, tracks)
)]
pub struct GeosetAnimation {
    #[mdl(animatable = "Alpha", track = "GeosetTrack::Alpha")]
    /// Base alpha.
    pub alpha: f32,
    #[mdl(flags(DropShadow = 1), allow_bits = 2)]
    /// Rendering flags.
    pub flags: GeosetAnimationFlags,
    #[mdl(
        animatable = "Color",
        track = "GeosetTrack::Color",
        enabled_if = "Self::uses_color",
        enable_with = "Self::enable_color"
    )]
    /// Base RGB color.
    pub color: Color,
    #[mdl(property = "GeosetId", required)]
    /// Referenced geoset index.
    pub geoset_id: u32,
    #[mdl(tracks)]
    /// Animated opacity and color channels.
    pub tracks: Vec<GeosetTrack>,
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
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of `GEOA` records in file order.
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
