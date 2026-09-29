//! Particles that use model or image resources.
use crate::model::animation::track_group;
use crate::model::mdl::Span;
use crate::model::scene::{impl_node_flags, NodeFlagInterpretation};
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::ModelVersion;
use crate::model::{mdl, mdx};
use bitfield::bitfield;
track_group! {
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

use crate::model::FixedText;
use crate::model::{Model, Node};

const PATH_SIZE: usize = 260;

/// A Classic particle emitter with optional animated properties.
///
/// Choose model or image resources with the emitter flags, and animate emission
/// through `tracks`.
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
            get = "Node::mdl_flags",
            set = "Node::set_mdl_flags",
            EmitterUsesMdl = 32768,
            EmitterUsesTga = 65536
        )
    )]
    /// Shared node.
    pub node: Node<ParticleEmitterFlags>,
    #[mdl(
        animatable = "EmissionRate",
        track = "ParticleTrack::EmissionRate",
        default
    )]
    /// Emission rate.
    pub emission_rate: f32,
    #[mdl(animatable = "Gravity", track = "ParticleTrack::Gravity", default)]
    /// Gravity.
    pub gravity: f32,
    #[mdl(animatable = "Longitude", track = "ParticleTrack::Longitude", default)]
    /// Longitude.
    pub longitude: f32,
    #[mdl(animatable = "Latitude", track = "ParticleTrack::Latitude", default)]
    /// Latitude.
    pub latitude: f32,
    #[mdl(property = "Path", default)]
    /// Model or image resource used for each particle.
    pub path: FixedText<PATH_SIZE>,
    #[mdl(animatable = "LifeSpan", track = "ParticleTrack::Lifespan", default)]
    /// Particle lifetime.
    pub life_span: f32,
    #[mdl(animatable = "InitVelocity", track = "ParticleTrack::Speed", default)]
    /// Initial velocity.
    pub initial_velocity: f32,
    #[mdl(tracks, channels(Visibility = "ParticleTrack::Visibility"))]
    /// Animation tracks.
    pub tracks: Vec<ParticleTrack>,
}

impl ParticleEmitter {
    /// Creates an emitter with zeroed physical values.
    pub fn new<F: NodeFlagInterpretation>(node: Node<F>, path: &str) -> Result<Self, ValueError> {
        let mut emitter = Self {
            node: node.cast_flags(),
            emission_rate: 0.0,
            gravity: 0.0,
            longitude: 0.0,
            latitude: 0.0,
            path: FixedText::default(),
            life_span: 0.0,
            initial_velocity: 0.0,
            tracks: Vec::new(),
        };
        emitter.path.set_text(path)?;
        Ok(emitter)
    }
}

impl<V: ModelVersion> Model<V> {
    /// Returns owned copies of `PREM` records in file order.
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
    validate_node_kind(&value.node, 0x1000 | (value.node.flags.bits() & 0x18000))
}

bitfield! {
    /// Complete node flag word interpreted in this emitter's context.
    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq, mdx::Read, mdx::Write)]
    pub struct ParticleEmitterFlags(u32);
    /// Returns the stored bits.
    pub bits, _: 31, 0;
    /// Returns or changes `emitter_uses_mdl`.
    pub emitter_uses_mdl, set_emitter_uses_mdl: 15;
    /// Returns or changes `emitter_uses_tga`.
    pub emitter_uses_tga, set_emitter_uses_tga: 16;
}
impl ParticleEmitterFlags {
    const MASK: u32 = 0x18000;
    /// Interprets a node word, preserving every stored bit.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
}
impl ParticleEmitter {
    /// Returns behavioral flags using this emitter's bit meanings.
    pub fn flags(&self) -> ParticleEmitterFlags {
        ParticleEmitterFlags::from_bits(self.node.flags.bits() & ParticleEmitterFlags::MASK)
    }
    /// Changes emitter behavior while preserving every unrelated node bit.
    pub fn set_flags(&mut self, flags: ParticleEmitterFlags) {
        let bits = (self.node.flags.bits() & !ParticleEmitterFlags::MASK)
            | (flags.bits() & ParticleEmitterFlags::MASK);
        self.node.flags = ParticleEmitterFlags(bits);
    }
}

impl_node_flags!(ParticleEmitterFlags);
