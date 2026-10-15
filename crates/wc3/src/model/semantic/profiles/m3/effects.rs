//! Logical M3 emitters and their animation properties.
use super::{values::*, Property};
use crate::model::semantic::{FeatureId, MaterialId, MeshId, NodeId};
use std::borrow::Cow;

/// External model resource used by an instanced particle.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExternalModel<'a> {
    /// Resource filename when declared.
    pub path: Cow<'a, str>,
    /// Numeric source resource identity when declared.
    pub hash: Option<u32>,
}

/// Editable M3 Variation values.
#[derive(Clone, Debug, PartialEq)]
pub struct VariationControl {
    /// Variation function code retained from the source emitter.
    pub kind: Variation,
    /// Magnitude of variation applied to the associated property.
    pub amplitude: Property<f32>,
    /// Frequency parameter of the selected variation function.
    pub frequency: Property<f32>,
}

/// An emitter instance inheriting its definition from an independent particle emitter.
#[derive(Clone, Debug, PartialEq)]
pub struct ParticleCopy {
    /// Independent emitter supplying all controls not overridden here.
    pub source: FeatureId,
    /// Emission-rate override for this instance.
    pub emission_rate: Property<f32>,
    /// Burst-count override for this instance.
    pub squirt_amount: Property<u16>,
}

/// Complete independent particle-emitter definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Particle<'a> {
    /// Material drawn by this emitter.
    pub material: Option<MaterialId>,
    /// Supplementary StarCraft II emitter flags; unknown bits remain available.
    pub additional_flags: ParticleAdditionalFlags,
    /// Initial particle-speed property, including its unanimated and reset values.
    pub initial_speed: Property<f32>,
    /// Randomized initial-speed property in the source emitter.
    pub initial_speed_random: Property<f32>,
    /// Initial yaw property for emitted particles.
    pub initial_yaw: Property<f32>,
    /// Initial pitch property for emitted particles.
    pub initial_pitch: Property<f32>,
    /// Initial horizontal emission property in the StarCraft II emitter frame.
    pub initial_horizontal: Property<f32>,
    /// Initial vertical emission property in the StarCraft II emitter frame.
    pub initial_vertical: Property<f32>,
    /// Particle lifetime control retained from the source emitter.
    pub lifetime: Property<f32>,
    /// Random contribution to the particle-lifetime property.
    pub lifetime_random: Property<f32>,
    /// Kill radius parameter stored as a floating-point value.
    pub kill_radius: f32,
    /// Source X-gravity word. Its integer interpretation in particle emitters is unresolved.
    pub gravity_x: u32,
    /// Source Y-gravity word. Its integer interpretation in particle emitters is unresolved.
    pub gravity_y: u32,
    /// Gravity control retained from the source emitter.
    pub gravity: f32,
    /// Normalized particle age at the intermediate size control point.
    pub size_mid_time: f32,
    /// Normalized particle age at the intermediate color control point.
    pub color_mid_time: f32,
    /// Normalized particle age at the intermediate alpha control point.
    pub alpha_mid_time: f32,
    /// Normalized particle age at the intermediate rotation control point.
    pub rotation_mid_time: f32,
    /// Size mid hold time parameter; units follow the enclosing animation or effect.
    pub size_mid_hold_time: f32,
    /// Color mid hold time parameter; units follow the enclosing animation or effect.
    pub color_mid_hold_time: f32,
    /// Alpha mid hold time parameter; units follow the enclosing animation or effect.
    pub alpha_mid_hold_time: f32,
    /// Rotation mid hold time parameter; units follow the enclosing animation or effect.
    pub rotation_mid_hold_time: f32,
    /// Three-component particle-size property.
    pub size_animation: Property<[f32; 3]>,
    /// Three-component particle-rotation property.
    pub rotation_animation: Property<[f32; 3]>,
    /// Color property at the beginning of particle life.
    pub color_start: Property<[f32; 4]>,
    /// Color property at the intermediate particle-age point.
    pub color_mid: Property<[f32; 4]>,
    /// Color property at the end of particle life.
    pub color_end: Property<[f32; 4]>,
    /// Drag applied to particle motion.
    pub drag: f32,
    /// Mass parameter stored as a floating-point value.
    pub mass: f32,
    /// Mass random parameter stored as a floating-point value.
    pub mass_random: f32,
    /// Mass size multiplier parameter stored as a floating-point value.
    pub mass_size_multiplier: f32,
    /// Mask selecting local force channels.
    pub local_forces: LocalForces,
    /// Mask selecting world force channels.
    pub world_forces: WorldForces,
    /// Fallback mask for local force channels.
    pub local_forces_fallback: LocalForces,
    /// Fallback mask for world force channels.
    pub world_forces_fallback: WorldForces,
    /// World forces mass multiplier parameter stored as a floating-point value.
    pub world_forces_mass_multiplier: f32,
    /// Noise amplitude parameter stored as a floating-point value.
    pub noise_amplitude: f32,
    /// Noise frequency parameter stored as a floating-point value.
    pub noise_frequency: f32,
    /// Noise coherence parameter stored as a floating-point value.
    pub noise_coherence: f32,
    /// Noise edge parameter stored as a floating-point value.
    pub noise_edge: f32,
    /// Index plus length selector; sentinel and unrecognized values are retained.
    pub index_plus_length: u32,
    /// Stored upper bound on the particle population.
    pub max_particles: u32,
    /// Particle emission-rate control; its stored source-format representation is retained.
    pub emission_rate: Property<f32>,
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
    /// Particle count emitted by a burst; animated changes use step selection.
    pub squirt_amount: Property<u16>,
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
}

