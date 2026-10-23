//! Owned emitter components, with model-clock controls separate from particle-age appearance.

use crate::model::common::{Detached, IntoOwned};
use crate::model::materials::mdlx::Blend;
use crate::model::profiles::mdlx::Frames;
use crate::model::values::model::{
    ParticleBlend, ParticleFlags as AgeDrivenFlags, ParticleGenerator, ParticleHeadTail,
};
use glam::{Vec2, Vec3, Vec4};

use crate::model::Construct;
use crate::model::ImageResourceId;
use crate::model::ModelResourceId;
use crate::model::{Link, PackedArray, Property, Texture, ValidateLinks};
/// Common emission controls.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ParticleEmission<R, T, A, V = f32> {
    /// Particles per second, or keyed burst magnitude when selected by trigger.
    pub rate: R,
    /// Independently authored random contribution; its convention follows the simulation details.
    pub variation: V,
    /// Continuous generation or explicit burst interpretation.
    pub trigger: T,
    /// Fractional visibility and discrete enablement remain distinct.
    pub activation: A,
}
/// Common initial velocity and acceleration controls.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ParticleMotion<S, V, G> {
    /// Initial velocity magnitude in model units per second.
    pub speed: S,
    /// Independently authored random contribution; its convention follows the simulation details.
    pub variation: V,
    /// Directional or vector acceleration.
    pub gravity: G,
}
/// Lifetime in seconds with an independently authored random contribution.
#[derive(Clone, Debug, Construct, PartialEq)]
pub struct ParticleLifetime<D, V> {
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
/// Dependency instantiated by a Warcraft III resource particle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MdlxParticle1Resource {
    /// Child model geometry and animation.
    Model(Option<ModelResourceId>),
    /// Image-based particle content.
    Image(Option<ImageResourceId>),
}
impl Default for MdlxParticle1Resource {
    fn default() -> Self {
        Self::Image(None)
    }
}
impl ValidateLinks for MdlxParticle1Resource {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::Model(id) => id.visit_links(visitor),
            Self::Image(id) => id.visit_links(visitor),
        }
    }
}
/// Simulation-specific controls composed with common emission, motion and lifetime.
#[derive(Clone, Debug, Construct, PartialEq, IntoOwned)]
pub struct MdlxParticle1Details {
    /// Model or image resource instantiated per particle.
    pub resource: MdlxParticle1Resource,
    /// Source longitude control, retained even when the renderer does not consume it.
    pub longitude: Property<f32>,
    /// Emission-cone latitude in radians for Warcraft III model particles.
    pub latitude: Property<f32>,
    /// Interpret the resource as a text image.
    pub text_resource: bool,
}

/// Simulation-specific controls composed with common emission, motion and lifetime.
#[derive(Clone, Debug, Construct, PartialEq, IntoOwned)]
pub struct MdlxParticle2Details {
    /// Model-space rather than world-space simulation.
    pub model_space: bool,

    /// Rendering controls.
    pub rendering: MdlxParticle2Rendering,
    /// Shape controls.
    pub shape: MdlxParticle2Shape,
}

/// Simulation-specific controls composed with common emission, motion and lifetime.
#[derive(Clone, Debug, PartialEq, IntoOwned)]
pub struct M2ParticleDetails<'a> {
    /// Runtime emitter identifier.
    pub id: u32,
    /// Generator behavior and extension bits.
    pub flags: AgeDrivenFlags,
    /// Twinkle frequency.
    pub twinkle_speed: f32,
    /// Twinkle visibility fraction.
    pub twinkle_percent: f32,
    /// Twinkle scale range.
    pub twinkle_scale: Vec2,
    /// Multiplier for velocity inherited from the carrying node.
    pub inherit_velocity: f32,
    /// Exponential velocity damping coefficient.
    pub drag: f32,
    /// Billboard initial angle, variation, angular speed, speed variation.
    pub spin: Vec4,
    /// Minimum angular-velocity vector for model particles.
    pub tumble_minimum: Vec3,
    /// Maximum angular-velocity vector for model particles.
    pub tumble_maximum: Vec3,
    /// Static wind vector.
    pub wind: Vec3,
    /// Wind timing parameter whose response interpretation is unresolved.
    pub wind_time: f32,
    /// Two follow-speed and follow-scale parameter pairs.
    pub follow: [Vec2; 2],

    /// Rendering controls.
    pub rendering: M2ParticleRendering,
    /// Shape controls.
    pub shape: M2ParticleShape<'a>,
}

