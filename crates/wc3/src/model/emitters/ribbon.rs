//! Ribbon emitter records in `RIBB` chunks.
use crate::model::mdl::Span;
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::ModelVersion;
use crate::model::{mdl, mdx};
crate::model::animation::track_group! {
    pub enum RibbonTrack {
        Visibility: RibbonVisibility,
        HeightAbove: RibbonHeightAbove,
        HeightBelow: RibbonHeightBelow,
        Alpha: RibbonAlpha,
        Color: RibbonColor,
        TextureSlot: RibbonTextureSlot,
    }
}

use crate::model::Color;
use crate::model::KnownChunk;

use crate::model::RibbonEmittersChunk;
use crate::model::{Model, Node};

/// One ribbon emitter with decoded fixed properties and animation tracks.
///
/// MDL reading restores the ribbon object-kind bit; writing requires matching
/// node bits and default hidden bases. TextureSlot accepts a bare scalar alias
/// and writes the canonical static spelling.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = RibbonEmittersChunk::TAG))]
#[mdl(
    block = "RibbonEmitter",
    after_read = "finish_ribbon",
    validate_write = "validate_ribbon"
)]
pub struct RibbonEmitter {
    #[mdl(flatten)]
    node: Node,
    #[mdl(
        animatable = "HeightAbove",
        track = "RibbonTrack::HeightAbove",
        default
    )]
    pub height_above: f32,
    #[mdl(
        animatable = "HeightBelow",
        track = "RibbonTrack::HeightBelow",
        default
    )]
    pub height_below: f32,
    #[mdl(animatable = "Alpha", track = "RibbonTrack::Alpha", default)]
    pub alpha: f32,
    #[mdl(animatable = "Color", track = "RibbonTrack::Color", default)]
    pub color: Color,
    #[mdl(property = "LifeSpan", default)]
    pub life_span: f32,
    #[mdl(
        animatable = "TextureSlot",
        track = "RibbonTrack::TextureSlot",
        default,
        bare_static
    )]
    pub texture_slot: u32,
    #[mdl(property = "EmissionRate", default)]
    pub emission_rate: u32,
    #[mdl(property = "Rows", default)]
    pub rows: u32,
    #[mdl(property = "Columns", default)]
    pub columns: u32,
    #[mdl(property = "MaterialID", default)]
    pub material_id: u32,
    #[mdl(property = "Gravity", default, skip_if = "zero_gravity")]
    pub gravity: f32,
    #[mdl(tracks, channels(Visibility = "RibbonTrack::Visibility"))]
    tracks: Vec<RibbonTrack>,
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
            tracks: Vec::new(),
        }
    }

    /// Borrows the embedded node.
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Borrows the embedded node for editing.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }

    /// Borrows decoded ribbon animation tracks.
    pub fn tracks(&self) -> &[RibbonTrack] {
        &self.tracks
    }

    /// Replaces optional ribbon animation tracks.
    pub fn set_tracks(&mut self, tracks: &[RibbonTrack]) {
        self.tracks = tracks.to_vec();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all ribbon emitter records in file order.
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
