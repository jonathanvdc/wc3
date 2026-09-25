//! Particle emitter 2 records in `PRE2` chunks.
use crate::Encoder;
use crate::ValueError;
use crate::{Color, Tag, Vec3};

use crate::Record;
use crate::{AnimationTrack, Error, Model, Node};
use crate::{Cursor, ModelChunk, ParticleEmitters2Chunk};

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
    tracks: Vec<AnimationTrack>,
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
    pub fn tracks(&self) -> &[AnimationTrack] {
        &self.tracks
    }

    /// Replaces optional animation tracks.
    pub fn set_tracks(&mut self, tracks: &[AnimationTrack]) -> Result<(), ValueError> {
        for track in tracks {
            if !is_track(track.tag) {
                return Err(ValueError::InvalidTrackTag {
                    record: ParticleEmitter2::TAG,
                    track: track.tag,
                });
            }
        }
        self.tracks = tracks.to_vec();
        Ok(())
    }
}

fn decode_fields(bytes: &[u8]) -> Particle2Fields {
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

fn encode_fields(fields: &Particle2Fields, bytes: &mut Encoder<'_>) {
    let start = bytes.position();
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
        bytes.write(value);
    }
    for value in [
        fields.filter_mode,
        fields.rows,
        fields.columns,
        fields.frame_flags,
    ] {
        bytes.write(value);
    }
    bytes.write(fields.tail_length);
    bytes.write(fields.time);
    for color in fields.segment_colors {
        for component in color {
            bytes.write(component);
        }
    }
    bytes.write_bytes(&fields.alpha);
    for value in fields.particle_scaling {
        bytes.write(value);
    }
    for group in fields.uv_animations {
        for value in group {
            bytes.write(value);
        }
    }
    for value in [
        fields.texture_id,
        fields.squirt,
        fields.priority_plane,
        fields.replaceable_id,
    ] {
        bytes.write(value);
    }
    debug_assert_eq!(bytes.position() - start, FIXED_SIZE);
}

fn is_track(tag: Tag) -> bool {
    matches!(
        &tag,
        b"KP2V" | b"KP2E" | b"KP2W" | b"KP2N" | b"KP2S" | b"KP2L" | b"KP2G" | b"KP2R"
    )
}

impl Model {
    /// Decodes all `PRE2` records in file order.
    pub fn particle_emitters2(&self) -> Vec<ParticleEmitter2> {
        self.collect_chunk_records::<ParticleEmitters2Chunk>(|chunk| match chunk {
            ModelChunk::ParticleEmitters2(decoded) => Some(decoded),
            _ => None,
        })
    }

    /// Replaces particle emitter 2 records in the first `PRE2` chunk.
    pub fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]) {
        self.replace_chunk(ModelChunk::ParticleEmitters2(ParticleEmitters2Chunk::new(
            emitters.to_vec(),
        )));
    }
}

impl Record for ParticleEmitter2 {
    fn decode_one(source: &mut Cursor<'_>, _version: u32) -> Result<Self, Error> {
        let mut cursor = source.slice_u32_sized()?;
        let node = Node::decode_one(&mut cursor, 0)?;
        let fields = decode_fields(cursor.read_exact(FIXED_SIZE)?);
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
        encode_fields(&self.fields, bytes);
        for track in &self.tracks {
            if !is_track(track.tag) {
                return Err(Error::MalformedRecord {
                    tag: ParticleEmitter2::TAG,
                    offset: bytes.position() - start,
                });
            }
            track.encode_to(bytes)?;
        }
        bytes.finish_sized(marker, ParticleEmitter2::TAG)?;
        Ok(())
    }
}

impl ParticleEmitter2 {
    /// The tag of the chunk containing this record.
    pub const TAG: Tag = *b"PRE2";
}
