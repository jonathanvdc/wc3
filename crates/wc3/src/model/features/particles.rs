//! Owned emitter components, with model-clock controls separate from particle-age appearance.

use crate::model::materials::mdlx::Blend;
use crate::model::profiles::{m2, mdlx};
use crate::model::scene::keyframed::Frames;
use crate::model::values::layered::{Particle2Flags, ParticleFlags as ModelParticleFlags};
use crate::model::values::model::{
    ParticleBlend, ParticleFlags as AgeDrivenFlags, ParticleGenerator, ParticleHeadTail,
};

use crate::model::Construct;
use crate::model::ImageResourceId;
use crate::model::ModelResourceId;
use crate::model::{Link, Property, Texture, ValidateLinks, ZeroVec};
/// Animated scalar in the model's clock.
#[derive(Clone, Debug)]
pub enum ParticleScalar {
    /// A shared clip binding and its baseline value.
    Property(Property<f32>),
}
/// Common emission controls.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ParticleEmission<
    R = ParticleScalar,
    T = ParticleTrigger,
    A = ParticleActivation,
    V = f32,
> {
    /// Particles per second, or keyed burst magnitude when selected by trigger.
    pub rate: R,
    /// Independently authored random contribution; its convention follows the simulation details.
    pub variation: V,
    /// Continuous generation or explicit burst interpretation.
    pub trigger: T,
    /// Fractional visibility and discrete enablement remain distinct.
    pub activation: A,
}
/// Emission event interpretation.
#[derive(Clone, Debug)]
pub enum ParticleTrigger {
    /// Generate particles continuously at the authored rate.
    Continuous,
    /// Emit a burst when a rate key is encountered.
    KeyedBurst,
}
/// Authored activation semantics.
#[derive(Clone, Debug)]
pub enum ParticleActivation {
    /// Fractional authored visibility.
    Fractional(Property<f32>),
    /// Discrete authored activation.
    Boolean(Property<bool>),
}
/// Common initial velocity and acceleration controls.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ParticleMotion<S = ParticleScalar, V = ParticleScalar, G = ParticleGravity> {
    /// Initial velocity magnitude in model units per second.
    pub speed: S,
    /// Independently authored random contribution; its convention follows the simulation details.
    pub variation: V,
    /// Directional or vector acceleration.
    pub gravity: G,
}
/// Gravity represented as a directional magnitude or a vector.
#[derive(Clone, Debug)]
pub enum ParticleGravity {
    /// Acceleration magnitude along the emitter gravity direction.
    Directional(Property<f32>),
    /// Acceleration vector in model coordinates.
    Vector(Property<[f32; 3]>),
}
/// Lifetime in seconds with an independently authored random contribution.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ParticleLifetime<D = ParticleScalar, V = ParticleScalar> {
    /// Particle lifetime in seconds.
    pub duration: D,
    /// Independently authored random contribution; its convention follows the simulation details.
    pub variation: V,
}
/// Complete composed particle definition.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ParticleComposition<E, M, L, A, D> {
    /// Model-clock emission timing and activation.
    pub emission: E,
    /// Initial velocity and acceleration.
    pub motion: M,
    /// Particle lifetime and its random contribution.
    pub lifetime: L,
    /// Appearance evaluated against normalized particle age, independently of model time.
    pub appearance: A,
    /// Explicit simulation, source shape, and rendering semantics.
    pub details: D,
}
/// Broad particle composition selected by the Universal profile.
pub type ParticleDefinition<'a> = ParticleComposition<
    ParticleEmission,
    ParticleMotion,
    ParticleLifetime,
    ParticleAppearance<'a>,
    ParticleDetails<'a>,
