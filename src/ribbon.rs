//! Ribbon emitter records in `RIBB` chunks.

use crate::{sized_node, AnimationTrack, Error, Model, Node};

const TAG: [u8; 4] = *b"RIBB";
const FIXED_SIZE: usize = 52;

/// Fixed properties of a ribbon emitter.
#[derive(Clone, Copy, Debug, PartialEq)]
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

/// One inclusive-size ribbon emitter record with its embedded node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RibbonEmitter {
    bytes: Vec<u8>,
}

impl RibbonEmitter {
    /// Creates a ribbon emitter with zeroed fixed properties.
    pub fn new(node: Node) -> Result<Self, Error> {
        Ok(Self {
            bytes: sized_node::new_record(&node, FIXED_SIZE, TAG)?,
        })
    }

    /// Wraps one inclusive-size record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        sized_node::layout(bytes, TAG, FIXED_SIZE)?;
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the entire record, including its size prefix.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the embedded node.
    pub fn node(&self) -> Node {
        Node::from_bytes(&self.bytes[4..self.fixed_start()]).expect("validated node")
    }

    /// Decodes the fixed properties.
    pub fn fields(&self) -> RibbonFields {
        let bytes = &self.bytes[self.fixed_start()..self.track_start()];
        let word = |offset: usize| {
            u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed field"))
        };
        let float = |offset: usize| f32::from_bits(word(offset));
        RibbonFields {
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
        }
    }

    /// Replaces fixed properties while retaining animation data.
    pub fn set_fields(&mut self, fields: &RibbonFields) {
        let values = [
            fields.height_above.to_bits(),
            fields.height_below.to_bits(),
            fields.alpha.to_bits(),
            fields.color[0].to_bits(),
            fields.color[1].to_bits(),
            fields.color[2].to_bits(),
            fields.life_span.to_bits(),
            fields.texture_slot,
            fields.emission_rate,
            fields.rows,
            fields.columns,
            fields.material_id,
            fields.gravity.to_bits(),
        ];
        let start = self.fixed_start();
        for (index, value) in values.iter().enumerate() {
            self.bytes[start + index * 4..start + index * 4 + 4]
                .copy_from_slice(&value.to_le_bytes());
        }
    }

    /// Decodes optional ribbon animation tracks.
    pub fn tracks(&self) -> Result<Vec<AnimationTrack>, Error> {
        let mut result = Vec::new();
        let mut offset = self.track_start();
        while offset < self.bytes.len() {
            let (track, consumed) = AnimationTrack::parse(&self.bytes, offset)?;
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord { tag: TAG, offset });
            }
            result.push(track);
            offset += consumed;
        }
        Ok(result)
    }

    /// Replaces optional ribbon animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), Error> {
        let mut bytes = self.bytes[..self.track_start()].to_vec();
        for track in tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: TAG,
                    offset: bytes.len(),
                });
            }
            bytes.extend_from_slice(&track.to_bytes()?);
        }
        if bytes.len() > u32::MAX as usize {
            return Err(Error::ChunkTooLarge {
                tag: TAG,
                size: bytes.len(),
            });
        }
        let size = bytes.len() as u32;
        bytes[..4].copy_from_slice(&size.to_le_bytes());
        self.bytes = bytes;
        Ok(())
    }

    fn fixed_start(&self) -> usize {
        sized_node::layout(&self.bytes, TAG, FIXED_SIZE)
            .expect("validated record")
            .fixed_start
    }
    fn track_start(&self) -> usize {
        sized_node::layout(&self.bytes, TAG, FIXED_SIZE)
            .expect("validated record")
            .track_start
    }
}

fn is_track(tag: [u8; 4]) -> bool {
    matches!(&tag, b"KRVS" | b"KRHA" | b"KRHB" | b"KRAL" | b"KRTX")
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
            sum.checked_add(emitter.bytes.len())
                .filter(|&size| size <= u32::MAX as usize)
                .ok_or(Error::ChunkTooLarge {
                    tag: TAG,
                    size: usize::MAX,
                })
        })?;
        let mut data = Vec::with_capacity(size);
        for emitter in emitters {
            data.extend_from_slice(emitter.as_bytes());
        }
        self.replace_chunks(TAG, data);
        Ok(())
    }
}
