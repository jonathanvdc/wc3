//! Curve-driven particle vocabulary selected by the model-shader profile.
pub use crate::model::LegacyAtlas;
use crate::model::M2ParticleAppearance;
/// Curves evaluated against normalized particle age.
pub type Appearance<'a> = M2ParticleAppearance<'a>;
use crate::model::M2ParticleEmitter;
/// Complete logical particle generator definition.
pub type ParticleEmitter<'a> = M2ParticleEmitter<'a>;
