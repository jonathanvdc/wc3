//! Typed StarCraft emitter destinations, independent of WEM selectors.
use crate::model::semantic::{FeatureId, PropertyKey};
/// Animated initial speed.
pub const INITIAL_SPEED: PropertyKey<FeatureId, f32> = PropertyKey::new("initial_speed");
/// Animated initial speed random.
pub const INITIAL_SPEED_RANDOM: PropertyKey<FeatureId, f32> =
    PropertyKey::new("initial_speed_random");
/// Animated initial yaw.
pub const INITIAL_YAW: PropertyKey<FeatureId, f32> = PropertyKey::new("initial_yaw");
/// Animated initial pitch.
pub const INITIAL_PITCH: PropertyKey<FeatureId, f32> = PropertyKey::new("initial_pitch");
/// Animated initial horizontal.
pub const INITIAL_HORIZONTAL: PropertyKey<FeatureId, f32> = PropertyKey::new("initial_horizontal");
/// Animated initial vertical.
pub const INITIAL_VERTICAL: PropertyKey<FeatureId, f32> = PropertyKey::new("initial_vertical");
/// Animated lifetime.
pub const LIFETIME: PropertyKey<FeatureId, f32> = PropertyKey::new("lifetime");
/// Animated lifetime random.
pub const LIFETIME_RANDOM: PropertyKey<FeatureId, f32> = PropertyKey::new("lifetime_random");
/// Animated size animation.
pub const SIZE_ANIMATION: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("size_animation");
/// Animated rotation animation.
pub const ROTATION_ANIMATION: PropertyKey<FeatureId, [f32; 3]> =
    PropertyKey::new("rotation_animation");
/// Animated color start.
pub const COLOR_START: PropertyKey<FeatureId, [f32; 4]> = PropertyKey::new("color_start");
/// Animated color mid.
pub const COLOR_MID: PropertyKey<FeatureId, [f32; 4]> = PropertyKey::new("color_mid");
/// Animated color end.
pub const COLOR_END: PropertyKey<FeatureId, [f32; 4]> = PropertyKey::new("color_end");
/// Animated emission rate.
pub const EMISSION_RATE: PropertyKey<FeatureId, f32> = PropertyKey::new("emission_rate");
/// Animated shape outer.
pub const SHAPE_OUTER: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("shape_outer");
/// Animated shape inner.
pub const SHAPE_INNER: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("shape_inner");
/// Animated outer radius.
pub const OUTER_RADIUS: PropertyKey<FeatureId, f32> = PropertyKey::new("outer_radius");
/// Animated inner radius.
pub const INNER_RADIUS: PropertyKey<FeatureId, f32> = PropertyKey::new("inner_radius");
/// Animated size random animation.
pub const SIZE_RANDOM_ANIMATION: PropertyKey<FeatureId, [f32; 3]> =
    PropertyKey::new("size_random_animation");
/// Animated rotation random animation.
pub const ROTATION_RANDOM_ANIMATION: PropertyKey<FeatureId, [f32; 3]> =
    PropertyKey::new("rotation_random_animation");
/// Animated color start random.
pub const COLOR_START_RANDOM: PropertyKey<FeatureId, [f32; 4]> =
    PropertyKey::new("color_start_random");
/// Animated color mid random.
pub const COLOR_MID_RANDOM: PropertyKey<FeatureId, [f32; 4]> = PropertyKey::new("color_mid_random");
/// Animated color end random.
pub const COLOR_END_RANDOM: PropertyKey<FeatureId, [f32; 4]> = PropertyKey::new("color_end_random");
/// Animated squirt amount.
pub const SQUIRT_AMOUNT: PropertyKey<FeatureId, u32> = PropertyKey::new("squirt_amount");
/// Animated pitch.amplitude.
pub const PITCH_AMPLITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("pitch.amplitude");
/// Animated pitch.frequency.
pub const PITCH_FREQUENCY: PropertyKey<FeatureId, f32> = PropertyKey::new("pitch.frequency");
/// Animated yaw.amplitude.
pub const YAW_AMPLITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("yaw.amplitude");
/// Animated yaw.frequency.
pub const YAW_FREQUENCY: PropertyKey<FeatureId, f32> = PropertyKey::new("yaw.frequency");
/// Animated speed.amplitude.
pub const SPEED_AMPLITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("speed.amplitude");
/// Animated speed.frequency.
pub const SPEED_FREQUENCY: PropertyKey<FeatureId, f32> = PropertyKey::new("speed.frequency");
/// Animated size.amplitude.
pub const SIZE_AMPLITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("size.amplitude");
/// Animated size.frequency.
pub const SIZE_FREQUENCY: PropertyKey<FeatureId, f32> = PropertyKey::new("size.frequency");
/// Animated alpha.amplitude.
pub const ALPHA_AMPLITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("alpha.amplitude");
/// Animated alpha.frequency.
pub const ALPHA_FREQUENCY: PropertyKey<FeatureId, f32> = PropertyKey::new("alpha.frequency");
/// Animated color.amplitude.
pub const COLOR_AMPLITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("color.amplitude");
/// Animated color.frequency.
pub const COLOR_FREQUENCY: PropertyKey<FeatureId, f32> = PropertyKey::new("color.frequency");
/// Animated rotation.amplitude.
pub const ROTATION_AMPLITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("rotation.amplitude");
/// Animated rotation.frequency.
pub const ROTATION_FREQUENCY: PropertyKey<FeatureId, f32> = PropertyKey::new("rotation.frequency");
/// Animated horizontal.amplitude.
pub const HORIZONTAL_AMPLITUDE: PropertyKey<FeatureId, f32> =
    PropertyKey::new("horizontal.amplitude");
