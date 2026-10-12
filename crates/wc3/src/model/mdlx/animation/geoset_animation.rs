//! Animated opacity and color for individual geosets.
use crate::model::common::Ownership;
use crate::model::mdlx::Animatable;
use crate::model::mdlx::Color;
use crate::model::mdlx::GeosetAnimationsChunk;
use crate::model::mdlx::KnownChunk;
use crate::model::mdlx::Model;
use crate::model::mdlx::ModelDialect;
use crate::model::mdlx::{mdl, mdx};
use bitfield::bitfield;

bitfield! {
    /// Geoset animation rendering flags, retaining unknown bits.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::ReadOwned, mdx::Write, mdl::Read, mdl::Write)]
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
#[derive(Clone, Debug, PartialEq, mdl::Read, mdl::Write, Ownership, mdx::ReadOwned)]
#[mdl(
    block = "GeosetAnim",
    default,
    write_order(alpha, flags, geoset_id, color)
)]
pub struct GeosetAnimation<'data> {
    #[mdl(property = "Alpha")]
    /// Base alpha.
    pub alpha: Animatable<'data, f32>,
    #[mdl(flatten)]
    /// Rendering flags.
    pub flags: GeosetAnimationFlags,
    #[mdl(
        property = "Color",
        enabled_if = "Self::uses_color",
        enable_with = "Self::enable_color"
    )]
    /// Base RGB color.
    pub color: Animatable<'data, Color>,
    #[mdl(property = "GeosetId", required)]
    /// Referenced geoset index.
    pub geoset_id: u32,
}

impl<'data> Default for GeosetAnimation<'data> {
    fn default() -> Self {
        Self {
            alpha: Animatable::Static(1.0),
            flags: GeosetAnimationFlags::default(),
            color: Animatable::Static([1.0; 3]),
            geoset_id: 0,
        }
    }
}

impl<'data> GeosetAnimation<'data> {
    /// Creates a geoset animation with opaque white color and full alpha.
    pub fn new(geoset_id: u32) -> Self {
        Self {
            geoset_id,
            ..Self::default()
        }
    }
}

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn geoset_animations<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'s GeosetAnimation<'data>> + use<'data, 's, D> {
        self.records::<GeosetAnimationsChunk<'data>>()
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn geoset_animations_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut GeosetAnimation<'data>> + use<'data, 's, D> {
        self.records_mut::<GeosetAnimationsChunk<'data>>()
    }

    /// Replaces geoset animations with one `GEOA` chunk, removing any duplicate chunks.
    pub fn set_geoset_animations(
        &mut self,
        animations: impl IntoIterator<Item = GeosetAnimation<'data>>,
    ) {
        self.replace_chunk(GeosetAnimationsChunk::new(animations.into_iter().collect()));
    }
}

impl<'data> GeosetAnimation<'data> {
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
struct BinaryGeosetAnimation<'data> {
    #[mdx(tag = *b"KGAO")]
    alpha: Animatable<'data, f32>,
    flags: GeosetAnimationFlags,
    #[mdx(tag = *b"KGAC")]
    color: Animatable<'data, Color>,
    geoset_id: u32,
}

fn swap_base_color<'data>(color: &mut Animatable<'data, Color>) {
    if let Some(mut value) = color.value().copied() {
        value.swap(0, 2);
        color.set_value(value);
    }
}

impl<'data> mdx::Read<'data> for GeosetAnimation<'data> {
    fn read_mdx(cursor: &mut mdx::Cursor<'data>) -> Result<Self, mdx::ReadError> {
        let mut value: BinaryGeosetAnimation<'data> = cursor.read()?;
        swap_base_color(&mut value.color);
        Ok(Self {
            alpha: value.alpha,
            flags: value.flags,
            color: value.color,
            geoset_id: value.geoset_id,
        })
    }
}

impl<'data> mdx::Write for GeosetAnimation<'data> {
    fn write_mdx(&self, encoder: &mut mdx::Encoder<'_>) -> Result<(), mdx::WriteError> {
        let mut color = self.color.as_borrowed();
        swap_base_color(&mut color);
        encoder.write(&BinaryGeosetAnimation {
            alpha: self.alpha.as_borrowed(),
            flags: self.flags,
            color,
            geoset_id: self.geoset_id,
        })
    }
}
