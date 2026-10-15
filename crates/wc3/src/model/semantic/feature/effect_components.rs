//! Shared controls for programmed particle and ribbon simulations.
use crate::model::semantic::profiles::m3::{values::*, Property};
/// Programmed effect Age controls.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammedAge {
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
}
/// Programmed effect Forces controls.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammedForces {
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
}
/// Programmed effect Noise controls.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammedNoise {
    /// Noise amplitude parameter stored as a floating-point value.
    pub noise_amplitude: f32,
    /// Noise frequency parameter stored as a floating-point value.
    pub noise_frequency: f32,
    /// Noise coherence parameter stored as a floating-point value.
    pub noise_coherence: f32,
    /// Noise edge parameter stored as a floating-point value.
    pub noise_edge: f32,
}
/// Initial directional controls and drag shared by programmed effects.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgrammedDirection {
    /// Initial yaw.
    pub initial_yaw: Property<f32>,
    /// Initial pitch.
    pub initial_pitch: Property<f32>,
    /// Horizontal emission control in the effect frame.
    pub initial_horizontal: Property<f32>,
    /// Vertical emission control in the effect frame.
    pub initial_vertical: Property<f32>,
    /// Velocity drag.
    pub drag: f32,
}

use crate::model::semantic::{Link, ValidateLinks};
impl ValidateLinks for ProgrammedAge {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        if !self.size_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.color_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.alpha_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.rotation_mid_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.size_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.color_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.alpha_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.rotation_mid_hold_time.is_finite() {
            v(Link::InvalidValue);
        }
        self.size_animation.visit_links(v);
        self.rotation_animation.visit_links(v);
        self.color_start.visit_links(v);
        self.color_mid.visit_links(v);
        self.color_end.visit_links(v);
    }
}
impl ValidateLinks for ProgrammedForces {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        if !self.mass.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.mass_random.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.mass_size_multiplier.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.world_forces_mass_multiplier.is_finite() {
            v(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for ProgrammedNoise {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        if !self.noise_amplitude.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_frequency.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_coherence.is_finite() {
            v(Link::InvalidValue);
        }
        if !self.noise_edge.is_finite() {
            v(Link::InvalidValue);
        }
    }
}
impl ValidateLinks for ProgrammedDirection {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.initial_yaw.visit_links(v);
        self.initial_pitch.visit_links(v);
        self.initial_horizontal.visit_links(v);
        self.initial_vertical.visit_links(v);
        if !self.drag.is_finite() {
            v(Link::InvalidValue);
        }
    }
}
