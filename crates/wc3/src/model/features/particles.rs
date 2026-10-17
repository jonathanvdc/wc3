//! Owned emitter components, with model-clock controls separate from particle-age appearance.
use super::effect_components::*;
use super::programmed_effects::{ExternalModel, VariationControl};
use crate::model::animation::programmed::Property;
use crate::model::materials::mdlx::Blend;
use crate::model::profiles::{m2, m3, mdlx};
use crate::model::scene::keyframed::Frames;
use crate::model::values::layered::{Particle2Flags, ParticleFlags as ModelParticleFlags};
use crate::model::values::model::{
    ParticleBlend, ParticleFlags as AgeDrivenFlags, ParticleGenerator, ParticleHeadTail,
};
use crate::model::values::programmed::*;
use crate::model::ImageResourceId;
use crate::model::ModelResourceId;
use crate::model::{
    Animated, FeatureId, Link, MaterialId, MeshId, TextureId, ValidateLinks, ZeroVec,
};
use std::borrow::Cow;
/// Animated scalar in the model's clock, retaining programmed reset values.
#[derive(Clone, Debug)]
pub enum ParticleScalar {
    /// A shared clip binding and its baseline value.
    Animated(Animated<f32>),
    /// A programmed property retaining null/reset values and interpolation flags.
    Programmed(Property<f32>),
}
/// Common emission controls.
#[derive(Clone, Debug, Default, PartialEq)]
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
    /// Independently animated programmed burst count.
    Programmed(Property<u16>),
}
/// Authored activation semantics.
#[derive(Clone, Debug)]
pub enum ParticleActivation {
    /// Fractional authored visibility.
    Fractional(Animated<f32>),
    /// Discrete authored activation.
    Boolean(Animated<bool>),
    /// Activation is determined by the programmed state flags.
    Programmed,
}
/// Common initial velocity and acceleration controls.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParticleMotion<S = ParticleScalar, V = ParticleScalar, G = ParticleGravity> {
    /// Initial velocity magnitude in model units per second.
    pub speed: S,
    /// Independently authored random contribution; its convention follows the simulation details.
    pub variation: V,
    /// Directional acceleration, vector acceleration, or explicitly unresolved control words.
    pub gravity: G,
}
/// Gravity representation distinguishes directional magnitude from vectors and unresolved source words.
#[derive(Clone, Debug)]
pub enum ParticleGravity {
    /// Acceleration magnitude along the emitter gravity direction.
    Directional(Animated<f32>),
    /// Acceleration vector in model coordinates.
    Vector(Animated<[f32; 3]>),
    /// Programmed scalar gravity and unresolved integer horizontal words.
    Programmed {
        /// Unresolved source X-gravity control word.
        x: u32,
        /// Unresolved source Y-gravity control word.
        y: u32,
        /// Authored gravity parameter.
        magnitude: f32,
    },
    /// Programmed effect gravity with floating horizontal components.
    ProgrammedVector {
        /// Authored X-gravity parameter.
        x: f32,
        /// Authored Y-gravity parameter.
        y: f32,
        /// Authored gravity parameter.
        magnitude: f32,
    },
}
/// Lifetime in seconds with an independently authored random contribution.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParticleLifetime<D = ParticleScalar, V = ParticleScalar> {
    /// Particle lifetime in seconds.
    pub duration: D,
    /// Independently authored random contribution; its convention follows the simulation details.
    pub variation: V,
}
/// Complete composed particle definition.
#[derive(Clone, Debug, Default, PartialEq)]
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
    /// Programmed simulation with secondary emitters and variation functions.
    Programmed(Box<ProgrammedParticle<'a>>),
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelParticle {
    /// Model or image resource instantiated per particle.
    pub resource: ParticleResource,
    /// Source longitude control, retained even when the renderer does not consume it.
    pub longitude: Animated<f32>,
    /// Emission-cone latitude in radians for Warcraft III model particles.
    pub latitude: Animated<f32>,
    /// Interpret the resource as a text image.
    pub text_resource: bool,
    /// Emitter controls with unresolved meanings.
    pub unclassified: ModelParticleFlags,
}

/// Simulation-specific controls composed with common emission, motion and lifetime.
#[derive(Clone, Debug, Default, PartialEq)]
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

/// Simulation-specific controls composed with common emission, motion and lifetime.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammedParticle<'a> {
    /// Forces.
    pub forces: ProgrammedForces,
    /// Noise.
    pub noise: ProgrammedNoise,
    /// Material drawn by this emitter.
    pub material: Option<MaterialId>,
    /// Supplementary StarCraft II emitter flags; unknown bits remain available.
    pub additional_flags: ParticleAdditionalFlags,
    /// Kill radius parameter stored as a floating-point value.
    pub kill_radius: f32,
    /// Index plus length selector; sentinel and unrecognized values are retained.
    pub index_plus_length: u32,
    /// Stored upper bound on the particle population.
    pub max_particles: u32,
    /// Shape from which this StarCraft II emitter produces particles.
    pub emitter_shape: EmitterShape,
    /// Dimensions of the outer emitter shape.
    pub shape_outer: Property<[f32; 3]>,
    /// Dimensions of the inner emitter shape.
    pub shape_inner: Property<[f32; 3]>,
    /// Outer radius control of the emitter’s source shape.
    pub outer_radius: Property<f32>,
    /// Inner radius control of the emitter’s source shape.
    pub inner_radius: Property<f32>,
    /// Mesh regions sampled by vertex-based emission.
    pub shape_sections: Vec<MeshId>,
    /// Source velocity-mode code; the meanings of its values are unresolved.
    pub velocity_type: VelocityKind,
    /// Source control word enabling size randomization.
    pub size_random_enable: u32,
    /// Random contribution to the three-component size property.
    pub size_random_animation: Property<[f32; 3]>,
    /// Source control word enabling rotation randomization.
    pub rotation_random_enable: u32,
    /// Random contribution to the three-component rotation property.
    pub rotation_random_animation: Property<[f32; 3]>,
    /// Source control word enabling color randomization.
    pub color_random_enable: u32,
    /// Random color contribution at the beginning of particle life.
    pub color_start_random: Property<[f32; 4]>,
    /// Random color contribution at the intermediate age point.
    pub color_mid_random: Property<[f32; 4]>,
    /// Random color contribution at the end of particle life.
    pub color_end_random: Property<[f32; 4]>,
    /// Source control word enabling alpha randomization.
    pub alpha_random_enable: u32,
    /// Flipbook start init index selector; sentinel and unrecognized values are retained.
    pub flipbook_start_init_index: u8,
    /// Flipbook start stop index selector; sentinel and unrecognized values are retained.
    pub flipbook_start_stop_index: u8,
    /// Flipbook end init index selector; sentinel and unrecognized values are retained.
    pub flipbook_end_init_index: u8,
    /// Flipbook end stop index selector; sentinel and unrecognized values are retained.
    pub flipbook_end_stop_index: u8,
    /// Flipbook mid time parameter; units follow the enclosing animation or effect.
    pub flipbook_mid_time: f32,
    /// Number of columns in the emitter’s flipbook texture atlas.
    pub flipbook_columns: u16,
    /// Number of rows in the emitter’s flipbook texture atlas.
    pub flipbook_rows: u16,
    /// Finite fraction occupied by one atlas column; absent for an undefined fraction.
    pub flipbook_column_fraction: Option<f32>,
    /// Finite fraction occupied by one atlas row; absent for an undefined fraction.
    pub flipbook_row_fraction: Option<f32>,
    /// Particle collision bounce parameter.
    pub bounce: f32,
    /// Particle collision friction parameter.
    pub friction: f32,
    /// Emitter spawned after a particle collision, when configured.
    pub collision_spawn: Option<FeatureId>,
    /// Lower bound on the collision-triggered particle count.
    pub collision_spawn_min: u32,
    /// Upper bound on the collision-triggered particle count.
    pub collision_spawn_max: u32,
    /// Collision spawn chance parameter stored as a floating-point value.
    pub collision_spawn_chance: f32,
    /// Collision spawn energy parameter stored as a floating-point value.
    pub collision_spawn_energy: f32,
    /// Source control word for particle death after bouncing; detailed value semantics are unresolved.
    pub collision_die_bounce: u32,
    /// Geometry and orientation mode used to draw a particle instance.
    pub instance_type: ParticleInstance,
    /// Length control for the particle trail.
    pub tail_length: f32,
    /// Three-component orientation parameter of the particle instance.
    pub instance_angle: [f32; 3],
    /// Instance distance parameter stored as a floating-point value.
    pub instance_distance: f32,
    /// Pitch variation settings, including the function, amplitude, and frequency.
    pub pitch: VariationControl,
    /// Yaw variation settings, including the function, amplitude, and frequency.
    pub yaw: VariationControl,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: VariationControl,
    /// Size variation settings, including the function, amplitude, and frequency.
    pub size: VariationControl,
    /// Opacity control for the emitted geometry.
    pub alpha: VariationControl,
    /// Color variation settings, including the function, amplitude, and frequency.
    pub color: VariationControl,
    /// Rotation variation settings, including the function, amplitude, and frequency.
    pub rotation: VariationControl,
    /// Horizontal variation settings, including the function, amplitude, and frequency.
    pub horizontal: VariationControl,
    /// Vertical variation settings, including the function, amplitude, and frequency.
    pub vertical: VariationControl,
    /// Animated particle-velocity control retained in the StarCraft II representation.
    pub particle_velocity: Property<f32>,
    /// Animated phase-shift control; its effect-specific interpretation is unresolved.
    pub phase_shift: Property<f32>,
    /// Stored state bits; unknown bits are retained.
    pub flags: ParticleFlags,
    /// Rotation behavior flags of the StarCraft II emitter.
    pub rotation_flags: ParticleRotationFlags,
    /// Interpolation mode for the particle color controls.
    pub color_smoothing: Smoothing,
    /// Interpolation mode for the particle size controls.
    pub size_smoothing: Smoothing,
    /// Interpolation mode for the particle rotation controls.
    pub rotation_smoothing: Smoothing,
    /// Animated alpha cutoff used when drawing particles.
    pub alpha_threshold: Property<f32>,
    /// Animated offset of particle texture coordinates.
    pub uv_offset: Property<[f32; 2]>,
    /// Animated three-component texture-orientation property.
    pub uv_angle: Property<[f32; 3]>,
    /// Animated tiling factors for the two texture-coordinate axes.
    pub uv_tiling: Property<[f32; 2]>,
    /// Authored spline control points defining the emission curve.
    pub spline_points: Cow<'a, [Property<[f32; 3]>]>,
    /// Wind multiplier parameter stored as a floating-point value.
    pub wind_multiplier: f32,
    /// Stored level-of-detail reduction control.
    pub lod_reduce: u32,
    /// Stored level-of-detail cutoff control.
    pub lod_cut: u32,
    /// Animated lower-bound control; its source effect semantics are unresolved.
    pub lower_bound: Property<f32>,
    /// Animated upper-bound control; its source effect semantics are unresolved.
    pub upper_bound: Property<f32>,
    /// Original emitter when this instance copies its configuration.
    pub trail: Option<FeatureId>,
    /// Trail chance parameter stored as a floating-point value.
    pub trail_chance: f32,
    /// Animated emission-rate control for the linked trail emitter.
    pub trail_emission_rate: Property<f32>,
    /// Splat projection index selector; sentinel and unrecognized values are retained.
    pub projection: Option<FeatureId>,
    /// Splat chance parameter stored as a floating-point value.
    pub splat_chance: f32,
    /// External model resources used to draw instanced particles.
    pub models: Vec<ExternalModel<'a>>,
    /// Spawn ribbon on bounce chance parameter stored as a floating-point value.
    pub spawn_ribbon_on_bounce_chance: f32,
    /// Original emitter when this instance copies its configuration.
    pub ribbon: Option<FeatureId>,
    /// Noise smoothness parameter stored as a floating-point value.
    pub noise_smoothness: f32,

    /// Direction controls.
    pub direction: ProgrammedDirection,
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
    pub texture: Option<TextureId>,
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
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtlasParticleShape {
    /// Emission-cone latitude in degrees.
    pub latitude: Animated<f32>,
    /// Length of the emission region.
    pub length: Animated<f32>,
    /// Width of the emission region.
    pub width: Animated<f32>,
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
    pub textures: Vec<TextureId>,
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
    pub vertical_range: Animated<f32>,
    /// Horizontal emission spread in radians.
    pub horizontal_range: Animated<f32>,
    /// Plane Y extent or sphere maximum radius.
    pub area_width: Animated<f32>,
    /// Plane X extent or sphere minimum radius.
    pub area_length: Animated<f32>,
    /// Z coordinate of the source point for aimed emission.
    pub z_source: Animated<f32>,
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
    /// Programmed age control points with explicit intermediate times and hold intervals.
    Programmed(Box<ProgrammedAge>),
}

/// Integer horizontal gravity controls whose physical interpretation is unresolved.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammedParticleGravity {
    /// Unresolved horizontal control word.
    pub x: u32,
    /// Unresolved horizontal control word.
    pub y: u32,
    /// Authored gravity parameter.
    pub magnitude: f32,
}
/// External-resource simulation with direction-based gravity and visibility.
pub type ResourceParticle = ParticleComposition<
    ParticleEmission<Animated<f32>, (), Animated<f32>, ()>,
    ParticleMotion<Animated<f32>, (), Animated<f32>>,
    ParticleLifetime<Animated<f32>, ()>,
    (),
    ModelParticle,
