//! Resource references, variation functions, and shared programmed effect controls.
use crate::model::semantic::animation::programmed::Property;
use crate::model::semantic::values::programmed::*;
use crate::model::semantic::{FeatureId, NodeId};
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
