//! Particles that use model or image resources.
use crate::model::mdl::Span;
use crate::model::scene::{impl_node_flags, NodeFlagInterpretation};
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::FixedText;
use crate::model::KnownChunk;
use crate::model::ModelDialect;
use crate::model::ParticleEmittersChunk;
use crate::model::ValueError;
use crate::model::{mdl, mdx};
use crate::model::{Animatable, Track};
use crate::model::{Model, Node};
use bitfield::bitfield;

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
    #[mdl(flatten)]
    /// Shared node.
    pub node: Node<ParticleEmitterFlags>,
    #[mdx(tag = *b"KPEE")]
    #[mdl(property = "EmissionRate", default)]
    /// Emission rate.
    pub emission_rate: Animatable<f32>,
    #[mdx(tag = *b"KPEG")]
    #[mdl(property = "Gravity", default)]
    /// Gravity.
    pub gravity: Animatable<f32>,
    #[mdx(tag = *b"KPLN")]
    #[mdl(property = "Longitude", default)]
    /// Longitude.
    pub longitude: Animatable<f32>,
    #[mdx(tag = *b"KPLT")]
    #[mdl(property = "Latitude", default)]
    /// Latitude.
    pub latitude: Animatable<f32>,
    #[mdl(property = "Path", default)]
    /// Model or image resource used for each particle.
    pub path: FixedText<PATH_SIZE>,
    #[mdx(tag = *b"KPEL")]
    #[mdl(property = "LifeSpan", default)]
    /// Particle lifetime.
    pub life_span: Animatable<f32>,
    #[mdx(tag = *b"KPES")]
    #[mdl(property = "InitVelocity", default)]
    /// Initial velocity.
    pub initial_velocity: Animatable<f32>,
    #[mdx(tag = *b"KPEV")]
    #[mdl(property = "Visibility")]
    /// Optional visibility animation.
    pub visibility: Option<Track<f32>>,
}

impl ParticleEmitter {
    /// Creates an emitter with zeroed physical values.
    pub fn new<F: NodeFlagInterpretation>(node: Node<F>, path: &str) -> Result<Self, ValueError> {
        let mut emitter = Self {
            node: node.cast_flags(),
            emission_rate: Animatable::Static(0.0),
            gravity: Animatable::Static(0.0),
            longitude: Animatable::Static(0.0),
            latitude: Animatable::Static(0.0),
            path: FixedText::default(),
            life_span: Animatable::Static(0.0),
            initial_velocity: Animatable::Static(0.0),
            visibility: None,
        };
        emitter.path.set_text(path)?;
        Ok(emitter)
    }
}

impl<D: ModelDialect> Model<D> {
    /// Returns owned copies of `PREM` records in file order.
    pub fn particle_emitters(&self) -> Vec<ParticleEmitter> {
        self.collect_chunk_records::<ParticleEmittersChunk>()
    }

    /// Replaces particle emitters with one `PREM` chunk, removing any duplicate chunks.
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
            EmitterUsesMdl = 32768,
            EmitterUsesTga = 65536,
        ),
        allow_bits = 0x1000
    )]
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
