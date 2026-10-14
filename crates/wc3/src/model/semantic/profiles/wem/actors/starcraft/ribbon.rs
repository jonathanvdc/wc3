use super::*;
use crate::model::semantic::{MaterialSlotId, Values};
/// Complete independently authored ribbon controls.
#[derive(Clone, Debug, Default)]
pub struct Ribbon<'a> {
    /// Appearance slot drawn by this effect.
    pub material_slot: Option<MaterialSlotId>,
    /// Supplementary StarCraft II emitter flags; unknown bits remain available.
    pub additional_flags: RibbonAdditionalFlags,
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
    /// Source ribbon kill-radius word; its integer encoding is retained without assigning distance units.
    pub kill_radius: u32,
    /// Acceleration along the emitter-frame X axis.
    pub gravity_x: f32,
    /// Acceleration along the emitter-frame Y axis.
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
    /// Shape from which this StarCraft II emitter produces particles.
    pub emitter_shape: u32,
    /// StarCraft II ribbon geometry mode.
    pub ribbon_type: RibbonType,
    /// Divisions parameter stored as a floating-point value.
    pub divisions: f32,
    /// Ribbon edge-count control.
    pub edges: u32,
    /// Inner radius control of the emitter’s source shape.
    pub inner_radius: f32,
    /// Animated maximum-length control for the ribbon.
    pub max_length: Control<f32>,
    /// Independently editable node-anchored spline controls.
    pub spline_points: Values<'a, RibbonPoint>,
    /// Animated ribbon activation control.
    pub active: Control<u32>,
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
    pub yaw: Variation,
    /// Pitch variation settings, including the function, amplitude, and frequency.
    pub pitch: Variation,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: Variation,
    /// Size variation settings, including the function, amplitude, and frequency.
    pub size: Variation,
    /// Opacity control for the emitted geometry.
    pub alpha: Variation,
    /// Animated particle-velocity control retained in the StarCraft II representation.
    pub particle_velocity: Control<f32>,
    /// Animated ribbon overlay control; its detailed source semantics are unresolved.
    pub overlay: Control<f32>,
    /// Unresolved deprecated unknown data retained in its original position and representation.
    pub deprecated_unknown: i32,
}
