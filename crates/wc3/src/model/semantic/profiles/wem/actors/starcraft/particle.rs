use super::super::AssetAddress;
use super::*;
use crate::model::semantic::{profiles::wem::SectionId, FeatureId, MaterialSlotId, Values};
/// Complete independently authored particle controls.
#[derive(Clone, Debug, Default)]
pub struct Particle<'a> {
    /// Appearance slot drawn by this effect.
    pub material_slot: Option<MaterialSlotId>,
    /// Supplementary StarCraft II emitter flags; unknown bits remain available.
    pub additional_flags: ParticleAdditionalFlags,
    /// Initial particle-speed property, including its unanimated and reset values.
    pub initial_speed: Control<f32>,
    /// Randomized initial-speed property in the source emitter.
    pub initial_speed_random: Control<f32>,
    /// Initial yaw property for emitted particles.
    pub initial_yaw: Control<f32>,
    /// Initial pitch property for emitted particles.
    pub initial_pitch: Control<f32>,
    /// Initial horizontal emission property in the StarCraft II emitter frame.
    pub initial_horizontal: Control<f32>,
    /// Initial vertical emission property in the StarCraft II emitter frame.
    pub initial_vertical: Control<f32>,
    /// Particle lifetime control retained from the source emitter.
    pub lifetime: Control<f32>,
    /// Random contribution to the particle-lifetime property.
    pub lifetime_random: Control<f32>,
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
    /// Size hold interval around the intermediate particle-age control.
    pub size_mid_hold_time: f32,
    /// Color hold interval around the intermediate particle-age control.
    pub color_mid_hold_time: f32,
    /// Alpha hold interval around the intermediate particle-age control.
    pub alpha_mid_hold_time: f32,
    /// Rotation hold interval around the intermediate particle-age control.
    pub rotation_mid_hold_time: f32,
    /// Three-component particle-size property.
    pub size_animation: Control<[f32; 3]>,
    /// Three-component particle-rotation property.
    pub rotation_animation: Control<[f32; 3]>,
    /// Color property at the beginning of particle life.
    pub color_start: Control<[f32; 4]>,
    /// Color property at the intermediate particle-age point.
    pub color_mid: Control<[f32; 4]>,
    /// Color property at the end of particle life.
    pub color_end: Control<[f32; 4]>,
    /// Drag applied to particle motion.
    pub drag: f32,
    /// Particle mass used by force response.
    pub mass: f32,
    /// Random contribution to particle mass.
    pub mass_random: f32,
    /// Contribution of particle size to mass.
    pub mass_size_multiplier: f32,
    /// Mask selecting local force channels.
    pub local_forces: LocalForceChannels,
    /// Mask selecting world force channels.
    pub world_forces: WorldForceChannels,
    /// Fallback mask for local force channels.
    pub local_forces_fallback: LocalForceChannels,
    /// Fallback mask for world force channels.
    pub world_forces_fallback: WorldForceChannels,
    /// Mass multiplier applied when receiving world forces.
    pub world_forces_mass_multiplier: f32,
    /// Amplitude of motion noise.
    pub noise_amplitude: f32,
    /// Spatial frequency of motion noise.
    pub noise_frequency: f32,
    /// Correlation control for neighboring motion-noise samples.
    pub noise_coherence: f32,
    /// Transition control at motion-noise boundaries.
    pub noise_edge: f32,
    /// Index plus length selector; sentinel and unrecognized values are retained.
    pub index_plus_length: u32,
    /// Stored upper bound on the particle population.
    pub max_particles: u32,
    /// Particle emission-rate control; its stored source-format representation is retained.
    pub emission_rate: Control<f32>,
    /// Shape from which this StarCraft II emitter produces particles.
    pub emitter_shape: EmitterShape,
    /// Dimensions of the outer emitter shape.
    pub shape_outer: Control<[f32; 3]>,
    /// Dimensions of the inner emitter shape.
    pub shape_inner: Control<[f32; 3]>,
    /// Outer radius control of the emitter’s source shape.
    pub outer_radius: Control<f32>,
    /// Inner radius control of the emitter’s source shape.
    pub inner_radius: Control<f32>,
    /// Actual first-mesh draw sections supplying emission geometry.
    pub shape_sections: Values<'a, SectionId>,
    /// Emission direction used to initialize particle velocity.
    pub velocity_type: EmissionDirection,
    /// Source control word enabling size randomization.
    pub size_random_enable: u32,
    /// Random contribution to the three-component size property.
    pub size_random_animation: Control<[f32; 3]>,
    /// Source control word enabling rotation randomization.
    pub rotation_random_enable: u32,
    /// Random contribution to the three-component rotation property.
    pub rotation_random_animation: Control<[f32; 3]>,
    /// Source control word enabling color randomization.
    pub color_random_enable: u32,
    /// Random color contribution at the beginning of particle life.
    pub color_start_random: Control<[f32; 4]>,
    /// Random color contribution at the intermediate age point.
    pub color_mid_random: Control<[f32; 4]>,
    /// Random color contribution at the end of particle life.
    pub color_end_random: Control<[f32; 4]>,
    /// Source control word enabling alpha randomization.
    pub alpha_random_enable: u32,
    /// Particle count emitted by a burst; animated changes use step selection.
    pub squirt_amount: Control<u32>,
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
    /// Flipbook column fraction parameter stored as a floating-point value.
    pub flipbook_column_fraction: f32,
    /// Flipbook row fraction parameter stored as a floating-point value.
    pub flipbook_row_fraction: f32,
    /// Particle collision bounce parameter.
    pub bounce: f32,
    /// Particle collision friction parameter.
    pub friction: f32,
    /// Actual linked emitter behavior.
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
    pub pitch: Variation,
    /// Yaw variation settings, including the function, amplitude, and frequency.
    pub yaw: Variation,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: Variation,
    /// Size variation settings, including the function, amplitude, and frequency.
    pub size: Variation,
    /// Opacity control for the emitted geometry.
    pub alpha: Variation,
    /// Color variation settings, including the function, amplitude, and frequency.
    pub color: Variation,
    /// Rotation variation settings, including the function, amplitude, and frequency.
    pub rotation: Variation,
    /// Horizontal variation settings, including the function, amplitude, and frequency.
    pub horizontal: Variation,
    /// Vertical variation settings, including the function, amplitude, and frequency.
    pub vertical: Variation,
    /// Animated particle-velocity control retained in the StarCraft II representation.
    pub particle_velocity: Control<f32>,
    /// Animated phase-shift control; its effect-specific interpretation is unresolved.
    pub phase_shift: Control<f32>,
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
    pub alpha_threshold: Control<f32>,
    /// Animated offset of particle texture coordinates.
    pub uv_offset: Control<[f32; 2]>,
    /// Animated three-component texture-orientation property.
    pub uv_angle: Control<[f32; 3]>,
    /// Animated tiling factors for the two texture-coordinate axes.
    pub uv_tiling: Control<[f32; 2]>,
    /// Authored spline positions and reset values.
    pub spline_points: Values<'a, Control<[f32; 3]>>,
    /// Wind multiplier parameter stored as a floating-point value.
    pub wind_multiplier: f32,
    /// Stored level-of-detail reduction control.
    pub lod_reduce: u32,
    /// Stored level-of-detail cutoff control.
    pub lod_cut: u32,
    /// Animated lower-bound control; its source effect semantics are unresolved.
    pub lower_bound: Control<f32>,
    /// Animated upper-bound control; its source effect semantics are unresolved.
    pub upper_bound: Control<f32>,
    /// Actual linked emitter behavior.
    pub trail_link: Option<FeatureId>,
    /// Trail chance parameter stored as a floating-point value.
    pub trail_chance: f32,
    /// Animated emission-rate control for the linked trail emitter.
    pub trail_emission_rate: Control<f32>,
    /// Splat projection index selector; sentinel and unrecognized values are retained.
    pub splat_projection_index: i32,
    /// Splat chance parameter stored as a floating-point value.
    pub splat_chance: f32,
    /// Independently addressed model particles.
    pub models: Values<'a, AssetAddress<'a>>,
    /// Spawn ribbon on bounce chance parameter stored as a floating-point value.
    pub spawn_ribbon_on_bounce_chance: f32,
    /// Actual linked emitter behavior.
    pub ribbon_link: Option<FeatureId>,
    /// Legacy motion-noise smoothing control.
    pub noise_smoothness: f32,
}
