//! Ribbon emitter records in `RIBB` chunks.
use crate::Encoder;
use crate::ValueError;
use crate::{Color, Tag};

use crate::Record;
use crate::{AnimationTrack, Error, Model, Node};
use crate::{Cursor, ModelChunk, RibbonEmittersChunk};

pub(crate) const FIXED_SIZE: usize = 52;

/// Fixed properties of a ribbon emitter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
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
    tracks: Vec<AnimationTrack>,
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
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }

    /// Replaces optional ribbon animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), ValueError> {
        for track in tracks {
            if !is_track(track.tag) {
                return Err(ValueError::InvalidTrackTag {
                    record: RibbonEmitter::TAG,
                    track: track.tag,
                });
            }
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track(tag: Tag) -> bool {
    matches!(
        &tag,
        b"KRVS" | b"KRHA" | b"KRHB" | b"KRAL" | b"KRCO" | b"KRTX"
    )
}

impl Model {
    /// Decodes all ribbon emitter records in file order.
    pub fn ribbon_emitters(&self) -> Result<Vec<RibbonEmitter>, Error> {
        self.collect_chunk_records::<RibbonEmittersChunk>(|chunk| match chunk {
            ModelChunk::RibbonEmitters(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces ribbon emitters in the first `RIBB` chunk.
    pub fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter]) {
        self.replace_chunk(ModelChunk::RibbonEmitters(RibbonEmittersChunk::new(
            emitters.to_vec(),
        )));
    }
}

impl Record for RibbonEmitter {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let node = Node::decode_one(&mut cursor, 0)?;
        let fixed = cursor.read_exact(FIXED_SIZE)?;
        let word = |offset: usize| {
            u32::from_le_bytes(fixed[offset..offset + 4].try_into().expect("fixed field"))
        };
        let float = |offset: usize| f32::from_bits(word(offset));
        let fields = RibbonFields {
            height_above: float(0),
            height_below: float(4),
            alpha: float(8),
            color: [float(12), float(16), float(20)],
            life_span: float(24),
            texture_slot: word(28),
            emission_rate: word(32),
            rows: word(36),
            columns: word(40),
            material_id: word(44),
            gravity: float(48),
        };
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            let offset = cursor.absolute_position();
            let track = AnimationTrack::decode_one(&mut cursor, 0)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: Self::TAG,
                    offset,
                });
            }

            tracks.push(track);
        }
        cursor.finish()?;
        Ok(Self {
            node,
            fields,
            tracks,
        })
    }

    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), Error> {
        let start = bytes.position();
        let marker = bytes.begin_sized();
        self.node.encode_to(bytes)?;
        let fields = &self.fields;
        for value in [
            fields.height_above,
            fields.height_below,
            fields.alpha,
            fields.color[0],
            fields.color[1],
            fields.color[2],
            fields.life_span,
        ] {
            bytes.write(value);
        }
        for value in [
            fields.texture_slot,
            fields.emission_rate,
            fields.rows,
            fields.columns,
            fields.material_id,
        ] {
            bytes.write(value);
        }
        bytes.write(fields.gravity);
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: RibbonEmitter::TAG,
                    offset: bytes.position() - start,
                });
            }
            track.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, RibbonEmitter::TAG)?;
        Ok(())
    }
}

impl RibbonEmitter {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"RIBB";
}
