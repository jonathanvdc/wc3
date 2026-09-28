//! Reforged popcorn particle emitters in `CORN` chunks.
use crate::model::mdl::{is_zero, Span};
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::{mdl, mdx};
use crate::model::{ModelVersion, SupportsReforgedChunks};
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

use std::borrow::Cow;

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
            get = "Node::flags",
            set = "Node::set_flags",
            SortPrimsFarZ = 65536,
            Unshaded = 32768,
            Unfogged = 131072,
            PopcornScaling = 262144
        )
    )]
    node: Node,
    #[mdl(
        animatable = "LifeSpan",
        track = "PopcornTrack::Lifespan",
        default = "one"
    )]
    life_span: f32,
    #[mdl(
        animatable = "EmissionRate",
        track = "PopcornTrack::EmissionRate",
        default = "one"
    )]
    emission_rate: f32,
    #[mdl(animatable = "Speed", track = "PopcornTrack::Speed", default = "one")]
    speed: f32,
    #[mdl(animatable = "Color", track = "PopcornTrack::Color", default = "white")]
    color: Color,
    #[mdl(animatable = "Alpha", track = "PopcornTrack::Alpha", default = "one")]
    alpha: f32,
    #[mdl(property = "ReplaceableId", default, skip_if = "is_zero")]
    replaceable_id: u32,
    #[mdl(property = "Path", default)]
    path: FixedText<PATH_SIZE>,
    #[mdl(property = "AnimVisibilityGuide", default)]
    visibility_guide: FixedText<PATH_SIZE>,
    #[mdl(tracks, channels(Visibility = "PopcornTrack::Visibility"))]
    tracks: Vec<PopcornTrack>,
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
        emitter.set_path(path)?;
        emitter.set_visibility_guide(visibility_guide)?;
        Ok(emitter)
    }

    /// Borrows the shared node.
    pub fn node(&self) -> &Node {
        &self.node
    }
    /// Borrows the shared node for editing.
    pub fn node_mut(&mut self) -> &mut Node {
        &mut self.node
    }
    /// Returns particle lifetime.
    pub fn life_span(&self) -> f32 {
        self.life_span
    }
    /// Sets particle lifetime.
    pub fn set_life_span(&mut self, value: f32) {
        self.life_span = value;
    }
    /// Returns emission rate.
    pub fn emission_rate(&self) -> f32 {
        self.emission_rate
    }
    /// Sets emission rate.
    pub fn set_emission_rate(&mut self, value: f32) {
        self.emission_rate = value;
    }
    /// Returns particle speed.
    pub fn speed(&self) -> f32 {
        self.speed
    }
    /// Sets particle speed.
    pub fn set_speed(&mut self, value: f32) {
        self.speed = value;
    }
    /// Returns RGB particle color.
    pub fn color(&self) -> Color {
        self.color
    }
    /// Sets RGB particle color.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }
    /// Returns base alpha.
    pub fn alpha(&self) -> f32 {
        self.alpha
    }
    /// Sets base alpha.
    pub fn set_alpha(&mut self, value: f32) {
        self.alpha = value;
    }
    /// Returns the replaceable texture ID.
    pub fn replaceable_id(&self) -> u32 {
        self.replaceable_id
    }
    /// Sets the replaceable texture ID.
    pub fn set_replaceable_id(&mut self, id: u32) {
        self.replaceable_id = id;
    }
    /// Returns the model path.
    pub fn path(&self) -> Cow<'_, str> {
        self.path.text()
    }
    /// Sets the model path.
    pub fn set_path(&mut self, path: &str) -> Result<(), ValueError> {
        self.path.set_text(path)
    }
    /// Returns the animation visibility guide path.
    pub fn visibility_guide(&self) -> Cow<'_, str> {
        self.visibility_guide.text()
    }
    /// Sets the animation visibility guide path.
    pub fn set_visibility_guide(&mut self, guide: &str) -> Result<(), ValueError> {
        self.visibility_guide.set_text(guide)
    }
    /// Borrows decoded animation tracks.
    pub fn tracks(&self) -> &[PopcornTrack] {
        &self.tracks
    }
    /// Replaces optional animation tracks.
    pub fn set_tracks(&mut self, tracks: &[PopcornTrack]) {
        self.tracks = tracks.to_vec();
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
    validate_node_kind(&value.node, 0x1000 | (value.node.flags().bits() & 0x78000))
}
