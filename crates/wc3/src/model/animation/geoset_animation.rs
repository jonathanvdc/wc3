//! Animated opacity and color for individual geosets.
use crate::model::Animatable;
use crate::model::Color;
use crate::model::GeosetAnimationsChunk;
use crate::model::KnownChunk;
use crate::model::Model;
use crate::model::ModelVersion;
use crate::model::{mdl, mdx};
use bitfield::bitfield;

bitfield! {
    /// Geoset animation rendering flags, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
    #[mdl(flags(DropShadow = 1), allow_bits = 2)]
    pub struct GeosetAnimationFlags(u32);
    /// Returns the exact stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes the `DROP_SHADOW` bit.
    pub drop_shadow, set_drop_shadow: 0;
    /// Returns or changes the `COLOR` bit.
    pub color, set_color: 1;
}

/// Opacity and optional color animation applied to one geoset.
#[derive(Clone, Debug, PartialEq, mdl::Read, mdl::Write)]
#[mdl(
    block = "GeosetAnim",
    default,
    write_order(alpha, flags, geoset_id, color)
)]
pub struct GeosetAnimation {
    #[mdl(property = "Alpha")]
    /// Base alpha.
    pub alpha: Animatable<f32>,
    #[mdl(flatten)]
    /// Rendering flags.
    pub flags: GeosetAnimationFlags,
    #[mdl(
        property = "Color",
        enabled_if = "Self::uses_color",
        enable_with = "Self::enable_color"
    )]
    /// Base RGB color.
    pub color: Animatable<Color>,
    #[mdl(property = "GeosetId", required)]
    /// Referenced geoset index.
    pub geoset_id: u32,
}

impl Default for GeosetAnimation {
    fn default() -> Self {
        Self {
            alpha: Animatable::Static(1.0),
            flags: GeosetAnimationFlags::default(),
            color: Animatable::Static([1.0; 3]),
            geoset_id: 0,
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

// MDX stores the fixed color as BGR, while KGAC tracks and MDL use RGB.
// Keep this wire representation private so the public model consistently uses RGB.
#[derive(mdx::Read, mdx::Write)]
#[mdx(sized(tag = GeosetAnimationsChunk::TAG))]
struct BinaryGeosetAnimation {
    #[mdx(tag = *b"KGAO")]
    alpha: Animatable<f32>,
    flags: GeosetAnimationFlags,
    #[mdx(tag = *b"KGAC")]
    color: Animatable<Color>,
    geoset_id: u32,
}

fn swap_base_color(color: &mut Animatable<Color>) {
    if let Some(mut value) = color.value().copied() {
        value.swap(0, 2);
        color.set_value(value);
    }
}

impl mdx::Read for GeosetAnimation {
    fn read_mdx(cursor: &mut mdx::Cursor<'_>) -> Result<Self, mdx::ReadError> {
        let mut value: BinaryGeosetAnimation = cursor.read()?;
        swap_base_color(&mut value.color);
        Ok(Self {
            alpha: value.alpha,
            flags: value.flags,
            color: value.color,
            geoset_id: value.geoset_id,
        })
    }
}

impl mdx::Write for GeosetAnimation {
    fn write_mdx(&self, encoder: &mut mdx::Encoder<'_>) -> Result<(), mdx::WriteError> {
        let mut color = self.color.clone();
        swap_base_color(&mut color);
        encoder.write(&BinaryGeosetAnimation {
            alpha: self.alpha.clone(),
            flags: self.flags,
            color,
            geoset_id: self.geoset_id,
        })
    }
}