>;
/// Distinct simulation and appearance rules after common controls are extracted.
#[derive(Clone, Debug)]
pub enum ParticleDetails<'a> {
    /// External-resource particles retaining their source emission-angle convention.
    Model(ModelParticle),
    /// Sprite particles with staged appearance and an atlas.
    Atlas(Box<AtlasParticle>),
    /// Particles with independently authored normalized-age curves.
    AgeDriven(Box<AgeDrivenParticle<'a>>),
}
/// Dependency instantiated by a Warcraft III resource particle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleResource {
    /// Child model geometry and animation.
    Model(Option<ModelResourceId>),
    /// Image-based particle content.
    Image(Option<ImageResourceId>),
}
impl Default for ParticleResource {
    fn default() -> Self {
        Self::Image(None)
    }
}
impl ValidateLinks for ParticleResource {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Model(id) => id.visit_links(visitor),
            Self::Image(id) => id.visit_links(visitor),
        }
    }
}
/// Simulation-specific controls composed with common emission, motion and lifetime.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ModelParticle {
    /// Model or image resource instantiated per particle.
    pub resource: ParticleResource,
    /// Source longitude control, retained even when the renderer does not consume it.
    pub longitude: Property<f32>,
    /// Emission-cone latitude in radians for Warcraft III model particles.
    pub latitude: Property<f32>,
    /// Interpret the resource as a text image.
    pub text_resource: bool,
    /// Emitter controls with unresolved meanings.
    pub unclassified: ModelParticleFlags,
}

/// Simulation-specific controls composed with common emission, motion and lifetime.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct AtlasParticle {
    /// Model-space rather than world-space simulation.
    pub model_space: bool,
    /// Emitter controls with unresolved meanings.
    pub unclassified: Particle2Flags,

    /// Rendering controls.
    pub rendering: AtlasParticleRendering,
    /// Shape controls.
    pub shape: AtlasParticleShape,
}

/// Simulation-specific controls composed with common emission, motion and lifetime.
#[derive(Clone, Debug, PartialEq)]
pub struct AgeDrivenParticle<'a> {
    /// Runtime emitter identifier.
    pub id: u32,
    /// Generator behavior and extension bits.
    pub flags: AgeDrivenFlags,
    /// Twinkle frequency.
    pub twinkle_speed: f32,
    /// Twinkle visibility fraction.
    pub twinkle_percent: f32,
    /// Twinkle scale range.
    pub twinkle_scale: [f32; 2],
    /// Multiplier for velocity inherited from the carrying node.
    pub inherit_velocity: f32,
    /// Exponential velocity damping coefficient.
    pub drag: f32,
    /// Billboard initial angle, variation, angular speed, speed variation.
    pub spin: [f32; 4],
    /// Minimum angular-velocity vector for model particles.
    pub tumble_minimum: [f32; 3],
    /// Maximum angular-velocity vector for model particles.
    pub tumble_maximum: [f32; 3],
    /// Static wind vector.
    pub wind: [f32; 3],
    /// Wind timing parameter whose response interpretation is unresolved.
    pub wind_time: f32,
    /// Two follow-speed and follow-scale parameter pairs.
    pub follow: [[f32; 2]; 2],

    /// Rendering controls.
    pub rendering: AgeDrivenParticleRendering,
    /// Shape controls.
    pub shape: AgeDrivenParticleShape<'a>,
}