/// Animated horizontal.frequency.
pub const HORIZONTAL_FREQUENCY: PropertyKey<FeatureId, f32> =
    PropertyKey::new("horizontal.frequency");
/// Animated vertical.amplitude.
pub const VERTICAL_AMPLITUDE: PropertyKey<FeatureId, f32> = PropertyKey::new("vertical.amplitude");
/// Animated vertical.frequency.
pub const VERTICAL_FREQUENCY: PropertyKey<FeatureId, f32> = PropertyKey::new("vertical.frequency");
/// Animated particle velocity.
pub const PARTICLE_VELOCITY: PropertyKey<FeatureId, f32> = PropertyKey::new("particle_velocity");
/// Animated phase shift.
pub const PHASE_SHIFT: PropertyKey<FeatureId, f32> = PropertyKey::new("phase_shift");
/// Animated alpha threshold.
pub const ALPHA_THRESHOLD: PropertyKey<FeatureId, f32> = PropertyKey::new("alpha_threshold");
/// Animated uv offset.
pub const UV_OFFSET: PropertyKey<FeatureId, [f32; 2]> = PropertyKey::new("uv_offset");
/// Animated uv angle.
pub const UV_ANGLE: PropertyKey<FeatureId, [f32; 3]> = PropertyKey::new("uv_angle");
/// Animated uv tiling.
pub const UV_TILING: PropertyKey<FeatureId, [f32; 2]> = PropertyKey::new("uv_tiling");
/// Animated lower bound.
pub const LOWER_BOUND: PropertyKey<FeatureId, f32> = PropertyKey::new("lower_bound");
/// Animated upper bound.
pub const UPPER_BOUND: PropertyKey<FeatureId, f32> = PropertyKey::new("upper_bound");
/// Animated trail emission rate.
pub const TRAIL_EMISSION_RATE: PropertyKey<FeatureId, f32> =
    PropertyKey::new("trail_emission_rate");
/// Animated max length.
pub const MAX_LENGTH: PropertyKey<FeatureId, f32> = PropertyKey::new("max_length");
/// Animated active.
pub const ACTIVE: PropertyKey<FeatureId, u32> = PropertyKey::new("active");
/// Animated overlay.
pub const OVERLAY: PropertyKey<FeatureId, f32> = PropertyKey::new("overlay");

/// Indexed spline-point destinations on an emitter.
#[derive(Clone, Copy, Debug)]
pub struct SplinePoint(pub usize);
impl SplinePoint {
    /// Animated point position.
    pub fn position(self) -> PropertyKey<FeatureId, [f32; 3]> {
        PropertyKey::indexed(format!("spline_points[{}]", self.0))
    }
    /// Animated point velocity.
    pub fn velocity(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!("spline_points[{}].velocity", self.0))
    }
    /// Animated point velocity base factor.
    pub fn velocity_base_factor(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!("spline_points[{}].velocity_base_factor", self.0))
    }
    /// Animated point velocity end factor.
    pub fn velocity_end_factor(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!("spline_points[{}].velocity_end_factor", self.0))
    }
    /// Animated point yaw variation.amplitude.
    pub fn yaw_variation_amplitude(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!("spline_points[{}].yaw_variation.amplitude", self.0))
    }
    /// Animated point yaw variation.frequency.
    pub fn yaw_variation_frequency(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!("spline_points[{}].yaw_variation.frequency", self.0))
    }
    /// Animated point pitch variation.amplitude.
    pub fn pitch_variation_amplitude(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!(
            "spline_points[{}].pitch_variation.amplitude",
            self.0
        ))
    }
    /// Animated point pitch variation.frequency.
    pub fn pitch_variation_frequency(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!(
            "spline_points[{}].pitch_variation.frequency",
            self.0
        ))
    }
    /// Animated point velocity variation.amplitude.
    pub fn velocity_variation_amplitude(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!(
            "spline_points[{}].velocity_variation.amplitude",
            self.0
        ))
    }
    /// Animated point velocity variation.frequency.
    pub fn velocity_variation_frequency(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!(
            "spline_points[{}].velocity_variation.frequency",
            self.0
        ))
    }
    /// Animated point yaw.
    pub fn yaw(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!("spline_points[{}].yaw", self.0))
    }
    /// Animated point pitch.
    pub fn pitch(self) -> PropertyKey<FeatureId, f32> {
        PropertyKey::indexed(format!("spline_points[{}].pitch", self.0))
    }
}