/// Editable M3 RibbonSplinePoint values.
#[derive(Clone, Debug, PartialEq)]
pub struct SplinePoint {
    /// Node index anchoring this ribbon spline point.
    pub node: NodeId,
    /// Emission-position offset at this spline point.
    pub emission_offset: [f32; 3],
    /// Emission direction vector at this spline point.
    pub emission_vector: [f32; 3],
    /// Animated velocity control at this spline point.
    pub velocity: Property<f32>,
    /// Unresolved reserved data retained in its original position and representation.
    pub reserved: u32,
    /// Animated velocity multiplier at the spline base.
    pub velocity_base_factor: Property<f32>,
    /// Animated velocity multiplier at the spline end.
    pub velocity_end_factor: Property<f32>,
    /// Yaw variation variation settings, including the function, amplitude, and frequency.
    pub yaw_variation: VariationControl,
    /// Pitch variation variation settings, including the function, amplitude, and frequency.
    pub pitch_variation: VariationControl,
    /// Velocity variation variation settings, including the function, amplitude, and frequency.
    pub velocity_variation: VariationControl,
    /// Animated yaw control at this spline point.
    pub yaw: Property<f32>,
    /// Animated pitch control at this spline point.
    pub pitch: Property<f32>,
    /// Emission vector norm factor parameter stored as a floating-point value.
    pub emission_vector_norm_factor: f32,
    /// Velocity norm factor parameter stored as a floating-point value.
    pub velocity_norm_factor: f32,
}