/// Appearance controls shared independently of emitter timing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StagedParticleAppearance {
    /// Lifetime fraction at the middle appearance stage.
    pub middle_time: f32,
    /// RGB colors at the three lifetime stages.
    pub colors: [[f32; 3]; 3],
    /// Opacity at the three lifetime stages.
    pub alpha: [u8; 3],
    /// Scale at the three lifetime stages.
    pub scale: [f32; 3],
    /// Start, end, and repeat counts for head/lifespan/decay/tail atlas animations.
    pub uv_intervals: [[u32; 3]; 4],
}
/// Rendering controls shared independently of emitter timing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtlasParticleRendering {
    /// Color composition operation.
    pub blend: Blend,
    /// Atlas dimensions in rows and columns.
    pub atlas: [u32; 2],
    /// Sprite and trail rendering choice.
    pub frames: Frames,
    /// Trail time span in seconds.
    pub tail_length: f32,
    /// Atlas texture resource.
    pub texture: Option<Texture>,
    /// Render-order priority.
    pub priority_plane: u32,
    /// External replaceable-texture category.
    pub replaceable: u32,
    /// Keep particles aligned to the XY plane.
    pub xy_quad: bool,
    /// Sort particles far to near.
    pub sort_far_z: bool,
    /// Bypass lighting.
    pub unlit: bool,
    /// Bypass fog.
    pub unfogged: bool,
}
/// Shape controls shared independently of emitter timing.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct AtlasParticleShape {
    /// Emission-cone latitude in degrees.
    pub latitude: Property<f32>,
    /// Length of the emission region.
    pub length: Property<f32>,
    /// Width of the emission region.
    pub width: Property<f32>,
    /// Apply linear rather than exponential variation.
    pub line_emitter: bool,
}
/// Rendering controls shared independently of emitter timing.
#[derive(Clone, Debug, PartialEq)]
pub struct AgeDrivenParticleRendering {
    /// Particle compositing operation.
    pub blend: ParticleBlend,
    /// External particle-color table category.
    pub color_index: u16,
    /// Ordered particle texture resources.
    pub textures: Vec<Texture>,
    /// Native texture-program selectors, when an emitter has a container-level
    /// combiner programme. Byte meanings remain opaque; absence selects the
    /// baseline texture path rather than discarding an unknown programme.
    pub texture_program: Option<[u8; 2]>,
    /// Model geometry resource used for model particles.
    pub model: Option<ModelResourceId>,
    /// Resource containing child emitters.
    pub child_model: Option<ModelResourceId>,
    /// Render head, tail, or both.
    pub head_tail: ParticleHeadTail,
    /// Per-layer UV scales.
    pub texture_scale: [f32; 2],
    /// Tile rotation or signed ordering parameter.
    pub texture_tile_rotation: i16,
    /// Atlas rows and columns.
    pub atlas: [u16; 2],
    /// Tail extent parameter; client time-versus-distance meaning is unresolved.
    pub tail_length: f32,
    /// Layer/component UV centers followed by ranges, in texture units.
    pub scroll: Option<[f32; 8]>,
}
/// Shape controls shared independently of emitter timing.
#[derive(Clone, Debug, PartialEq)]
pub struct AgeDrivenParticleShape<'a> {
    /// Emission shape.
    pub generator: ParticleGenerator,
    /// Legacy particle shape selector whose client meaning is unresolved.
    pub legacy_shape: u8,
    /// Vertical emission spread in radians.
    pub vertical_range: Property<f32>,
    /// Horizontal emission spread in radians.
    pub horizontal_range: Property<f32>,
    /// Plane Y extent or sphere maximum radius.
    pub area_width: Property<f32>,
    /// Plane X extent or sphere minimum radius.
    pub area_length: Property<f32>,
    /// Z coordinate of the source point for aimed emission.
    pub z_source: Property<f32>,
    /// Emission path in model coordinates.
    pub spline: ZeroVec<'a, [f32; 3]>,
}

mod age;
pub use age::*;
mod conversions;
mod validation;

/// Particle-age appearance programs, separate from emitter animation on the model clock.
#[derive(Clone, Debug)]
pub enum ParticleAppearance<'a> {
    /// The instantiated resource supplies its own appearance; no particle-age controls are authored.
    Resource,
    /// Three lifetime stages with independent atlas intervals.
    Staged(StagedParticleAppearance),
    /// Independently authored normalized-age color, opacity, scale, and atlas curves.
    Curves(Box<ParticleAgeCurves<'a>>),
}

/// External-resource simulation with direction-based gravity and visibility.
pub type ResourceParticle = ParticleComposition<
    ParticleEmission<Property<f32>, (), Property<f32>, ()>,
    ParticleMotion<Property<f32>, (), Property<f32>>,
    ParticleLifetime<Property<f32>, ()>,
    (),
    ModelParticle,
>;
/// Atlas simulation with keyed burst selection and staged particle-age appearance.
pub type AtlasParticles = ParticleComposition<
    ParticleEmission<Property<f32>, bool, Property<f32>, ()>,
    ParticleMotion<Property<f32>, Property<f32>, Property<f32>>,
    ParticleLifetime<f32, ()>,
    StagedParticleAppearance,
    AtlasParticle,
>;
/// Curve-driven age appearance and vector-gravity simulation.
pub type AgeDrivenParticles<'a> = ParticleComposition<
    ParticleEmission<Property<f32>, (), Property<bool>>,
    ParticleMotion<Property<f32>, Property<f32>, Property<[f32; 3]>>,
    ParticleLifetime<Property<f32>, f32>,
    ParticleAgeCurves<'a>,
    AgeDrivenParticle<'a>,
>;
