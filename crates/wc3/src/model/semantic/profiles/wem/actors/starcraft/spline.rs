use super::*;
use crate::model::semantic::NodeId;
/// Complete independently authored spline controls.
#[derive(Clone, Debug, Default)]
pub struct RibbonPoint {
    /// Hierarchy node anchoring the spline point.
    pub node: Option<NodeId>,
    /// Emission-position offset at this spline point.
    pub emission_offset: [f32; 3],
    /// Emission direction vector at this spline point.
    pub emission_vector: [f32; 3],
    /// Animated velocity control at this spline point.
    pub velocity: Control<f32>,
    /// Additional producer control whose meaning is unresolved.
    pub unresolved_control: u32,
    /// Animated velocity multiplier at the spline base.
    pub velocity_base_factor: Control<f32>,
    /// Animated velocity multiplier at the spline end.
    pub velocity_end_factor: Control<f32>,
    /// Yaw variation variation settings, including the function, amplitude, and frequency.
    pub yaw_variation: Variation,
    /// Pitch variation variation settings, including the function, amplitude, and frequency.
    pub pitch_variation: Variation,
    /// Velocity variation variation settings, including the function, amplitude, and frequency.
    pub velocity_variation: Variation,
    /// Animated yaw control at this spline point.
    pub yaw: Control<f32>,
    /// Animated pitch control at this spline point.
    pub pitch: Control<f32>,
    /// Emission vector norm factor parameter stored as a floating-point value.
    pub emission_vector_norm_factor: f32,
    /// Velocity norm factor parameter stored as a floating-point value.
    pub velocity_norm_factor: f32,
}
