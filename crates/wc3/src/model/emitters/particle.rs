//! Classic particle emitters stored in `PREM` chunks.
use crate::model::mdx;
use crate::model::ModelVersion;
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

const PATH_SIZE: usize = 256;

/// A Classic particle emitter with optional animated properties.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write)]
#[mdx(sized(tag = ParticleEmittersChunk::TAG))]
pub struct ParticleEmitter {
    node: Node,
    emission_rate: f32,
    gravity: f32,
    longitude: f32,
    latitude: f32,
    path: FixedText<PATH_SIZE>,
    reserved: u32,
    life_span: f32,
    initial_velocity: f32,
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
            reserved: 0,
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

    /// Returns the untyped reserved word following the path.
    pub fn reserved(&self) -> u32 {
        self.reserved
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
