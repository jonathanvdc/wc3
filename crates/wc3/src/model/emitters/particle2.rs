//! Textured particles with animated emission and head/tail rendering.
use crate::model::mdl::is_zero;
use crate::model::scene::{impl_node_flags, NodeFlagInterpretation};
use crate::model::{mdl, mdx};
use crate::model::{Animatable, Track};
use bitfield::bitfield;
use mdl_codec::SegmentColors;
mod mdl_codec;
use crate::model::KnownChunk;
use crate::model::ModelVersion;
use crate::model::ParticleEmitters2Chunk;
use crate::model::{Color, Vec3};
use crate::model::{Model, Node};

/// Which parts of each particle are rendered: head, tail, or both.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    mdx::Read,
    mdx::Write,
    mdx::Value,
    mdl::Read,
    mdl::Write,
)]
#[mdx(value = u32)]
#[mdl(choice, default)]
pub enum Particle2Frames {
    #[default]
    #[mdx(value = 0)]
    /// Renders particle heads only.
    Head,
    #[mdx(value = 1)]
    /// Renders particle tails only.
    Tail,
    #[mdx(value = 2)]
    /// Renders both heads and tails.
    Both,
    #[mdx(unknown)]
    #[mdl(unknown)]
    /// An unrecognized wire value preserved by MDX; unsupported in MDL.
    Unknown(u32),
}

/// How particle colors blend with the scene. Unknown values round-trip in MDX.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Hash,
    mdx::Read,
    mdx::Write,
    mdx::Value,
    mdl::Read,
    mdl::Write,
)]
#[mdx(value = u32)]
#[mdl(choice, default)]
pub enum Particle2FilterMode {
    #[default]
    #[mdx(value = 0)]
    /// Source-alpha blending.
    Blend,
    #[mdx(value = 1)]
    /// Additive blending.
    Additive,
    #[mdx(value = 2)]
    /// Multiplicative blending.
    Modulate,
    #[mdx(value = 3)]
    /// Multiplicative blending with doubled contribution.
    Modulate2x,
    #[mdx(value = 4)]
    /// Alpha-tested rendering.
    AlphaKey,
    #[mdx(unknown)]
    #[mdl(unknown)]
    /// An unrecognized wire value preserved by MDX; unsupported in MDL.
    Unknown(u32),
}

/// An emitter of textured particle heads and tails.
///
/// Rows and columns divide the texture into animation cells. Color, alpha, and
/// scaling arrays describe three stages of each particle lifetime.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = ParticleEmitters2Chunk::TAG))]
#[mdl(block = "ParticleEmitter2", after_read = "Self::finish_mdl", validate_write = "Self::validate_mdl",
    write_order(node, speed, variation, latitude, gravity, life_span, emission_rate, length, width,
        filter_mode, rows, columns, frames, tail_length, time, segments, alpha, particle_scaling,
        life_uv, decay_uv, tail_uv, tail_decay_uv, texture_id, squirt, priority_plane, replaceable_id, visibility),
    virtual_fields(
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
    #[mdl(flatten)]
    /// Embedded node.
    pub node: Node<Particle2Flags>,
    #[mdx(tag = *b"KP2S")]
    #[mdl(property = "Speed", default)]
    /// Initial particle speed in model units per second.
    pub speed: Animatable<f32>,
    #[mdx(tag = *b"KP2R")]
    #[mdl(property = "Variation", default)]
    /// Authored variation factor for initial particle speed.
    pub variation: Animatable<f32>,
    #[mdx(tag = *b"KP2L")]
    #[mdl(property = "Latitude", default)]
    /// Emission-cone latitude in degrees.
    pub latitude: Animatable<f32>,
    #[mdx(tag = *b"KP2G")]
    #[mdl(property = "Gravity", default)]
    /// Acceleration due to gravity in model units per second squared.
    pub gravity: Animatable<f32>,
    #[mdl(property = "LifeSpan", default)]
    /// Particle lifetime in seconds.
    pub life_span: f32,
    #[mdx(tag = *b"KP2E")]
    #[mdl(property = "EmissionRate", default)]
    /// Emission rate in particles per second, or burst count when squirt is enabled.
    pub emission_rate: Animatable<f32>,
    #[mdx(tag = *b"KP2N")]
    #[mdl(property = "Length", default)]
    /// Length of the emission region in model units.
    pub length: Animatable<f32>,
    #[mdx(tag = *b"KP2W")]
    #[mdl(property = "Width", default)]
    /// Width of the emission region in model units.
    pub width: Animatable<f32>,
    #[mdl(flatten)]
    /// How particles blend with the scene.
    pub filter_mode: Particle2FilterMode,
    #[mdl(property = "Rows", default)]
    /// Number of rows in the particle texture atlas.
    pub rows: u32,
    #[mdl(property = "Columns", default)]
    /// Number of columns in the particle texture atlas.
    pub columns: u32,
    /// Which particle parts are rendered.
    #[mdl(flatten)]
    pub frames: Particle2Frames,
    #[mdl(property = "TailLength", default)]
    /// Time span represented by a particle tail, in seconds.
    pub tail_length: f32,
    #[mdl(property = "Time", default)]
    /// Fraction of particle lifetime at the middle color, alpha, and scale stage.
    pub time: f32,
    #[mdl(skip, default)]
    /// RGB color at the start, middle, and end of particle life.
    pub segment_colors: [Color; 3],
    #[mdl(property = "Alpha", default)]
    /// Opacity from 0 to 255 at the three lifetime stages.
    pub alpha: [u8; 3],
    #[mdl(property = "ParticleScaling", default)]
    /// Scale at the start, middle, and end of particle life.
    pub particle_scaling: Vec3,
    /// Life span, decay, tail, and tail decay UV intervals.
    #[mdl(skip, default)]
    pub uv_animations: [[u32; 3]; 4],
    #[mdl(property = "TextureID", default)]
    /// Index into the model texture collection.
    pub texture_id: u32,
    #[mdl(property = "Squirt", default, skip_if = "is_zero")]
    /// Nonzero enables burst emission at emission-rate keys.
    pub squirt: u32,
    #[mdl(property = "PriorityPlane", default, skip_if = "is_zero")]
    /// Authored rendering priority plane.
    pub priority_plane: u32,
    #[mdl(property = "ReplaceableId", default, skip_if = "is_zero")]
    /// Replaceable-texture identifier; zero uses the texture collection entry.
    pub replaceable_id: u32,
    #[mdx(tag = *b"KP2V")]
    #[mdl(property = "Visibility")]
    /// Optional visibility animation.
    pub visibility: Option<Track<f32>>,
}

