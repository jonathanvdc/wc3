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

/// Fixed properties of a ribbon emitter.
#[derive(Clone, Copy, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct RibbonFields {
    pub height_above: f32,
    pub height_below: f32,
    pub alpha: f32,
    pub color: Color,
    pub life_span: f32,
    pub texture_slot: u32,
    pub emission_rate: u32,
    pub rows: u32,
    pub columns: u32,
    pub material_id: u32,
    pub gravity: f32,
}

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
    #[mdl(project(
        #[mdl(animatable = "HeightAbove", track = "RibbonTrack::HeightAbove", default)] height_above: f32,
        #[mdl(animatable = "HeightBelow", track = "RibbonTrack::HeightBelow", default)] height_below: f32,
        #[mdl(animatable = "Alpha", track = "RibbonTrack::Alpha", default)] alpha: f32,
        #[mdl(animatable = "Color", track = "RibbonTrack::Color", default)] color: Color,
        #[mdl(property = "LifeSpan", default)] life_span: f32,
        #[mdl(animatable = "TextureSlot", track = "RibbonTrack::TextureSlot", default, bare_static)] texture_slot: u32,
        #[mdl(property = "EmissionRate", default)] emission_rate: u32,
        #[mdl(property = "Rows", default)] rows: u32,
        #[mdl(property = "Columns", default)] columns: u32,
        #[mdl(property = "MaterialID", default)] material_id: u32,
        #[mdl(property = "Gravity", default, skip_if = "zero_gravity")] gravity: f32,
    ))]
    fields: RibbonFields,
    #[mdl(tracks, channels(Visibility = "RibbonTrack::Visibility"))]
    tracks: Vec<RibbonTrack>,
}

impl RibbonEmitter {
    /// Creates a ribbon emitter with zeroed fixed properties.
    pub fn new(node: Node) -> Self {
        Self {
            node,
            fields: RibbonFields::default(),
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

    /// Returns fixed properties.
    pub fn fields(&self) -> RibbonFields {
        self.fields
    }

    /// Replaces fixed properties while retaining animation data.
    pub fn set_fields(&mut self, fields: &RibbonFields) {
        self.fields = *fields;
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