/// Appearance controls shared independently of emitter timing.
#[derive(Clone, Debug, Default, PartialEq, IntoOwned)]
pub struct MdlxParticle2Appearance {
    /// Lifetime fraction at the middle appearance stage.
    pub middle_time: f32,
    /// RGB colors at the three lifetime stages.
    pub colors: [Vec3; 3],
    /// Opacity at the three lifetime stages.
    pub alpha: [u8; 3],
    /// Scale at the three lifetime stages.
    pub scale: Vec3,
    /// Start, end, and repeat counts for head/lifespan/decay/tail atlas animations.
    pub uv_intervals: [[u32; 3]; 4],
}
/// Rendering controls shared independently of emitter timing.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MdlxParticle2Rendering {
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
pub struct MdlxParticle2Shape {
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
pub struct M2ParticleRendering {
    /// Particle compositing operation.
    pub blend: ParticleBlend,
    /// External particle-color table category.
    pub color_index: u16,
    /// Ordered particle texture resources.
    pub textures: Vec<Texture>,
    /// Native container-level texture-program selectors with opaque byte meanings.
    /// These are retained independently of recognized particle-composition flags.
    pub texture_program: Option<[u8; 2]>,
    /// Model geometry resource used for model particles.
    pub model: Option<ModelResourceId>,
    /// Resource containing child emitters.
    pub child_model: Option<ModelResourceId>,
    /// Render head, tail, or both.
    pub head_tail: ParticleHeadTail,
    /// Per-layer UV scales.
    pub texture_scale: Vec2,
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
#[derive(Clone, Debug, PartialEq, IntoOwned)]
pub struct M2ParticleShape<'a> {
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
    pub spline: PackedArray<'a, Vec3>,
}

mod age;
pub use age::*;
mod validation;

/// External-resource simulation with direction-based gravity and visibility.
pub type MdlxParticleEmitter1 = ParticleComposition<
    ParticleEmission<Property<f32>, (), Property<f32>, ()>,
    ParticleMotion<Property<f32>, (), Property<f32>>,
    ParticleLifetime<Property<f32>, ()>,
    (),
    MdlxParticle1Details,
>;
/// Atlas simulation with keyed burst selection and staged particle-age appearance.
pub type MdlxParticleEmitter2 = ParticleComposition<
    ParticleEmission<Property<f32>, bool, Property<f32>, ()>,
    ParticleMotion<Property<f32>, Property<f32>, Property<f32>>,
    ParticleLifetime<f32, ()>,
    MdlxParticle2Appearance,
    MdlxParticle2Details,
>;
/// Curve-driven age appearance and vector-gravity simulation.
pub type M2ParticleEmitter<'a> = ParticleComposition<
    ParticleEmission<Property<f32>, (), Property<bool>>,
    ParticleMotion<Property<f32>, Property<f32>, Property<Vec3>>,
    ParticleLifetime<Property<f32>, f32>,
    M2ParticleAppearance<'a>,
    M2ParticleDetails<'a>,
>;

owned_components!(ParticleEmission<R, T, A, V> { rate, trigger, activation, variation; });
owned_components!(ParticleMotion<S, V, G> { speed, variation, gravity; });
owned_components!(ParticleLifetime<D, V> { duration, variation; });
owned_components!(ParticleComposition<E, M, L, A, D> { emission, motion, lifetime, appearance, details; });
