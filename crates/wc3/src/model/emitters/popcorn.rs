//! Reforged popcorn particle emitters in `CORN` chunks.
use crate::model::mdl::{is_zero, Span};
use crate::model::scene::NodeFlags;
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::{mdl, mdx};
use crate::model::{ModelVersion, SupportsReforgedChunks};
use bitfield::bitfield;
crate::model::animation::track_group! {
    pub enum PopcornTrack {
        Alpha: PopcornAlpha,
        Color: PopcornColor,
        EmissionRate: PopcornEmissionRate,
        Lifespan: PopcornLifespan,
        Speed: PopcornSpeed,
        Visibility: PopcornVisibility,
    }
}

use crate::model::Color;
use crate::model::KnownChunk;
use crate::model::ValueError;

use crate::model::PopcornEmittersChunk;

use crate::model::FixedText;
use crate::model::{Model, Node};

const PATH_SIZE: usize = 260;

/// A popcorn particle emitter with decoded fields and animation tracks.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = PopcornEmittersChunk::TAG))]
#[mdl(
    block = "ParticleEmitterPopcorn",
    after_read = "finish_popcorn",
    validate_write = "validate_popcorn"
)]
pub struct PopcornEmitter {
    #[mdl(
        flatten,
        extra_flags(
            get = "Node::mdl_flags",
            set = "Node::set_mdl_flags",
            SortPrimsFarZ = 65536,
            Unshaded = 32768,
            Unfogged = 131072,
            PopcornScaling = 262144
        )
    )]
    pub node: Node,
    #[mdl(
        animatable = "LifeSpan",
        track = "PopcornTrack::Lifespan",
        default = "one"
    )]
    pub life_span: f32,
    #[mdl(
        animatable = "EmissionRate",
        track = "PopcornTrack::EmissionRate",
        default = "one"
    )]
    pub emission_rate: f32,
    #[mdl(animatable = "Speed", track = "PopcornTrack::Speed", default = "one")]
    pub speed: f32,
    #[mdl(animatable = "Color", track = "PopcornTrack::Color", default = "white")]
    pub color: Color,
    #[mdl(animatable = "Alpha", track = "PopcornTrack::Alpha", default = "one")]
    pub alpha: f32,
    #[mdl(property = "ReplaceableId", default, skip_if = "is_zero")]
    pub replaceable_id: u32,
    #[mdl(property = "Path", default)]
    pub path: FixedText<PATH_SIZE>,
    #[mdl(property = "AnimVisibilityGuide", default)]
    pub visibility_guide: FixedText<PATH_SIZE>,
    #[mdl(tracks, channels(Visibility = "PopcornTrack::Visibility"))]
    pub tracks: Vec<PopcornTrack>,
}

impl PopcornEmitter {
    /// Creates an emitter with unit lifespan, emission rate, speed, alpha, and white color.
    pub fn new(node: Node, path: &str, visibility_guide: &str) -> Result<Self, ValueError> {
        let mut emitter = Self {
            node,
            life_span: 1.0,
            emission_rate: 1.0,
            speed: 1.0,
            color: [1.0; 3],
            alpha: 1.0,
            replaceable_id: 0,
            path: FixedText::default(),
            visibility_guide: FixedText::default(),
            tracks: Vec::new(),
        };
        emitter.path.set_text(path)?;
        emitter.visibility_guide.set_text(visibility_guide)?;
        Ok(emitter)
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns an error if this model version does not support `CORN`.
    pub fn try_popcorn_emitters(&self) -> Result<Vec<PopcornEmitter>, ValueError> {
        self.check_chunk_version(*b"CORN")?;
        Ok(self.collect_chunk_records::<PopcornEmittersChunk>())
    }

    /// Returns an error if this model version does not support `CORN`.
    pub fn try_set_popcorn_emitters(
        &mut self,
        emitters: &[PopcornEmitter],
    ) -> Result<(), ValueError> {
        self.check_chunk_version(*b"CORN")?;
        self.replace_chunk(PopcornEmittersChunk::new(emitters.to_vec()));
        Ok(())
    }
}

impl<V: SupportsReforgedChunks> Model<V> {
    /// Returns decoded `CORN` records.
    pub fn popcorn_emitters(&self) -> Vec<PopcornEmitter> {
        self.collect_chunk_records::<PopcornEmittersChunk>()
    }

    /// Replaces `CORN` records.
    pub fn set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter]) {
        self.replace_chunk(PopcornEmittersChunk::new(emitters.to_vec()));
    }
}

fn one() -> f32 {
    1.0
}
fn white() -> Color {
    [1.0; 3]
}
fn finish_popcorn(value: &mut PopcornEmitter, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x1000);
    Ok(())
}
fn validate_popcorn(value: &PopcornEmitter) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x1000 | (value.node.flags.bits() & 0x78000))
}

bitfield! {
    /// Behavioral flags interpreted in this emitter's context.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub struct PopcornFlags(u32);
    /// Returns the stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes `unshaded`.
    pub unshaded, set_unshaded: 15;
    /// Returns or changes `sort_prims_far_z`.
    pub sort_prims_far_z, set_sort_prims_far_z: 16;
    /// Returns or changes `unfogged`.
    pub unfogged, set_unfogged: 17;
    /// Returns or changes `popcorn_scaling`.
    pub popcorn_scaling, set_popcorn_scaling: 18;
}
impl PopcornFlags {
    const MASK: u32 = 0x78000;
    /// Extracts this emitter's behavioral bits from a node word.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits & Self::MASK)
    }
}
impl PopcornEmitter {
    /// Returns behavioral flags using this emitter's bit meanings.
    pub fn flags(&self) -> PopcornFlags {
        PopcornFlags::from_bits(self.node.flags.bits())
    }
    /// Changes emitter behavior while preserving every unrelated node bit.
    pub fn set_flags(&mut self, flags: PopcornFlags) {
        let bits =
            (self.node.flags.bits() & !PopcornFlags::MASK) | (flags.bits() & PopcornFlags::MASK);
        self.node.flags = NodeFlags(bits);
    }
}