/// Editable M3 RibbonEmitterPayload values.
#[derive(Clone, Debug, PartialEq)]
pub struct Ribbon<'a> {
    /// Secondary bone used by the authored ribbon fallback attachment.
    pub fallback_node: NodeId,
    /// Material drawn by this emitter.
    pub material: Option<MaterialId>,
    /// Supplementary StarCraft II emitter flags; unknown bits remain available.
    pub additional_flags: RibbonAdditionalFlags,
    /// Initial particle-speed property, including its unanimated and reset values.
    pub initial_speed: Property<f32>,
    /// Randomized initial-speed property in the source emitter.
    pub initial_speed_random: Property<f32>,
    /// Initial yaw property for emitted particles.
    pub initial_yaw: Property<f32>,
    /// Initial pitch property for emitted particles.
    pub initial_pitch: Property<f32>,
    /// Initial horizontal emission property in the StarCraft II emitter frame.
    pub initial_horizontal: Property<f32>,
    /// Initial vertical emission property in the StarCraft II emitter frame.
    pub initial_vertical: Property<f32>,
    /// Particle lifetime control retained from the source emitter.
    pub lifetime: Property<f32>,
    /// Random contribution to the particle-lifetime property.
    pub lifetime_random: Property<f32>,
    /// Source ribbon kill-radius word; its integer encoding is retained without assigning distance units.
    pub kill_radius: u32,
    /// Gravity x parameter stored as a floating-point value.
    pub gravity_x: f32,
    /// Gravity y parameter stored as a floating-point value.
    pub gravity_y: f32,
    /// Gravity control retained from the source emitter.
    pub gravity: f32,
    /// Normalized particle age at the intermediate size control point.
    pub size_mid_time: f32,
    /// Normalized particle age at the intermediate color control point.
    pub color_mid_time: f32,
    /// Normalized particle age at the intermediate alpha control point.
    pub alpha_mid_time: f32,
    /// Normalized particle age at the intermediate rotation control point.
    pub rotation_mid_time: f32,
    /// Size mid hold time parameter; units follow the enclosing animation or effect.
    pub size_mid_hold_time: f32,
    /// Color mid hold time parameter; units follow the enclosing animation or effect.
    pub color_mid_hold_time: f32,
    /// Alpha mid hold time parameter; units follow the enclosing animation or effect.
    pub alpha_mid_hold_time: f32,
    /// Rotation mid hold time parameter; units follow the enclosing animation or effect.
    pub rotation_mid_hold_time: f32,
    /// Three-component particle-size property.
    pub size_animation: Property<[f32; 3]>,
    /// Three-component particle-rotation property.
    pub rotation_animation: Property<[f32; 3]>,
    /// Color property at the beginning of particle life.
    pub color_start: Property<[f32; 4]>,
    /// Color property at the intermediate particle-age point.
    pub color_mid: Property<[f32; 4]>,
    /// Color property at the end of particle life.
    pub color_end: Property<[f32; 4]>,
    /// Drag applied to particle motion.
    pub drag: f32,
    /// Mass parameter stored as a floating-point value.
    pub mass: f32,
    /// Mass random parameter stored as a floating-point value.
    pub mass_random: f32,
    /// Mass size multiplier parameter stored as a floating-point value.
    pub mass_size_multiplier: f32,
    /// Mask selecting local force channels.
    pub local_forces: LocalForces,
    /// Mask selecting world force channels.
    pub world_forces: WorldForces,
    /// Fallback mask for local force channels.
    pub local_forces_fallback: LocalForces,
    /// Fallback mask for world force channels.
    pub world_forces_fallback: WorldForces,
    /// World forces mass multiplier parameter stored as a floating-point value.
    pub world_forces_mass_multiplier: f32,
    /// Noise amplitude parameter stored as a floating-point value.
    pub noise_amplitude: f32,
    /// Noise frequency parameter stored as a floating-point value.
    pub noise_frequency: f32,
    /// Noise coherence parameter stored as a floating-point value.
    pub noise_coherence: f32,
    /// Noise edge parameter stored as a floating-point value.
    pub noise_edge: f32,
    /// Index plus length selector; sentinel and unrecognized values are retained.
    pub index_plus_length: u32,
    /// Shape from which this StarCraft II emitter produces particles.
    pub emitter_shape: u32,
    /// StarCraft II ribbon geometry mode.
    pub ribbon_type: RibbonKind,
    /// Divisions parameter stored as a floating-point value.
    pub divisions: f32,
    /// Ribbon edge-count control.
    pub edges: u32,
    /// Inner radius control of the emitter’s source shape.
    pub inner_radius: f32,
    /// Animated maximum-length control for the ribbon.
    pub max_length: Property<f32>,
    /// Authored spline control points defining the emission curve.
    pub spline_points: Cow<'a, [SplinePoint]>,
    /// Animated ribbon activation control.
    pub active: Property<u32>,
    /// Stored state bits; unknown bits are retained.
    pub flags: RibbonFlags,
    /// Interpolation mode for the particle size controls.
    pub size_smoothing: Smoothing,
    /// Interpolation mode for the particle color controls.
    pub color_smoothing: Smoothing,
    /// Particle collision friction parameter.
    pub friction: f32,
    /// Particle collision bounce parameter.
    pub bounce: f32,
    /// Stored level-of-detail reduction control.
    pub lod_reduce: u32,
    /// Stored level-of-detail cutoff control.
    pub lod_cut: u32,
    /// Yaw variation settings, including the function, amplitude, and frequency.
    pub yaw: VariationControl,
    /// Pitch variation settings, including the function, amplitude, and frequency.
    pub pitch: VariationControl,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: VariationControl,
    /// Size variation settings, including the function, amplitude, and frequency.
    pub size: VariationControl,
    /// Opacity control for the emitted geometry.
    pub alpha: VariationControl,
    /// Animated particle-velocity control retained in the StarCraft II representation.
    pub particle_velocity: Property<f32>,
    /// Animated ribbon overlay control; its detailed source semantics are unresolved.
    pub overlay: Property<f32>,
    /// Unresolved deprecated unknown data retained in its original position and representation.
    pub deprecated_unknown: i32,
}
