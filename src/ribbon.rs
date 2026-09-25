//! Ribbon emitter records in `RIBB` chunks.

use crate::{sized_node, AnimationTrack, Error, Model, Node};

const TAG: [u8; 4] = *b"RIBB";
const FIXED_SIZE: usize = 52;

/// Fixed properties of a ribbon emitter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RibbonFields {
    pub height_above: f32,
    pub height_below: f32,
    pub alpha: f32,
    pub color: [f32; 3],
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
    pub fn new(node: Node) -> Result<Self, Error> {
        let emitter = Self {
            node,
            fields: RibbonFields::default(),
            tracks: Vec::new(),
        };
        emitter.to_bytes()?;
        Ok(emitter)
    }

    /// Parses one inclusive-size record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let layout = sized_node::layout(bytes, TAG, FIXED_SIZE)?;
        let node = Node::from_bytes(&bytes[4..layout.fixed_start])?;
        let fixed = &bytes[layout.fixed_start..layout.track_start];
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
        let mut offset = layout.track_start;
        while offset < bytes.len() {
            let (track, consumed) = AnimationTrack::parse(bytes, offset)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord { tag: TAG, offset });
            }
            tracks.push(track);
            offset += consumed;
        }
        Ok(Self {
            node,
            fields,
            tracks,
        })
    }

    /// Serializes the inclusive-size record.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut bytes = vec![0; 4];
        bytes.extend_from_slice(&self.node.to_bytes());
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
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [
            fields.texture_slot,
            fields.emission_rate,
            fields.rows,
            fields.columns,
            fields.material_id,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&fields.gravity.to_le_bytes());
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        let size = u32::try_from(bytes.len()).map_err(|_| Error::ChunkTooLarge {
            tag: TAG,
            size: bytes.len(),
        })?;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        Ok(bytes)
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
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        for track in tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: 0,
                });
            }
            track.to_bytes()?;
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn is_track(tag: [u8; 4]) -> bool {
    matches!(
        &tag,
        b"KRVS" | b"KRHA" | b"KRHB" | b"KRAL" | b"KRCO" | b"KRTX"
    )
}

impl Model {
    /// Decodes all ribbon emitter records in file order.
    pub fn ribbon_emitters(&self) -> Result<Vec<RibbonEmitter>, Error> {
        let mut result = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            result.extend(
                sized_node::records(&chunk.data, TAG, FIXED_SIZE)?
                    .into_iter()
                    .map(RibbonEmitter::from_bytes)
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        Ok(result)
    }

    /// Replaces ribbon emitters in the first `RIBB` chunk.
    pub fn set_ribbon_emitters(&mut self, emitters: &[RibbonEmitter]) -> Result<(), Error> {
        let size = emitters.iter().try_fold(0usize, |sum, emitter| {
            sum.checked_add(emitter.to_bytes()?.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for emitter in emitters {
            data.extend_from_slice(&emitter.to_bytes()?);
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
