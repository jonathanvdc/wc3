//! Classic particle emitters stored in `PREM` chunks.
use crate::model::mdl::Span;
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::ModelVersion;
use crate::model::{mdl, mdx};
crate::model::animation::track_group! {
    pub enum ParticleTrack {
        Visibility: ParticleVisibility,
        EmissionRate: ParticleEmissionRate,
        Gravity: ParticleGravity,
        Longitude: ParticleLongitude,
        Latitude: ParticleLatitude,
        Lifespan: ParticleLifespan,
        Speed: ParticleSpeed,
    }
}

use crate::model::KnownChunk;
use crate::model::ValueError;

use crate::model::ParticleEmittersChunk;

use std::borrow::Cow;

use crate::model::FixedText;
use crate::model::{Model, Node};

const PATH_SIZE: usize = 260;

/// A Classic particle emitter with optional animated properties.
///
/// MDL reading restores the particle object-kind bit and resource flags. Writing
/// requires matching node bits and default hidden bases.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write)]
#[mdx(sized(tag = ParticleEmittersChunk::TAG))]
#[mdl(
    block = "ParticleEmitter",
    after_read = "finish_particle",
    validate_write = "validate_particle"
)]
pub struct ParticleEmitter {
    #[mdl(
        flatten,
        extra_flags(
            get = "Node::flags",
            set = "Node::set_flags",
            EmitterUsesMdl = 32768,
            EmitterUsesTga = 65536
        )
    )]
    node: Node,
    #[mdl(
        animatable = "EmissionRate",
        track = "ParticleTrack::EmissionRate",
        default
    )]
    emission_rate: f32,
    #[mdl(animatable = "Gravity", track = "ParticleTrack::Gravity", default)]
    gravity: f32,
    #[mdl(animatable = "Longitude", track = "ParticleTrack::Longitude", default)]
    longitude: f32,
    #[mdl(animatable = "Latitude", track = "ParticleTrack::Latitude", default)]
    latitude: f32,
    #[mdl(property = "Path", default)]
    path: FixedText<PATH_SIZE>,
    #[mdl(animatable = "LifeSpan", track = "ParticleTrack::Lifespan", default)]
    life_span: f32,
    #[mdl(animatable = "InitVelocity", track = "ParticleTrack::Speed", default)]
    initial_velocity: f32,
    #[mdl(tracks, channels(Visibility = "ParticleTrack::Visibility"))]
    tracks: Vec<ParticleTrack>,
}

impl ParticleEmitter {
    /// Creates an emitter with zeroed physical values.
    pub fn new(node: Node, path: &str) -> Result<Self, ValueError> {
        let mut emitter = Self {
            node,
            emission_rate: 0.0,
            gravity: 0.0,
            longitude: 0.0,
            latitude: 0.0,
            path: FixedText::default(),
            life_span: 0.0,
            initial_velocity: 0.0,
            tracks: Vec::new(),
        };
        emitter.set_path(path)?;
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

    /// Returns emission rate.
    pub fn emission_rate(&self) -> f32 {
        self.emission_rate
    }
    /// Sets emission rate.
    pub fn set_emission_rate(&mut self, value: f32) {
        self.emission_rate = value;
    }
    /// Returns gravity.
    pub fn gravity(&self) -> f32 {
        self.gravity
    }
    /// Sets gravity.
    pub fn set_gravity(&mut self, value: f32) {
        self.gravity = value;
    }
    /// Returns longitude.
    pub fn longitude(&self) -> f32 {
        self.longitude
    }
    /// Sets longitude.
    pub fn set_longitude(&mut self, value: f32) {
        self.longitude = value;
    }
    /// Returns latitude.
    pub fn latitude(&self) -> f32 {
        self.latitude
    }
    /// Sets latitude.
    pub fn set_latitude(&mut self, value: f32) {
        self.latitude = value;
    }
    /// Returns particle lifetime.
    pub fn life_span(&self) -> f32 {
        self.life_span
    }
    /// Sets particle lifetime.
    pub fn set_life_span(&mut self, value: f32) {
        self.life_span = value;
    }
    /// Returns initial velocity.
    pub fn initial_velocity(&self) -> f32 {
        self.initial_velocity
    }
    /// Sets initial velocity.
    pub fn set_initial_velocity(&mut self, value: f32) {
        self.initial_velocity = value;
    }

    /// Returns the emitter resource path up to the first NUL.
    pub fn path(&self) -> Cow<'_, str> {
        self.path.text()
    }

    /// Sets the emitter path while retaining all other fields.
    pub fn set_path(&mut self, path: &str) -> Result<(), ValueError> {
        self.path.set_text(path)
    }

    /// Borrows decoded animation tracks.
    pub fn tracks(&self) -> &[ParticleTrack] {
        &self.tracks
    }

    /// Replaces optional animation tracks.
    pub fn set_tracks(&mut self, tracks: &[ParticleTrack]) {
        self.tracks = tracks.to_vec();
    }
}

impl<V: ModelVersion> Model<V> {
    /// Decodes all `PREM` records in file order.
    pub fn particle_emitters(&self) -> Vec<ParticleEmitter> {
        self.collect_chunk_records::<ParticleEmittersChunk>()
    }

    /// Replaces particle emitters in the first `PREM` chunk.
    pub fn set_particle_emitters(&mut self, emitters: &[ParticleEmitter]) {
        self.replace_chunk(ParticleEmittersChunk::new(emitters.to_vec()));
    }
}

fn finish_particle(value: &mut ParticleEmitter, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x1000);
    Ok(())
}
fn validate_particle(value: &ParticleEmitter) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x1000 | (value.node.flags().bits() & 0x18000))
}
