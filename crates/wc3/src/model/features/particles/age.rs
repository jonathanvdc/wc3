//! Appearance evaluated against normalized particle age.
use crate::model::common::IntoOwned;
use crate::model::Curve;
use glam::{Vec2, Vec3};
/// Appearance evaluated against normalized particle age rather than a model clip.
#[derive(Clone, Debug, Default, IntoOwned)]
pub struct M2ParticleAppearance<'a> {
    /// Legacy atlas interpolation segments, when independently authored.
    pub legacy_atlas: Option<LegacyAtlas>,

    /// RGB multiplier over normalized age.
    pub color: Option<Curve<'a, Vec3>>,
    /// Opacity fraction over normalized age.
    pub alpha: Option<Curve<'a, f32>>,
    /// Two-dimensional particle scale over normalized age.
    pub scale: Option<Curve<'a, Vec2>>,
    /// Per-axis random scale variation.
    pub scale_variation: Vec2,
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
