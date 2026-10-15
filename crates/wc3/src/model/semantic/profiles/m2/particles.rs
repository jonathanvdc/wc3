//! Model-clock emission and particle-age appearance are distinct logical controls.
use super::values::{ParticleBlend, ParticleFlags, ParticleGenerator, ParticleHeadTail};
use crate::model::semantic::{Animated, Curve, Link, Resource, TextureId, ValidateLinks, ZeroVec};
/// Appearance evaluated against normalized particle age rather than a model clip.
#[derive(Clone, Debug, Default)]
pub struct Appearance<'a> {
    /// Legacy atlas interpolation segments, when independently authored.
    pub legacy_atlas: Option<LegacyAtlas>,

    /// RGB multiplier over normalized age.
    pub color: Option<Curve<'a, [f32; 3]>>,
    /// Opacity fraction over normalized age.
    pub alpha: Option<Curve<'a, f32>>,
    /// Two-dimensional particle scale over normalized age.
    pub scale: Option<Curve<'a, [f32; 2]>>,
    /// Per-axis random scale variation.
    pub scale_variation: [f32; 2],
    /// Head and tail flipbook cell curves.
    pub head_uv: Option<Curve<'a, u16>>,
    /// Tail flipbook cell curve.
    pub tail_uv: Option<Curve<'a, u16>>,
}
/// Head and tail atlas segment endpoints used by classic particle shaders.
#[derive(Clone, Debug, Default)]
pub struct LegacyAtlas {
    /// Head cells during the lifespan interval.
    pub head_lifespan: [u16; 2],
    /// Head cells during the decay interval.
    pub head_decay: [u16; 2],
    /// Tail cells during the lifespan interval.
    pub tail_lifespan: [i16; 2],
    /// Tail cells during the decay interval.
    pub tail_decay: [i16; 2],
}
/// Complete logical particle generator definition.
#[derive(Clone, Debug)]
pub struct Particle<'a> {
    /// Runtime emitter identifier.
    pub id: u32,
    /// Generator behavior and extension bits.
    pub flags: ParticleFlags,
    /// Particle compositing operation.
    pub blend: ParticleBlend,
    /// Emission shape.
    pub generator: ParticleGenerator,
    /// External particle-color table category.
    pub color_index: u16,
    /// Ordered particle texture resources.
    pub textures: Vec<TextureId>,
    /// Model geometry resource used for model particles.
    pub model: Option<Resource<'a>>,
    /// Resource containing child emitters.
    pub child_model: Option<Resource<'a>>,
    /// Legacy particle shape selector whose client meaning is unresolved.
    pub legacy_shape: u8,
    /// Render head, tail, or both.
    pub head_tail: ParticleHeadTail,
    /// Per-layer UV scales.
    pub texture_scale: [f32; 2],
    /// Tile rotation or signed ordering parameter.
    pub texture_tile_rotation: i16,
    /// Atlas rows and columns.
    pub atlas: [u16; 2],
    /// Initial velocity magnitude in model units per second.
    pub speed: Animated<f32>,
    /// Fractional variation of initial velocity.
    pub speed_variation: Animated<f32>,
    /// Vertical emission spread in radians.
    pub vertical_range: Animated<f32>,
    /// Horizontal emission spread in radians.
    pub horizontal_range: Animated<f32>,
    /// Acceleration vector in model units per second squared.
    pub gravity: Animated<[f32; 3]>,
    /// Particle lifetime in seconds.
    pub lifespan: Animated<f32>,
    /// Random lifetime variation in seconds.
    pub lifespan_variation: f32,
    /// Particles emitted per second.
    pub rate: Animated<f32>,
    /// Random variation of emission rate.
    pub rate_variation: f32,
    /// Plane Y extent or sphere maximum radius.
    pub area_width: Animated<f32>,
    /// Plane X extent or sphere minimum radius.
    pub area_length: Animated<f32>,
    /// Z coordinate of the source point for aimed emission.
    pub z_source: Animated<f32>,
    /// Particle-age appearance.
    pub appearance: Appearance<'a>,
    /// Tail extent parameter; client time-versus-distance meaning is unresolved.
    pub tail_length: f32,
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
    /// Emission path in model coordinates.
    pub spline: ZeroVec<'a, [f32; 3]>,
    /// Animated emission toggle.
    pub enabled: Animated<bool>,
    /// Layer/component UV centers followed by ranges, in texture units.
    pub scroll: Option<[f32; 8]>,
}
impl ValidateLinks for Particle<'_> {
    fn visit_links(&self, v: &mut dyn FnMut(Link)) {
        self.textures.visit_links(v);
        self.speed.visit_links(v);
        self.speed_variation.visit_links(v);
        self.vertical_range.visit_links(v);
        self.horizontal_range.visit_links(v);
        self.gravity.visit_links(v);
        self.lifespan.visit_links(v);
        self.rate.visit_links(v);
        self.area_width.visit_links(v);
        self.area_length.visit_links(v);
        self.z_source.visit_links(v);
        self.enabled.visit_links(v)
    }
}
