//! Particle emitter 2 records in `PRE2` chunks.
use crate::ModelVersion;
crate::animation::track_group! {
    pub enum Particle2Track {
        Visibility: Particle2Visibility,
        EmissionRate: Particle2EmissionRate,
        Width: Particle2Width,
        Length: Particle2Length,
        Speed: Particle2Speed,
        Latitude: Particle2Latitude,
        Gravity: Particle2Gravity,
        Variation: Particle2Variation,
    }
}

use crate::EncodeError;
use crate::Encoder;
use crate::KnownChunk;
use crate::{Color, Vec3};

use crate::{Cursor, ParticleEmitters2Chunk};
use crate::{Decodable, Encodable, Readable, Writable};
use crate::{DecodeError, Model, Node};

pub(crate) const FIXED_SIZE: usize = 171;

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
#[derive(Clone, Debug, Default, PartialEq, Readable, Writable)]
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
    pub segment_colors: [Color; 3],
    pub alpha: [u8; 3],
    pub particle_scaling: Vec3,
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

/// A particle emitter 2 with decoded fields and animation tracks.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleEmitter2 {
    node: Node,
    fields: Particle2Fields,
    tracks: Vec<Particle2Track>,
}

impl ParticleEmitter2 {
    /// Creates an emitter with zeroed fixed fields.
    pub fn new(node: Node) -> Self {
        Self {
            node,
            fields: Particle2Fields::default(),
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

    /// Returns a copy of the fixed fields.
    pub fn fields(&self) -> Particle2Fields {
        self.fields.clone()
    }

    /// Borrows the fixed fields for editing.
    pub fn fields_mut(&mut self) -> &mut Particle2Fields {
        &mut self.fields
    }

    /// Replaces fixed fields without changing the node or tracks.
    pub fn set_fields(&mut self, fields: &Particle2Fields) {
        self.fields = fields.clone();
    }

    /// Borrows decoded optional animation tracks.
    pub fn tracks(&self) -> &[Particle2Track] {
        &self.tracks
    }

    /// Replaces optional animation tracks.
    pub fn set_tracks(&mut self, tracks: &[Particle2Track]) {
        self.tracks = tracks.to_vec();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all `PRE2` records in file order.
    pub fn particle_emitters2(&self) -> Vec<ParticleEmitter2> {
        self.collect_chunk_records::<ParticleEmitters2Chunk>()
    }

    /// Replaces particle emitter 2 records in the first `PRE2` chunk.
    pub fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]) {
        self.replace_chunk(ParticleEmitters2Chunk::new(emitters.to_vec()));
    }
}

impl Decodable for ParticleEmitter2 {
    fn decode_one(source: &mut Cursor<'_>) -> Result<Self, DecodeError> {
        let mut cursor = source.slice_u32_sized()?;
        let node = Node::decode_one(&mut cursor)?;
        let mut fixed = cursor.slice(FIXED_SIZE)?;
        let fields = fixed.read()?;
        fixed.finish()?;
        let mut tracks = Vec::new();
        while !cursor.remaining().is_empty() {
            tracks.push(cursor.read::<Particle2Track>()?);
        }
        cursor.finish()?;
        Ok(Self {
            node,
            fields,
            tracks,
        })
    }
}

impl Encodable for ParticleEmitter2 {
    fn encode_to(&self, bytes: &mut Encoder<'_>) -> Result<(), EncodeError> {
        let marker = bytes.begin_sized();
        self.node.encode_to(bytes)?;
        bytes.write(&self.fields);
        for track in &self.tracks {
            bytes.write(track);
        }
        bytes.finish_sized(marker, ParticleEmitters2Chunk::TAG)?;
        Ok(())
    }
}
