//! World of Warcraft particle declarations retain their own editable-model vocabulary.
use super::AssetAddress;
use crate::model::semantic::{Animated, Link, TextureId, ValidateLinks, Values};
use bitfield::bitfield;
bitfield! {
    /// Particle state retains producer-defined bits without guessed behavior names.
    #[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
    pub struct ParticleFlags(u32);
    /// Retained state bits.
    pub bits,_:31,0;
}
impl ParticleFlags {
    /// Retains the complete producer bit pattern.
    pub const fn from_bits_retain(bits: u32) -> Self {
        Self(bits)
    }
}
/// Authored ParticleBlend convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleBlend {
    /// Opaque convention.
    Opaque,
    /// Alpha convention.
    Alpha,
    /// Additive convention.
    Additive,
    /// AlphaKey convention.
    AlphaKey,
    /// AdditiveAlphaKey convention.
    AdditiveAlphaKey,
    /// Producer-defined convention.
    Unknown(u32),
}
/// Authored EmitterShape convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmitterShape {
    /// Plane convention.
    Plane,
    /// Sphere convention.
    Sphere,
    /// Spline convention.
    Spline,
    /// Bone convention.
    Bone,
    /// Producer-defined convention.
    Unknown(u32),
}
/// Authored particle parameters; channel initialization remains independently owned.
#[derive(Clone, Debug)]
pub struct Particle<'a> {
    /// M2 particle-emitter identifier; negative values are retained.
    pub particle_id: i32,
    /// Stored state bits; unknown bits are retained.
    pub flags: ParticleFlags,
    /// Shared texture identity used by the emitter.
    pub texture: Option<TextureId>,
    /// Second shared texture identity, used by multi-texture emitters.
    pub texture2: Option<TextureId>,
    /// Third shared texture identity, used by multi-texture emitters.
    pub texture3: Option<TextureId>,
    /// M2 blend-mode number, independent of Warcraft III particle blending modes.
    pub blend: ParticleBlend,
    /// M2 emitter shape code: 1 plane, 2 sphere, 3 spline, or 4 bone.
    pub emitter_type: EmitterShape,
    /// Color index selector; sentinel and unrecognized values are retained.
    pub color_index: u32,
    /// Number of rows in the particle texture atlas.
    pub rows: u32,
    /// Number of columns in the particle texture atlas.
    pub columns: u32,
    /// Signed drawing-priority plane of the emitter.
    pub priority_plane: i32,
    /// Initial particle-speed control retained from the source emitter.
    pub speed: Animated<f32>,
    /// Speed variation parameter stored as a floating-point value.
    pub speed_variation: Animated<f32>,
    /// Vertical range parameter stored as a floating-point value.
    pub vertical_range: Animated<f32>,
    /// Horizontal range parameter stored as a floating-point value.
    pub horizontal_range: Animated<f32>,
    /// Gravity control retained from the source emitter.
    pub gravity: Animated<[f32; 3]>,
    /// Lifetime of an emitted particle, in seconds.
    pub lifespan: Animated<f32>,
    /// Lifespan variation parameter stored as a floating-point value.
    pub lifespan_variation: f32,
    /// Particle emission-rate control; its stored source-format representation is retained.
    pub emission_rate: Animated<f32>,
    /// Emission rate variation parameter stored as a floating-point value.
    pub emission_rate_variation: f32,
    /// Width parameter stored as a floating-point value.
    pub width: Animated<f32>,
    /// Length parameter stored as a floating-point value.
    pub length: Animated<f32>,
    /// Z source parameter stored as a floating-point value.
    pub z_source: Animated<f32>,
    /// Particle-age positions for the color control points.
    pub color_times: Values<'a, f32>,
    /// Authored colors.
    pub colors: Values<'a, [f32; 3]>,
    /// Particle-age positions for the alpha control points.
    pub alpha_times: Values<'a, f32>,
    /// Authored alphas.
    pub alphas: Values<'a, f32>,
    /// Particle-age positions for the scale control points.
    pub scale_times: Values<'a, f32>,
    /// Authored scales.
    pub scales: Values<'a, [f32; 2]>,
    /// Variation in the two components of the particle scale.
    pub scale_variation: [f32; 2],
    /// Particle-age positions for the head cell control points.
    pub head_cell_times: Values<'a, f32>,
    /// Authored head cells.
    pub head_cells: Values<'a, u32>,
    /// Particle-age positions for the tail cell control points.
    pub tail_cell_times: Values<'a, f32>,
    /// Authored tail cells.
    pub tail_cells: Values<'a, u32>,
    /// Length control for the particle trail.
    pub tail_length: f32,
    /// Twinkle speed parameter stored as a floating-point value.
    pub twinkle_speed: f32,
    /// Twinkle percent parameter stored as a floating-point value.
    pub twinkle_percent: f32,
    /// Two-component particle-scale control for the twinkle effect; component interpretation is unresolved.
    pub twinkle_scale: [f32; 2],
    /// Drag applied to particle motion.
    pub drag: f32,
    /// Base spin parameter stored as a floating-point value.
    pub base_spin: f32,
    /// Base spin variation parameter stored as a floating-point value.
    pub base_spin_variation: f32,
    /// Spin parameter stored as a floating-point value.
    pub spin: f32,
    /// Spin variation parameter stored as a floating-point value.
    pub spin_variation: f32,
    /// Wind vector applied to particle motion.
    pub wind: [f32; 3],
    /// Wind time parameter; units follow the enclosing animation or effect.
    pub wind_time: f32,
    /// Authored spline points.
    pub spline_points: Values<'a, [f32; 3]>,
    /// Texture2 scale parameter stored as a floating-point value.
    pub texture2_scale: Option<f32>,
    /// Texture3 scale parameter stored as a floating-point value.
    pub texture3_scale: Option<f32>,
    /// Central UV scrolling rate of texture layer 2, in texture-coordinate units per second.
    pub texture2_scroll_mid: Option<[f32; 2]>,
    /// Central UV scrolling rate of texture layer 3, in texture-coordinate units per second.
    pub texture3_scroll_mid: Option<[f32; 2]>,
    /// Random spread around the UV scrolling rate of texture layer 2, in texture-coordinate units per second.
    pub texture2_scroll_range: Option<[f32; 2]>,
    /// Random spread around the UV scrolling rate of texture layer 3, in texture-coordinate units per second.
    pub texture3_scroll_range: Option<[f32; 2]>,
    /// Inherit velocity scale parameter stored as a floating-point value.
    pub inherit_velocity_scale: Option<f32>,
    /// Follow speed1 parameter stored as a floating-point value.
    pub follow_speed1: Option<f32>,
    /// Follow scale1 parameter stored as a floating-point value.
    pub follow_scale1: Option<f32>,
    /// Follow speed2 parameter stored as a floating-point value.
    pub follow_speed2: Option<f32>,
    /// Follow scale2 parameter stored as a floating-point value.
    pub follow_scale2: Option<f32>,
    /// Lower per-axis bound on model-particle angular velocity, in radians per second.
    pub tumble_min: Option<[f32; 3]>,
    /// Upper per-axis bound on model-particle angular velocity, in radians per second.
    pub tumble_max: Option<[f32; 3]>,
    /// Color mult parameter stored as a floating-point value.
    pub color_mult: Option<f32>,
    /// Alpha mult parameter stored as a floating-point value.
    pub alpha_mult: Option<f32>,
    /// Particle-age positions for the alpha cutoff control points.
    pub alpha_cutoff_times: Option<Values<'a, f32>>,
    /// Authored alpha cutoffs.
    pub alpha_cutoffs: Option<Values<'a, f32>>,
    /// M2 asset drawn for each particle instead of a textured quad.
    pub particle_model: Option<AssetAddress<'a>>,
    /// M2 asset supplying child emitters that trail the particles.
    pub trail_model: Option<AssetAddress<'a>>,
}
impl ValidateLinks for Particle<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        if matches!(self.blend, ParticleBlend::Unknown(0..=4))
            || matches!(self.emitter_type, EmitterShape::Unknown(1..=4))
        {
            visitor(Link::InvalidValue);
        }
        for values in [&self.colors, &self.spline_points] {
            if values.iter().any(|v| v.iter().any(|c| !c.is_finite())) {
                visitor(Link::InvalidValue);
            }
        }
        if self.scales.iter().any(|v| v.iter().any(|c| !c.is_finite()))
            || self.alphas.iter().any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        match (&self.alpha_cutoff_times, &self.alpha_cutoffs) {
            (Some(times), Some(values))
                if times.len() == values.len()
                    && times.iter().chain(values.iter()).all(|v| v.is_finite()) => {}
            (None, None) => {}
            _ => visitor(Link::InvalidValue),
        }

        self.texture.visit_links(visitor);
        self.texture2.visit_links(visitor);
        self.texture3.visit_links(visitor);
        self.speed.visit_links(visitor);
        self.speed_variation.visit_links(visitor);
        self.vertical_range.visit_links(visitor);
        self.horizontal_range.visit_links(visitor);
        self.gravity.visit_links(visitor);
        self.lifespan.visit_links(visitor);
        if !self.lifespan_variation.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.emission_rate.visit_links(visitor);
        if !self.emission_rate_variation.is_finite() {
            visitor(Link::InvalidValue);
        }
        self.width.visit_links(visitor);
        self.length.visit_links(visitor);
        self.z_source.visit_links(visitor);
        if self.scale_variation.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if !self.tail_length.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.twinkle_speed.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.twinkle_percent.is_finite() {
            visitor(Link::InvalidValue);
        }
        if self.twinkle_scale.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if !self.drag.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.base_spin.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.base_spin_variation.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.spin.is_finite() {
            visitor(Link::InvalidValue);
        }
        if !self.spin_variation.is_finite() {
            visitor(Link::InvalidValue);
        }
        if self.wind.iter().any(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if !self.wind_time.is_finite() {
            visitor(Link::InvalidValue);
        }
        if self.texture2_scale.is_some_and(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.texture3_scale.is_some_and(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self
            .texture2_scroll_mid
            .is_some_and(|v| v.iter().any(|c| !c.is_finite()))
        {
            visitor(Link::InvalidValue);
        }
        if self
            .texture3_scroll_mid
            .is_some_and(|v| v.iter().any(|c| !c.is_finite()))
        {
            visitor(Link::InvalidValue);
        }
        if self
            .texture2_scroll_range
            .is_some_and(|v| v.iter().any(|c| !c.is_finite()))
        {
            visitor(Link::InvalidValue);
        }
        if self
            .texture3_scroll_range
            .is_some_and(|v| v.iter().any(|c| !c.is_finite()))
        {
            visitor(Link::InvalidValue);
        }
        if self.inherit_velocity_scale.is_some_and(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.follow_speed1.is_some_and(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.follow_scale1.is_some_and(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.follow_speed2.is_some_and(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.follow_scale2.is_some_and(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self
            .tumble_min
            .is_some_and(|v| v.iter().any(|c| !c.is_finite()))
        {
            visitor(Link::InvalidValue);
        }
        if self
            .tumble_max
            .is_some_and(|v| v.iter().any(|c| !c.is_finite()))
        {
            visitor(Link::InvalidValue);
        }
        if self.color_mult.is_some_and(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        if self.alpha_mult.is_some_and(|v| !v.is_finite()) {
            visitor(Link::InvalidValue);
        }
        self.particle_model.visit_links(visitor);
        self.trail_model.visit_links(visitor);
        if self.color_times.len() != self.colors.len()
            || self.color_times.iter().any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        if self.alpha_times.len() != self.alphas.len()
            || self.alpha_times.iter().any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        if self.scale_times.len() != self.scales.len()
            || self.scale_times.iter().any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        if self.head_cell_times.len() != self.head_cells.len()
            || self.head_cell_times.iter().any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
        if self.tail_cell_times.len() != self.tail_cells.len()
            || self.tail_cell_times.iter().any(|v| !v.is_finite())
        {
            visitor(Link::InvalidValue);
        }
    }
}
