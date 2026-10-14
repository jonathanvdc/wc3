//! Warcraft effect actors independent of their encoded records.
/// Typed destinations for emitter animation.
pub mod properties;
use super::AssetAddress;
use crate::model::semantic::{Animated, Link, MaterialSlotId, TextureId, ValidateLinks};
use std::borrow::Cow;
/// Blending operation of Warcraft sprite particles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleBlend {
    /// Standard alpha blending.
    AlphaBlend,
    /// Adds particle color to the destination.
    Additive,
    /// Multiplies destination color.
    Multiply,
    /// Multiplies destination color with doubled intensity.
    Multiply2,
    /// Discards pixels below the opacity threshold.
    AlphaTest,
    /// Uninterpreted producer-defined operation.
    Unknown(u32),
}
/// Particle components selected for drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticleGeometry {
    /// Draws heads.
    Head,
    /// Draws tails.
    Tail,
    /// Draws both heads and tails.
    Both,
    /// Uninterpreted producer-defined selection.
    Unknown(u32),
}
/// Appearance at one particle-age stage.
#[derive(Clone, Copy, Debug)]
pub struct LifeStage {
    /// RGB tint.
    pub color: [f32; 3],
    /// Opacity on the zero-to-255 scale.
    pub opacity: u8,
    /// Scale multiplier.
    pub scale: f32,
}
/// Inclusive texture-atlas interval and repetition count.
#[derive(Clone, Copy, Debug)]
pub struct FrameInterval {
    /// First cell.
    pub first: u32,
    /// Last cell.
    pub last: u32,
    /// Repetitions of the interval.
    pub repetitions: u32,
}
/// Authored resourceparticles parameters.
#[derive(Clone, Debug)]
pub struct ResourceParticles<'a> {
    /// Particles created per second.
    pub emission_rate: Animated<f32>,
    /// Acceleration applied to emitted particles.
    pub gravity: Animated<f32>,
    /// Horizontal emission direction.
    pub longitude: Animated<f32>,
    /// Vertical emission spread.
    pub latitude: Animated<f32>,
    /// Particle lifetime in seconds.
    pub lifespan: Animated<f32>,
    /// Initial particle speed.
    pub speed: Animated<f32>,
    /// Resource instantiated for each particle.
    pub spawn_model: AssetAddress<'a>,
    /// Instantiates the resource as model geometry.
    pub uses_mdl: bool,
    /// Uses the resource as an image particle.
    pub uses_tga: bool,
}
/// Authored spriteparticles parameters.
#[derive(Clone, Debug)]
pub struct SpriteParticles {
    /// Initial particle speed.
    pub speed: Animated<f32>,
    /// Variation of initial particle speed.
    pub variation: Animated<f32>,
    /// Vertical emission spread.
    pub latitude: Animated<f32>,
    /// Acceleration applied to particles.
    pub gravity: Animated<f32>,
    /// Particles created per second.
    pub emission_rate: Animated<f32>,
    /// Emitter width.
    pub width: Animated<f32>,
    /// Emitter length.
    pub length: Animated<f32>,
    /// Particle lifetime in seconds.
    pub lifespan: f32,
    /// Particle blending operation.
    pub blend: ParticleBlend,
    /// Texture atlas row and column counts.
    pub atlas: [u32; 2],
    /// Draws particle heads, tails, or both.
    pub geometry: ParticleGeometry,
    /// Trail length control.
    pub tail_length: f32,
    /// Particle age at the intermediate appearance stage.
    pub middle_age: f32,
    /// Appearance at birth, intermediate age, and death.
    pub stages: [LifeStage; 3],
    /// Head life, head decay, tail life, and tail decay atlas intervals.
    pub frames: [FrameInterval; 4],
    /// Document texture used by the particles.
    pub texture: Option<TextureId>,
    /// Game replaceable-texture category.
    pub replaceable_id: u32,
    /// Emission occurs in bursts instead of continuously.
    pub squirt: bool,
    /// Signed draw ordering plane.
    pub priority_plane: i32,
    /// Bypasses scene lighting.
    pub unshaded: bool,
    /// Sorts particle primitives by distance.
    pub sort_far_to_near: bool,
    /// Emits along a line.
    pub line_emitter: bool,
    /// Bypasses scene fog.
    pub unfogged: bool,
    /// Uses a quad aligned to the local XY plane.
    pub xy_quad: bool,
}
/// Authored ribbon parameters.
#[derive(Clone, Debug)]
pub struct Ribbon {
    /// Ribbon extent above its anchor.
    pub height_above: Animated<f32>,
    /// Ribbon extent below its anchor.
    pub height_below: Animated<f32>,
    /// Ribbon opacity multiplier.
    pub opacity: f32,
    /// Ribbon RGB tint.
    pub color: [f32; 3],
    /// Ribbon segment lifetime in seconds.
    pub lifespan: f32,
    /// Atlas cell selected initially.
    pub texture_slot: u32,
    /// Ribbon segments emitted per second.
    pub emission_rate: u32,
    /// Texture atlas row and column counts.
    pub atlas: [u32; 2],
    /// Model appearance slot drawn by the ribbon.
    pub material_slot: Option<MaterialSlotId>,
    /// Acceleration applied to ribbon segments.
    pub gravity: f32,
}
/// Authored popcorn parameters.
#[derive(Clone, Debug)]
pub struct Popcorn<'a> {
    /// Effect particle lifetime in seconds.
    pub lifespan: Animated<f32>,
    /// Effect emission rate.
    pub emission_rate: Animated<f32>,
    /// Initial effect particle speed.
    pub speed: Animated<f32>,
    /// Effect RGB tint.
    pub color: [f32; 3],
    /// Effect opacity multiplier.
    pub opacity: f32,
    /// Game replaceable-texture category.
    pub replaceable_id: u32,
    /// External effect resource.
    pub effect: AssetAddress<'a>,
    /// Visibility guide for named animations.
    pub animation_visibility: Cow<'a, str>,
    /// Bypasses scene lighting.
    pub unshaded: bool,
    /// Sorts primitives by distance.
    pub sort_far_to_near: bool,
    /// Bypasses scene fog.
    pub unfogged: bool,
    /// Enables effect scaling.
    pub scaling: bool,
}
/// Warcraft-specific actor attached to a hierarchy node.
#[derive(Clone, Debug)]
pub enum Feature<'a> {
    /// Particles that instantiate resources.
    ResourceParticles(ResourceParticles<'a>),
    /// Atlas-textured sprite particles.
    SpriteParticles(SpriteParticles),
    /// Trailing ribbon geometry.
    Ribbon(Ribbon),
    /// External Popcorn effect.
    Popcorn(Popcorn<'a>),
    /// Facial animation resource.
    FaceAnimation(AssetAddress<'a>),
}
impl ValidateLinks for Feature<'_> {
    fn visit_links(&self, visitor: &mut dyn FnMut(Link)) {
        match self {
            Self::ResourceParticles(value) => {
                value.spawn_model.visit_links(visitor);
                value.emission_rate.visit_links(visitor);
                value.gravity.visit_links(visitor);
                value.longitude.visit_links(visitor);
                value.latitude.visit_links(visitor);
                value.lifespan.visit_links(visitor);
                value.speed.visit_links(visitor);
            }
            Self::SpriteParticles(value) => {
                value.speed.visit_links(visitor);
                value.variation.visit_links(visitor);
                value.latitude.visit_links(visitor);
                value.gravity.visit_links(visitor);
                value.emission_rate.visit_links(visitor);
                value.width.visit_links(visitor);
                value.length.visit_links(visitor);
                value.texture.visit_links(visitor);
                if [value.lifespan, value.tail_length, value.middle_age]
                    .iter()
                    .any(|v| !v.is_finite())
                {
                    visitor(Link::InvalidValue);
                }
                if value
                    .stages
                    .iter()
                    .any(|v| !v.scale.is_finite() || v.color.iter().any(|c| !c.is_finite()))
                {
                    visitor(Link::InvalidValue);
                }
            }
            Self::Ribbon(value) => {
                value.height_above.visit_links(visitor);
                value.height_below.visit_links(visitor);
                value.material_slot.visit_links(visitor);
                if [value.opacity, value.lifespan, value.gravity]
                    .iter()
                    .any(|v| !v.is_finite())
                {
                    visitor(Link::InvalidValue);
                }
                if value.color.iter().any(|v| !v.is_finite()) {
                    visitor(Link::InvalidValue);
                }
            }
            Self::Popcorn(value) => {
                value.effect.visit_links(visitor);
                value.lifespan.visit_links(visitor);
                value.emission_rate.visit_links(visitor);
                value.speed.visit_links(visitor);
                if [value.opacity].iter().any(|v| !v.is_finite()) {
                    visitor(Link::InvalidValue);
                }
                if value.color.iter().any(|v| !v.is_finite()) {
                    visitor(Link::InvalidValue);
                }
            }
            Self::FaceAnimation(value) => value.visit_links(visitor),
        }
    }
}
