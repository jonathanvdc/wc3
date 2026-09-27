//! Ribbon emitter records in `RIBB` chunks.
use crate::ModelVersion;
crate::animation::track_group! {
    pub enum RibbonTrack {
        Visibility: RibbonVisibility,
        HeightAbove: RibbonHeightAbove,
        HeightBelow: RibbonHeightBelow,
        Alpha: RibbonAlpha,
        Color: RibbonColor,
        TextureSlot: RibbonTextureSlot,
    }
}

use crate::Color;
use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;

use crate::{Cursor, RibbonEmittersChunk};
use crate::{DecodeError, Model, Node};
use crate::{Readable, Writable};

pub(crate) const FIXED_SIZE: usize = 52;

/// Fixed properties of a ribbon emitter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Readable, Writable)]
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
#[derive(Clone, Debug, PartialEq)]
pub struct RibbonEmitter {
    node: Node,
    fields: RibbonFields,
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

impl Readable for RibbonEmitter {
    fn read_from(source: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let mut cursor = source.slice_u32_sized()?;
        let node = cursor.read()?;
        let mut fixed = cursor.slice(FIXED_SIZE)?;
        let fields = fixed.read()?;
        fixed.finish()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            tracks.push(cursor.read::<RibbonTrack>()?);
        }
        cursor.finish()?;
        Ok(Self {
            node,
            fields,
            tracks,
        })
    }
}

impl Writable for &RibbonEmitter {
    fn write_to(self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let marker = bytes.begin_sized();
        self.node.write_to(bytes)?;
        bytes.write(&self.fields)?;
        for track in &self.tracks {
            bytes.write(track)?;
        }
        bytes.finish_sized(marker, RibbonEmittersChunk::TAG)?;
        Ok(())
    }
}