>;
/// Atlas simulation with keyed burst selection and staged particle-age appearance.
pub type AtlasParticles = ParticleComposition<
    ParticleEmission<Animated<f32>, bool, Animated<f32>, ()>,
    ParticleMotion<Animated<f32>, Animated<f32>, Animated<f32>>,
    ParticleLifetime<f32, ()>,
    StagedParticleAppearance,
    AtlasParticle,
>;
/// Curve-driven age appearance and vector-gravity simulation.
pub type AgeDrivenParticles<'a> = ParticleComposition<
    ParticleEmission<Animated<f32>, (), Animated<bool>>,
    ParticleMotion<Animated<f32>, Animated<f32>, Animated<[f32; 3]>>,
    ParticleLifetime<Animated<f32>, f32>,
    ParticleAgeCurves<'a>,
    AgeDrivenParticle<'a>,
>;
/// Programmed simulation with reset values and authored control flags.
pub type ProgrammedParticles<'a> = ParticleComposition<
    ParticleEmission<Property<f32>, Property<u16>, (), ()>,
    ParticleMotion<Property<f32>, Property<f32>, ProgrammedParticleGravity>,
    ParticleLifetime<Property<f32>, Property<f32>>,
    ProgrammedAge,
    ProgrammedParticle<'a>,
>;