impl ParticleEmitter2 {
    /// Reports whether newly emitted particles are animated in one burst.
    pub fn squirt_enabled(&self) -> bool {
        self.squirt != 0
    }
    /// Changes the squirt flag.
    pub fn set_squirt_enabled(&mut self, enabled: bool) {
        self.squirt = u32::from(enabled);
    }

    /// Creates an emitter with zeroed fixed fields.
    pub fn new<F: NodeFlagInterpretation>(node: Node<F>) -> Self {
        Self {
            node: node.cast_flags(),
            speed: Default::default(),
            variation: Default::default(),
            latitude: Default::default(),
            gravity: Default::default(),
            life_span: Default::default(),
            emission_rate: Default::default(),
            length: Default::default(),
            width: Default::default(),
            filter_mode: Default::default(),
            rows: Default::default(),
            columns: Default::default(),
            frames: Default::default(),
            tail_length: Default::default(),
            time: Default::default(),
            segment_colors: Default::default(),
            alpha: Default::default(),
            particle_scaling: Default::default(),
            uv_animations: Default::default(),
            texture_id: Default::default(),
            squirt: Default::default(),
            priority_plane: Default::default(),
            replaceable_id: Default::default(),
            visibility: None,
        }
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of `PRE2` records in file order.
    pub fn particle_emitters2(&self) -> Vec<ParticleEmitter2> {
        self.collect_chunk_records::<ParticleEmitters2Chunk>()
    }

    /// Replaces particle emitter 2 records with one `PRE2` chunk, removing any duplicate chunks.
    pub fn set_particle_emitters2(&mut self, emitters: &[ParticleEmitter2]) {
        self.replace_chunk(ParticleEmitters2Chunk::new(emitters.to_vec()));
    }
}

bitfield! {
    /// Complete node flag word interpreted in this emitter's context.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
    #[mdl(
        flags(
            DontInheritTranslation = 1,
            DontInheritRotation = 2,
            DontInheritScaling = 4,
            Billboarded = 8,
            BillboardedLockX = 16,
            BillboardedLockY = 32,
            BillboardedLockZ = 64,
            CameraAnchored = 128,
            SortPrimsFarZ = 65536,
            LineEmitter = 131072,
            Unfogged = 262144,
            ModelSpace = 524288,
            Unshaded = 32768,
            XYQuad = 1048576,
        ),
        allow_bits = 0x1000
    )]
    pub struct Particle2Flags(u32);
    /// Returns the stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes `unshaded`.
    pub unshaded, set_unshaded: 15;
    /// Returns or changes `sort_prims_far_z`.
    pub sort_prims_far_z, set_sort_prims_far_z: 16;
    /// Returns or changes `line_emitter`.
    pub line_emitter, set_line_emitter: 17;
    /// Returns or changes `unfogged`.
    pub unfogged, set_unfogged: 18;
    /// Returns or changes `model_space`.
    pub model_space, set_model_space: 19;
    /// Returns or changes `xy_quad`.
    pub xy_quad, set_xy_quad: 20;
}
impl Particle2Flags {
    const MASK: u32 = 0x1f8000;
    /// Interprets a node word, preserving every stored bit.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
}
impl ParticleEmitter2 {
    /// Returns behavioral flags using this emitter's bit meanings.
    pub fn flags(&self) -> Particle2Flags {
        Particle2Flags::from_bits(self.node.flags.bits() & Particle2Flags::MASK)
    }
    /// Changes emitter behavior while preserving every unrelated node bit.
    pub fn set_flags(&mut self, flags: Particle2Flags) {
        let bits = (self.node.flags.bits() & !Particle2Flags::MASK)
            | (flags.bits() & Particle2Flags::MASK);
        self.node.flags = Particle2Flags(bits);
    }
}

impl_node_flags!(Particle2Flags);
