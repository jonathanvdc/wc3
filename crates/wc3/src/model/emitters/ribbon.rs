//! Animated ribbon trails.
use crate::model::mdl::Span;
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::Color;
use crate::model::KnownChunk;
use crate::model::ModelVersion;
use crate::model::RibbonEmittersChunk;
use crate::model::{mdl, mdx};
use crate::model::{Animatable, Track};
use crate::model::{Model, Node};

/// A trail emitted from an animated node and rendered with a model material.
///
/// Height, opacity, color, and texture-cell selection can be animated.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = RibbonEmittersChunk::TAG))]
#[mdl(
    block = "RibbonEmitter",
    after_read = "finish_ribbon",
    validate_write = "validate_ribbon"
)]
pub struct RibbonEmitter {
    #[mdl(flatten)]
    /// Embedded node.
    pub node: Node,
    #[mdx(tag = *b"KRHA")]
    #[mdl(property = "HeightAbove", default)]
    /// Ribbon height above its emission point.
    pub height_above: Animatable<f32>,
    #[mdx(tag = *b"KRHB")]
    #[mdl(property = "HeightBelow", default)]
    /// Ribbon height below its emission point.
    pub height_below: Animatable<f32>,
    #[mdx(tag = *b"KRAL")]
    #[mdl(property = "Alpha", default)]
    /// Opacity when no alpha track is active.
    pub alpha: Animatable<f32>,
    #[mdx(tag = *b"KRCO")]
    #[mdl(property = "Color", default)]
    pub color: Animatable<Color>,
    #[mdl(property = "LifeSpan", default)]
    pub life_span: f32,
    #[mdx(tag = *b"KRTX")]
    #[mdl(property = "TextureSlot", default, bare_static)]
    /// Texture-atlas cell used when no texture-slot track is active.
    pub texture_slot: Animatable<u32>,
    #[mdl(property = "EmissionRate", default)]
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
    pub gravity: f32,
    #[mdx(tag = *b"KRVS")]
    #[mdl(property = "Visibility")]
    /// Optional visibility animation.
    pub visibility: Option<Track<f32>>,
}

impl RibbonEmitter {
    /// Creates a ribbon emitter with zeroed fixed properties.
    pub fn new(node: Node) -> Self {
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

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of ribbon emitter records in file order.
    pub fn ribbon_emitters(&self) -> Vec<RibbonEmitter> {
        self.collect_chunk_records::<RibbonEmittersChunk>()
    }

    /// Replaces ribbon emitters in the first `RIBB` chunk.
    pub fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter]) {
        self.replace_chunk(RibbonEmittersChunk::new(emitters.to_vec()));
    }
}

fn zero_gravity(value: &f32) -> bool {
    value.to_bits() == 0
}
fn finish_ribbon(value: &mut RibbonEmitter, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x4000);
    Ok(())
}
fn validate_ribbon(value: &RibbonEmitter) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x4000)
}
