//! Reforged PopcornFX effect resources and emission settings.
use crate::model::mdl::{is_zero, Span};
use crate::model::scene::{impl_node_flags, NodeFlagInterpretation};
use crate::model::scene::{set_node_kind, validate_node_kind};
use crate::model::Color;
use crate::model::FixedText;
use crate::model::KnownChunk;
use crate::model::ModelDialect;
use crate::model::ModelOwnership;
use crate::model::PopcornEmittersChunk;
use crate::model::SupportsReforgedChunks;
use crate::model::ValueError;
use crate::model::{mdl, mdx};
use crate::model::{Animatable, Track};
use crate::model::{Model, Node};
use bitfield::bitfield;

const PATH_SIZE: usize = 260;

/// A Reforged particle effect loaded from a PopcornFX resource.
#[derive(Clone, Debug, PartialEq, mdx::Read, mdx::Write, mdl::Read, mdl::Write, ModelOwnership)]
#[mdx(sized(tag = PopcornEmittersChunk::TAG))]
#[mdl(
    block = "ParticleEmitterPopcorn",
    after_read = "finish_popcorn",
    validate_write = "validate_popcorn"
)]
pub struct PopcornEmitter<'data> {
    #[mdl(flatten)]
    /// Shared node.
    pub node: Node<'data, PopcornFlags>,
    #[mdx(tag = *b"KPPL")]
    #[mdl(property = "LifeSpan", default = "one")]
    /// Particle lifetime.
    pub life_span: Animatable<'data, f32>,
    #[mdx(tag = *b"KPPE")]
    #[mdl(property = "EmissionRate", default = "one")]
    /// Emission rate.
    pub emission_rate: Animatable<'data, f32>,
    #[mdx(tag = *b"KPPS")]
    #[mdl(property = "Speed", default = "one")]
    /// Particle speed.
    pub speed: Animatable<'data, f32>,
    #[mdx(tag = *b"KPPC")]
    #[mdl(property = "Color", default = "white")]
    /// RGB particle color.
    pub color: Animatable<'data, Color>,
    #[mdx(tag = *b"KPPA")]
    #[mdl(property = "Alpha", default = "one")]
    /// Base alpha.
    pub alpha: Animatable<'data, f32>,
    #[mdl(property = "ReplaceableId", default, skip_if = "is_zero")]
    /// Replaceable texture ID.
    pub replaceable_id: u32,
    #[mdl(property = "Path", default)]
    /// Path to the PopcornFX effect resource, typically a `.pkfx` file.
    pub path: FixedText<PATH_SIZE>,
    #[mdl(property = "AnimVisibilityGuide", default)]
    /// Animation-name rules controlling effect visibility. Line breaks are literal.
    pub visibility_guide: FixedText<PATH_SIZE>,
    #[mdx(tag = *b"KPPV")]
    #[mdl(property = "Visibility")]
    /// Optional visibility animation.
    pub visibility: Option<Track<'data, f32>>,
}

impl<'data> PopcornEmitter<'data> {
    /// Creates an emitter with unit lifespan, emission rate, speed, alpha, and white color.
    pub fn new<F: NodeFlagInterpretation>(
        node: Node<'data, F>,
        path: &str,
        visibility_guide: &str,
    ) -> Result<Self, ValueError> {
        let mut emitter = Self {
            node: node.cast_flags(),
            life_span: Animatable::Static(1.0),
            emission_rate: Animatable::Static(1.0),
            speed: Animatable::Static(1.0),
            color: Animatable::Static([1.0; 3]),
            alpha: Animatable::Static(1.0),
            replaceable_id: 0,
            path: FixedText::default(),
            visibility_guide: FixedText::default(),
            visibility: None,
        };
        emitter.path.set_text(path)?;
        emitter.visibility_guide.set_text(visibility_guide)?;
        Ok(emitter)
    }
}

impl<'data, D: ModelDialect> Model<'data, D> {
    /// Borrows records in chunk and record order.
    pub fn try_popcorn_emitters<'s>(
        &'s self,
    ) -> Result<impl Iterator<Item = &'s PopcornEmitter<'data>> + use<'data, 's, D>, ValueError>
    {
        self.check_chunk_version(*b"CORN")?;
        Ok(self.records::<PopcornEmittersChunk<'_>>())
    }

    /// Returns an error if this model version does not support `CORN`.
    pub fn try_set_popcorn_emitters(
        &mut self,
        emitters: &[PopcornEmitter<'data>],
    ) -> Result<(), ValueError> {
        self.check_chunk_version(*b"CORN")?;
        self.replace_chunk(PopcornEmittersChunk::new(emitters.to_vec()));
        Ok(())
    }
}

impl<'data, D: ModelDialect> Model<'data, D>
where
    D::Version: SupportsReforgedChunks,
{
    /// Borrows records in chunk and record order.
    pub fn popcorn_emitters<'s>(
        &'s self,
    ) -> impl Iterator<Item = &'s PopcornEmitter<'data>> + use<'data, 's, D> {
        self.records::<PopcornEmittersChunk<'_>>()
    }

    /// Replaces `CORN` records.
    pub fn set_popcorn_emitters(&mut self, emitters: &[PopcornEmitter<'data>]) {
        self.replace_chunk(PopcornEmittersChunk::new(emitters.to_vec()));
    }
}

fn one<'data>() -> Animatable<'data, f32> {
    Animatable::Static(1.0)
}
fn white<'data>() -> Animatable<'data, Color> {
    Animatable::Static([1.0; 3])
}
fn finish_popcorn<'data>(value: &mut PopcornEmitter<'data>, _: Span) -> Result<(), mdl::ReadError> {
    set_node_kind(&mut value.node, 0x1000);
    Ok(())
}
fn validate_popcorn<'data>(value: &PopcornEmitter<'data>) -> Result<(), mdl::WriteError> {
    validate_node_kind(&value.node, 0x1000 | (value.node.flags.bits() & 0x78000))
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
            Unshaded = 32768,
            Unfogged = 131072,
            PopcornScaling = 262144,
        ),
        allow_bits = 0x1000
    )]
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
    /// Interprets a node word, preserving every stored bit.
    pub const fn from_bits(bits: u32) -> Self {
        Self(bits)
    }
}
impl<'data> PopcornEmitter<'data> {
    /// Returns behavioral flags using this emitter's bit meanings.
    pub fn flags(&self) -> PopcornFlags {
        PopcornFlags::from_bits(self.node.flags.bits() & PopcornFlags::MASK)
    }
    /// Changes emitter behavior while preserving every unrelated node bit.
    pub fn set_flags(&mut self, flags: PopcornFlags) {
        let bits =
            (self.node.flags.bits() & !PopcornFlags::MASK) | (flags.bits() & PopcornFlags::MASK);
        self.node.flags = PopcornFlags(bits);
    }
}

impl_node_flags!(PopcornFlags);
