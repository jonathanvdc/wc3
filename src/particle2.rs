//! Particle emitter 2 records in `PRE2` chunks.

use crate::{sized_node, AnimationTrack, Error, Model, Node};

const TAG: [u8; 4] = *b"PRE2";
const FIXED_SIZE: usize = 171;

/// Which particle parts are rendered for each emitted particle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Particle2Frames {
    Head,
    Tail,
    Both,
    Unknown(u32),
}

impl Particle2Frames {
    pub const fn from_raw(value: u32) -> Self {
        match value {
            0 => Self::Head,
            1 => Self::Tail,
            2 => Self::Both,
            other => Self::Unknown(other),
        }
    }
    pub const fn raw(self) -> u32 {
        match self {
            Self::Head => 0,
            Self::Tail => 1,
            Self::Both => 2,
            Self::Unknown(value) => value,
        }
    }
}

/// Fixed physical, texture, and color fields of a particle emitter 2.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Particle2Fields {
    pub speed: f32,
    pub variation: f32,
    pub latitude: f32,
    pub gravity: f32,
    pub life_span: f32,
    pub emission_rate: f32,
    pub width: f32,
    pub length: f32,
    pub filter_mode: u32,
    pub rows: u32,
    pub columns: u32,
    /// 0 = head, 1 = tail, 2 = both.
    pub frame_flags: u32,
    pub tail_length: f32,
    pub time: f32,
    pub segment_colors: [[f32; 3]; 3],
    pub alpha: [u8; 3],
    pub particle_scaling: [f32; 3],
    /// Life span, decay, tail, and tail decay UV intervals.
    pub uv_animations: [[u32; 3]; 4],
    pub texture_id: u32,
    pub squirt: u32,
    pub priority_plane: u32,
    pub replaceable_id: u32,
}

impl Particle2Fields {
    /// Decodes the frame mode while retaining unrecognized values.
    pub fn frames(&self) -> Particle2Frames {
        Particle2Frames::from_raw(self.frame_flags)
    }
    /// Sets the frame mode.
    pub fn set_frames(&mut self, frames: Particle2Frames) {
        self.frame_flags = frames.raw();
    }
    /// Reports whether newly emitted particles are animated in one burst.
    pub fn squirt_enabled(&self) -> bool {
        self.squirt != 0
    }
    /// Changes the squirt flag.
    pub fn set_squirt_enabled(&mut self, enabled: bool) {
        self.squirt = u32::from(enabled);
    }
}

/// A particle emitter 2 with an embedded node and optional animation tracks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticleEmitter2 {
    bytes: Vec<u8>,
}

impl ParticleEmitter2 {
    /// Creates an emitter with zeroed fixed fields.
    pub fn new(node: Node) -> Result<Self, Error> {
        Ok(Self {
            bytes: sized_node::new_record(&node, FIXED_SIZE, TAG)?,
        })
    }

    /// Wraps one inclusive-size emitter record.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        sized_node::layout(bytes, TAG, FIXED_SIZE)?;
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    /// Returns the complete record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the embedded node.
    pub fn node(&self) -> Node {
        Node::from_bytes(&self.bytes[4..self.fixed_start()]).expect("validated node")
    }

    /// Decodes the fixed fields.
    pub fn fields(&self) -> Particle2Fields {
        let bytes = &self.bytes[self.fixed_start()..self.track_start()];
        let word = |offset: usize| {
            u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed field"))
        };
        let float = |offset: usize| f32::from_bits(word(offset));
        Particle2Fields {
            speed: float(0),
            variation: float(4),
            latitude: float(8),
            gravity: float(12),
            life_span: float(16),
            emission_rate: float(20),
            width: float(24),
            length: float(28),
            filter_mode: word(32),
            rows: word(36),
            columns: word(40),
            frame_flags: word(44),
            tail_length: float(48),
            time: float(52),
            segment_colors: std::array::from_fn(|segment| {
                std::array::from_fn(|axis| float(56 + (segment * 3 + axis) * 4))
            }),
            alpha: bytes[92..95].try_into().expect("three alpha bytes"),
            particle_scaling: std::array::from_fn(|axis| float(95 + axis * 4)),
            uv_animations: std::array::from_fn(|group| {
                std::array::from_fn(|index| word(107 + (group * 3 + index) * 4))
            }),
            texture_id: word(155),
            squirt: word(159),
            priority_plane: word(163),
            replaceable_id: word(167),
        }
    }

    /// Replaces fixed fields without changing the node or tracks.
    pub fn set_fields(&mut self, fields: &Particle2Fields) {
        let start = self.fixed_start();
        let mut bytes = Vec::with_capacity(FIXED_SIZE);
        for value in [
            fields.speed,
            fields.variation,
            fields.latitude,
            fields.gravity,
            fields.life_span,
            fields.emission_rate,
            fields.width,
            fields.length,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for value in [
            fields.filter_mode,
            fields.rows,
            fields.columns,
            fields.frame_flags,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&fields.tail_length.to_le_bytes());
        bytes.extend_from_slice(&fields.time.to_le_bytes());
        for color in fields.segment_colors {
            for component in color {
                bytes.extend_from_slice(&component.to_le_bytes());
            }
        }
        bytes.extend_from_slice(&fields.alpha);
        for value in fields.particle_scaling {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        for group in fields.uv_animations {
            for value in group {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        for value in [
            fields.texture_id,
            fields.squirt,
            fields.priority_plane,
            fields.replaceable_id,
        ] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        debug_assert_eq!(bytes.len(), FIXED_SIZE);
        self.bytes[start..start + FIXED_SIZE].copy_from_slice(&bytes);
    }

    /// Decodes optional `KP2*` animation tracks.
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

    /// Replaces optional animation tracks and updates the inclusive size.
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
    matches!(
        &tag,
        b"KP2V" | b"KP2E" | b"KP2W" | b"KP2N" | b"KP2S" | b"KP2L" | b"KP2G" | b"KP2R"
    )
}

impl Model {
    /// Decodes all `PRE2` records in file order.
    pub fn particle_emitters2(&self) -> Result<Vec<ParticleEmitter2>, Error> {
        let mut result = Vec::new();
        for chunk in self.chunks().iter().filter(|chunk| chunk.tag == TAG) {
            result.extend(
                sized_node::records(&chunk.data, TAG, FIXED_SIZE)?
                    .into_iter()
                    .map(ParticleEmitter2::from_bytes)
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        Ok(result)
    }

    /// Replaces particle emitter 2 records in the first `PRE2` chunk.
    pub fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]) -> Result<(), Error> {
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
