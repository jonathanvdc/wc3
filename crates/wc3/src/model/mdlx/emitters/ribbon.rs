//! Animated ribbon trails.
use crate::model::common::Ownership;
use crate::model::mdlx::mdl::Span;
use crate::model::mdlx::scene::{set_node_kind, validate_node_kind};
use crate::model::mdlx::Color;
use crate::model::mdlx::KnownChunk;
use crate::model::mdlx::ModelDialect;
use crate::model::mdlx::RibbonEmittersChunk;
use crate::model::mdlx::{mdl, mdx};
use crate::model::mdlx::{Animatable, Track};
use crate::model::mdlx::{Model, Node};

/// A trail emitted from an animated node and rendered with a model material.
///
/// Height, opacity, color, and texture-cell selection can be animated.
#[derive(
    Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write, Ownership, mdx::ReadOwned,
)]
#[mdx(sized(tag = RibbonEmittersChunk::TAG))]
#[mdl(
    block = "RibbonEmitter",
    after_read = "finish_ribbon",
    validate_write = "validate_ribbon"
)]
pub struct RibbonEmitter<'data> {
    #[mdl(flatten)]
    /// Embedded node.
    pub node: Node<'data>,
    #[mdx(tag = *b"KRHA")]
    #[mdl(property = "HeightAbove", default)]
    /// Ribbon height above its emission point.
    pub height_above: Animatable<'data, f32>,
    #[mdx(tag = *b"KRHB")]
    #[mdl(property = "HeightBelow", default)]
    /// Ribbon height below its emission point.
    pub height_below: Animatable<'data, f32>,
    #[mdx(tag = *b"KRAL")]
    #[mdl(property = "Alpha", default)]
    /// Opacity when no alpha track is active.
    pub alpha: Animatable<'data, f32>,
    #[mdx(tag = *b"KRCO")]
    #[mdl(property = "Color", default)]
    /// Animated ribbon color.
    pub color: Animatable<'data, Color>,
    #[mdl(property = "LifeSpan", default)]
    /// Lifetime of each trail section in seconds.
    pub life_span: f32,
    #[mdx(tag = *b"KRTX")]
    #[mdl(property = "TextureSlot", default, bare_static)]
    /// Texture-atlas cell used when no texture-slot track is active.
    pub texture_slot: Animatable<'data, u32>,
    #[mdl(property = "EmissionRate", default)]
    /// Number of trail sections emitted per second.
    pub emission_rate: u32,
    #[mdl(property = "Rows", default)]
    /// Number of rows in the texture atlas.
    pub rows: u32,
    #[mdl(property = "Columns", default)]
    /// Number of columns in the texture atlas.
    pub columns: u32,
    #[mdl(property = "MaterialID", default)]
    /// Index into the model material collection.
    pub material_id: u32,
    #[mdl(property = "Gravity", default, skip_if = "zero_gravity")]
    /// Gravity applied to trail sections in model units per second squared.
    pub gravity: f32,
    #[mdx(tag = *b"KRVS")]
    #[mdl(property = "Visibility")]
    /// Optional visibility animation.
    pub visibility: Option<Track<'data, f32>>,
}

impl<'data> RibbonEmitter<'data> {
    /// Creates a ribbon emitter with zeroed fixed properties.
    pub fn new(node: Node<'data>) -> Self {
        Self {
            node,
            height_above: Default::default(),
            height_below: Default::default(),
            alpha: Default::default(),
            color: Default::default(),
            life_span: Default::default(),
            texture_slot: Default::default(),
            emission_rate: Default::default(),
            rows: Default::default(),
            columns: Default::default(),
            material_id: Default::default(),
            gravity: Default::default(),
            visibility: None,
        }
    }
}

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn ribbon_emitters<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'s RibbonEmitter<'data>> + use<'data, 's, D> {
        self.records::<RibbonEmittersChunk<'_>>()
    }

    /// Mutably borrows records in file order without replacing their chunks.
    pub fn ribbon_emitters_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = &'s mut RibbonEmitter<'data>> + use<'data, 's, D> {
        self.records_mut::<RibbonEmittersChunk<'_>>()
    }

    /// Replaces ribbon emitters with one `RIBB` chunk, removing any duplicate chunks.
    pub fn set_ribbon_emitters(
        &mut self,
        emitters: impl IntoIterator<Item = RibbonEmitter<'data>>,
    ) {
        self.replace_chunk(RibbonEmittersChunk::new(emitters.into_iter().collect()));
    }
}

fn zero_gravity(value: &f32) -> bool {
    value.to_bits() == 0
}
fn finish_ribbon<'data>(value: &mut RibbonEmitter<'data>, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x4000);
    Ok(())
}
fn validate_ribbon<'data>(value: &RibbonEmitter<'data>) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x4000)
}
