//! Particle emitter 2 records in `PRE2` chunks.
use crate::model::mdl::is_zero;
use crate::model::{mdl, mdx};
use mdl_codec::SegmentColors;
mod mdl_codec;
use crate::model::ModelVersion;
crate::model::animation::track_group! {
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

use crate::model::KnownChunk;
use crate::model::{Color, Vec3};

use crate::model::ParticleEmitters2Chunk;
use crate::model::{Model, Node};

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
#[derive(Clone, Debug, Default, PartialEq, mdx::Read, mdx::Write)]
pub struct Particle2Fields {
    pub speed: f32,
    pub variation: f32,
    pub latitude: f32,
    pub gravity: f32,
    pub life_span: f32,
    pub emission_rate: f32,
    pub length: f32,
    pub width: f32,
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
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = ParticleEmitters2Chunk::TAG))]
#[mdl(block = "ParticleEmitter2", after_read = "Self::finish_mdl", validate_write = "Self::validate_mdl",
    write_order(node, speed, variation, latitude, gravity, life_span, emission_rate, length, width,
        filter, rows, columns, frames, tail_length, time, segments, alpha, particle_scaling,
        life_uv, decay_uv, tail_uv, tail_decay_uv, texture_id, squirt, priority_plane, replaceable_id, tracks),
    virtual_fields(
        #[mdl(flags(Blend = 1, Additive = 2, Modulate = 4, Modulate2x = 8, AlphaKey = 16), get = "Self::mdl_filter", set = "Self::set_mdl_filter")]
        filter: u32,
        #[mdl(flags(Head = 1, Tail = 2, Both = 4), get = "Self::mdl_frames", set = "Self::set_mdl_frames")]
        frames: u32,
        #[mdl(property = "SegmentColor", delegate, get = "Self::mdl_segments", set = "Self::set_mdl_segments")]
        segments: SegmentColors,
        #[mdl(property = "LifeSpanUVAnim", default, get = "Self::mdl_life_uv", set = "Self::set_mdl_life_uv")]
        life_uv: [u32; 3],
        #[mdl(property = "DecayUVAnim", default, get = "Self::mdl_decay_uv", set = "Self::set_mdl_decay_uv")]
        decay_uv: [u32; 3],
        #[mdl(property = "TailUVAnim", default, get = "Self::mdl_tail_uv", set = "Self::set_mdl_tail_uv")]
        tail_uv: [u32; 3],
        #[mdl(property = "TailDecayUVAnim", default, get = "Self::mdl_tail_decay_uv", set = "Self::set_mdl_tail_decay_uv")]
        tail_decay_uv: [u32; 3],
    )
)]
pub struct ParticleEmitter2 {
    #[mdl(
        flatten,
        extra_flags(
            get = "Node::flags",
            set = "Node::set_flags",
            SortPrimsFarZ = 65536,
            LineEmitter = 131072,
            Unfogged = 262144,
            ModelSpace = 524288,
            Unshaded = 32768,
            XYQuad = 1048576
        )
    )]
    node: Node,
    #[mdl(project(
        #[mdl(animatable = "Speed", track = "Particle2Track::Speed", default)] speed: f32,
        #[mdl(animatable = "Variation", track = "Particle2Track::Variation", default)] variation: f32,
        #[mdl(animatable = "Latitude", track = "Particle2Track::Latitude", default)] latitude: f32,
        #[mdl(animatable = "Gravity", track = "Particle2Track::Gravity", default)] gravity: f32,
        #[mdl(property = "LifeSpan", default)] life_span: f32,
        #[mdl(animatable = "EmissionRate", track = "Particle2Track::EmissionRate", default)] emission_rate: f32,
        #[mdl(animatable = "Length", track = "Particle2Track::Length", default)] length: f32,
        #[mdl(animatable = "Width", track = "Particle2Track::Width", default)] width: f32,
        #[mdl(property = "Rows", default)] rows: u32,
        #[mdl(property = "Columns", default)] columns: u32,
        #[mdl(property = "TailLength", default)] tail_length: f32,
        #[mdl(property = "Time", default)] time: f32,
        #[mdl(property = "Alpha", default)] alpha: [u8; 3],
        #[mdl(property = "ParticleScaling", default)] particle_scaling: Vec3,
        #[mdl(property = "TextureID", default)] texture_id: u32,
        #[mdl(property = "Squirt", default, skip_if = "is_zero")] squirt: u32,
        #[mdl(property = "PriorityPlane", default, skip_if = "is_zero")] priority_plane: u32,
        #[mdl(property = "ReplaceableId", default, skip_if = "is_zero")] replaceable_id: u32,
        #[mdl(skip, default)] filter_mode: u32,
        #[mdl(skip, default)] frame_flags: u32,
        #[mdl(skip, default)] segment_colors: [Color; 3],
        #[mdl(skip, default)] uv_animations: [[u32; 3]; 4],
    ))]
    fields: Particle2Fields,
    #[mdl(tracks, channels(Visibility = "Particle2Track::Visibility"))]
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
